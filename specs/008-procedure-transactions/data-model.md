# Data Model & Contracts: Procedure Transactions

## Data Model (AST Additions)

This feature introduces new abstract syntax tree (AST) nodes to the PL/SQL parser. The core database schema and system catalogs remain unchanged.

### `src/functions/plsql/ast.rs`

The `PlSqlStatement` enum is extended with three new variants:

```rust
pub enum PlSqlStatement {
    // ... existing variants ...
    
    /// COMMIT transaction
    Commit(Token),
    
    /// ROLLBACK transaction
    Rollback(Token),
    
    /// BEGIN explicit transaction (no-op at runtime)
    BeginTransaction(Token),
}
```

## Contracts

### `SqlRunner` Trait

The `SqlRunner` trait acts as the bridge between the execution engines (Rhai, JS, Python, PL/SQL) and the core database executor. It is extended to support transaction control.

```rust
pub trait SqlRunner: Send + Sync {
    // Existing methods
    fn execute_query(&self, sql: &str) -> Result<Box<dyn crate::storage::traits::QueryResult>>;
    fn execute_ast(&self, stmt: &crate::parser::ast::Statement) -> Result<Box<dyn crate::storage::traits::QueryResult>>;
    
    // New methods for procedure transaction management
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
    fn begin(&self) -> Result<()>;
}
```

### Scripting APIs

For the scripting backends, the following functions will be injected into the execution context:

**Rhai (`LANGUAGE rhai`)**:
- `commit()` -> Result<(), Error>
- `rollback()` -> Result<(), Error>
- `begin()` -> Result<(), Error>

**JavaScript (`LANGUAGE js`)**:
- `commit()` -> void
- `rollback()` -> void
- `begin()` -> void

**Python (`LANGUAGE python`)**:
- `oxibase.commit()` -> None
- `oxibase.rollback()` -> None
- `oxibase.begin()` -> None
