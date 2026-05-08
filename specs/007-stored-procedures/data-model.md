# Phase 1: Data Model & Contracts

## System Catalog: `system.procedures`

```sql
CREATE TABLE IF NOT EXISTS system.procedures (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    schema TEXT,
    name TEXT NOT NULL,
    parameters JSON NOT NULL, -- Serialized Vec<StoredProcedureParameter>
    language TEXT NOT NULL,
    code TEXT NOT NULL
);
```

## Internal Rust Models

```rust
// In src/parser/ast.rs

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterMode {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureParameter {
    pub mode: ParameterMode,
    pub name: Identifier,
    pub data_type: String,
}

pub struct CreateProcedureStatement {
    pub token: Token,
    pub procedure_name: FunctionName,
    pub parameters: Vec<ProcedureParameter>,
    pub language: String,
    pub body: String,
    pub or_replace: bool,
}

pub struct CallStatement {
    pub token: Token,
    pub procedure_name: FunctionName,
    pub arguments: Vec<Expression>,
}
```

```rust
// In src/storage/procedures.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProcedureParameter {
    pub mode: ParameterMode,
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProcedure {
    pub id: i64,
    pub schema: Option<String>,
    pub name: String,
    pub parameters: Vec<StoredProcedureParameter>,
    pub language: String,
    pub code: String,
}
```
