# Data Model: Fix Transaction Updates with Foreign Keys

This feature does not introduce new user-facing database schemas, but it patches the internal data structures that manage Transactional Concurrency (MVCC).

## Internal Entities

### `TransactionVersionStore`
A transaction-local store that tracks uncommitted row versions and holds locks/claims on rows in the global `VersionStore`.

**Fields involved in the fix:**
- `write_set`: A map of `row_id` to `WriteSetEntry`.
- `parent_store`: A reference to the global `VersionStore`.

**State Transitions:**
1.  **Update (put):** Row is added to `write_set`. `parent_store.try_claim_row()` adds the row to `uncommitted_writes`.
2.  **Commit:** Replaces old versions in `parent_store` with new ones from `local_versions`. Calls `release_all_claims()` to remove the row from `uncommitted_writes`.
3.  **Rollback:** MUST call `release_all_claims()` to remove the row from `uncommitted_writes` and discard `local_versions`. (This was missing at the engine level).

### `MvccEngine`
The central engine that manages global tables and active transaction states.

**Fields involved in the fix:**
- `txn_version_stores`: A thread-safe cache (`DashMap` or `RwLock<HashMap>`) mapping `(txn_id, table_name)` to `TransactionVersionStore`.

**State Transitions:**
1.  **Transaction Start / Table Access:** An entry is inserted into `txn_version_stores`.
2.  **Transaction Commit:** `commit_all_tables()` flushes changes and removes the `txn_id` entries from `txn_version_stores`.
3.  **Transaction Rollback:** **NEW BEHAVIOR** `rollback_all_tables()` will explicitly call rollback on each cached store and remove the `txn_id` entries from `txn_version_stores`.
