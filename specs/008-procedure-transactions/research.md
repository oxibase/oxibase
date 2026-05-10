# Architecture & Research Findings: Procedure Transactions

## 1. Transaction Context Management for Procedures

**Needs Clarification:** How do we manage the transaction state during a `CALL` statement?
**Decision:** We will enhance `SqlRunner` with `begin()`, `commit()`, and `rollback()` methods. The `CALL` execution (`execute_call`) will wrap the procedure execution in an explicit transaction context if one is not already active. 
**Rationale:** In PostgreSQL, a procedure invoked without an explicit `BEGIN` runs in an implicit transaction. If it calls `COMMIT`, that implicit transaction is committed, and a *new* implicit transaction is automatically started for the remainder of the procedure. We will replicate this by temporarily setting `active_transaction` in the `Executor` during `CALL` execution. We must also track whether the transaction was started by the user (`BEGIN; CALL proc();`) or implicitly by the `CALL` statement itself.
**Mechanism:**
1. In `Executor::execute_call`, check if `self.active_transaction` is `Some`.
2. If `Some`, mark a flag `is_explicit_tx = true`.
3. If `None`, start a new transaction (`begin_transaction`), set it as `active_transaction`, and mark `is_explicit_tx = false`.
4. The `SqlRunner` implementation on `Executor` will implement `commit()` and `rollback()`.
5. Inside `SqlRunner::commit()`, if `is_explicit_tx == true`, it throws an "invalid transaction termination" error. If `false`, it calls `execute_commit_stmt()`, then *immediately* starts a new transaction and sets it as `active_transaction` so the procedure can continue.
6. Upon completion of the procedure, if `is_explicit_tx == false`, the current `active_transaction` is committed and cleared.

## 2. Scripting Language APIs

**Needs Clarification:** How to expose `begin()`, `commit()` and `rollback()` to the scripting backends (JS, Python, Rhai)?
**Decision:** 
- **Rhai**: Register `begin()`, `commit()` and `rollback()` as global functions in the Rhai engine instance.
- **Javascript (Boa)**: Register `begin()`, `commit()` and `rollback()` as global functions in the Boa context.
- **Python (RustPython)**: Add `begin()`, `commit()` and `rollback()` functions to the existing `oxibase` native module.
- All exposed functions will delegate to the `SqlRunner` instance that is passed via `with_sql_runner`.

## 3. PL/SQL Parser Updates

**Decision:** 
- Add `Commit`, `Rollback`, and `BeginTransaction` variants to `PlSqlStatement`.
- Update `PlSqlParser::parse_statement()` to recognize the `COMMIT`, `ROLLBACK`, and `BEGIN` tokens.
- In `PlSqlInterpreter::evaluate_statement()`, for `Commit` and `Rollback`, invoke the `SqlRunner`'s `commit()` or `rollback()`. For `BeginTransaction`, perform a no-op (return `Ok(())`).

## 4. SqlRunner Interface Extension

**Decision:**
```rust
pub trait SqlRunner: Send + Sync {
    // Existing methods
    fn execute_query(&self, sql: &str) -> Result<Box<dyn QueryResult>>;
    fn execute_ast(&self, stmt: &Statement) -> Result<Box<dyn QueryResult>>;
    
    // New methods
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
    fn begin(&self) -> Result<()>; // Exposed for completeness, mostly a no-op
}
```

## Alternatives Considered

- *Autonomous Transactions*: Executing procedures in an entirely separate transaction context. Rejected because it violates PostgreSQL compatibility (FR-005) and complicates MVCC visibility rules for users expecting standard behavior.
- *Failing on COMMIT in implicit transactions*: Rejected because the primary use case of `COMMIT` inside a procedure is to commit batch progress when called standalone.
