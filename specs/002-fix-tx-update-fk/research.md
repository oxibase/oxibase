# Research: Fix Transaction Updates with Foreign Keys

## Problem Context
When executing the Oxibase CLI with a transaction that updates rows with foreign keys, a `ROLLBACK` fails to release the transaction's row claims. Subsequent operations (like `ALTER TABLE` constraint checks or further updates) fail with:
`Error: internal error: row <ID> has uncommitted changes from transaction <TXN_ID>`

## Findings

1. **Transaction Caching & Row Claims:**
   - When a transaction updates a row, it calls `TransactionVersionStore::put`, which adds the row to its `write_set` and claims the row in the parent `VersionStore`'s `uncommitted_writes`.
   - To release the claim, `TransactionVersionStore::rollback()` or `commit()` calls `release_all_claims()`.

2. **The Rollback Bug in `MvccTransaction`:**
   - `MvccTransaction::rollback()` attempts to roll back local changes by iterating over `self.tables`.
   - However, `self.tables` is rarely populated because `MvccTransaction::get_table()` delegates to the engine and explicitly notes: "For now, always get from engine (engine will handle caching internally)."
   - Because `self.tables` is empty, `table.rollback()` is never called.

3. **The Memory Leak in `MvccEngine`:**
   - `MvccEngine` caches the transaction-local stores in `self.txn_version_stores`.
   - `MvccEngine::commit_all_tables()` properly cleans up this cache using `cache.retain(...)`.
   - There is no equivalent `rollback_all_tables()` method. When a transaction rolls back, the `TransactionVersionStore` remains in the engine's cache indefinitely.
   - Since `TransactionVersionStore` does not implement `Drop`, its row claims are never released, permanently locking the rows.

## Decision

**Add `rollback_all_tables` to `TransactionEngineOperations` and `MvccEngine`.**

- **Rationale:** Just as `commit_all_tables` handles the centralized commit and cleanup of the engine's `txn_version_stores` cache, a symmetric `rollback_all_tables` will cleanly iterate over the cache, call `rollback()` on each store (which releases row claims), and then remove the transaction's entries from the cache.
- **Alternatives considered:** We could populate `self.tables` in `MvccTransaction::get_table()`, but the code comment explicitly warns that `Table` doesn't implement `Clone` and it's better to let the engine handle caching. We could also implement `Drop` for `TransactionVersionStore`, but explicit resource cleanup via `rollback_all_tables` is more predictable and avoids relying on garbage collection timing, though `Drop` can be added as a defense-in-depth measure.

## Implementation Plan

1.  **Modify `TransactionEngineOperations` (in `src/storage/mvcc/transaction.rs`)**
    - Add `fn rollback_all_tables(&self, txn_id: i64) -> Result<()>;`

2.  **Modify `MvccEngine` (in `src/storage/mvcc/engine.rs`)**
    - Implement `rollback_all_tables`:
      ```rust
      fn rollback_all_tables(&self, txn_id: i64) -> Result<()> {
          let cache = self.txn_version_stores().read().unwrap();
          for ((cached_txn_id, _), txn_store) in cache.iter() {
              if *cached_txn_id == txn_id {
                  txn_store.write().unwrap().rollback();
              }
          }
          drop(cache);
          let mut cache = self.txn_version_stores().write().unwrap();
          cache.retain(|(cached_txn_id, _), _| *cached_txn_id != txn_id);
          Ok(())
      }
      ```

3.  **Modify `MvccTransaction::rollback` (in `src/storage/mvcc/transaction.rs`)**
    - Call `ops.rollback_all_tables(self.id)` instead of iterating over `self.tables` to notify the engine.
    - Remove the existing loops for `table.rollback()` and `ops.rollback_table()` since `rollback_all_tables` handles it at the engine level.

4.  **Add `Drop` for `TransactionVersionStore` (in `src/storage/mvcc/version_store.rs`)** (Optional but recommended)
    - Implement `Drop` to call `self.release_all_claims()` to prevent any future leaks if a transaction panics or is dropped without commit/rollback.
