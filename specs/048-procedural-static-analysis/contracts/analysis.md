# Contract: AST-in-AST Static Analysis API

This document specifies the exact API contracts, return types, and serialization formats for the script static analysis feature.

## 1. Public Rust Structs

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelatedObject {
    pub object_type: String, // "Table", "Procedure", "Function", "Dynamic"
    pub name: String,
}
```

- `object_type` values MUST be one of:
  - `"Table"`: For referenced tables or views
  - `"Procedure"`: For procedures invoked via `CALL` or `oxibase::call`
  - `"Function"`: For scalar or table-valued functions invoked in SQL expressions
  - `"Dynamic"`: To signal that non-statically-resolvable SQL strings are executed (the name in this case is also `"Dynamic"`)

## 2. Public Database Method

```rust
impl Database {
    /// Statically analyze a procedural script to detect referenced database objects.
    /// Returns a sorted list of unique related database objects.
    pub fn analyze_script(&self, script: &str, backend: &str) -> Result<Vec<RelatedObject>>;
}
```

- **Arguments**:
  - `script`: The script source code block to analyze.
  - `backend`: Case-insensitive scripting language identifier (e.g. `"rhai"`, `"python"`, `"py"`, `"plsql"`, `"sql"`).
- **Errors**:
  - Returns `Error::Internal` if the language backend is unsupported or if parsing/compilation fails.
