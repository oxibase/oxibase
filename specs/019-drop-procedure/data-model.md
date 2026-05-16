# Data Model: Drop Procedure

## AST Additions (`src/parser/ast.rs`)

### `DropProcedureStatement`
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct DropProcedureStatement {
    pub token: Token,
    pub procedure_name: FunctionName,
    pub if_exists: bool,
}
```

## Statement Enum (`src/parser/ast.rs`)
Add the variant to `Statement`:
```rust
pub enum Statement {
    // ...
    DropProcedure(DropProcedureStatement),
}
```

## AST Evaluation (`src/executor/ddl.rs`)
The `DropProcedureStatement` needs to be processed by the `execute_drop_procedure` function, modifying the catalog similar to other DROP commands.
