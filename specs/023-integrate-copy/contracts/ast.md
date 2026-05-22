# Internal Interface Contract: AST Additions

## Parser Grammar
The SQL parser must recognize the `COPY` statement format:

```sql
COPY table_name [(column_name [, ...])]
FROM 'file_path'
[ WITH ( option [, ...] ) ]

where option can be one of:
    FORMAT format_name  -- 'csv' or 'json'
    HEADER boolean      -- for CSV only
    DELIMITER 'char'    -- for CSV only, default ','
    NULL 'string'       -- string representing null
```

## Abstract Syntax Tree (AST)

The `Statement` enum in `src/parser/ast.rs` must include the new `Copy` variant:

```rust
pub enum Statement {
    // ... existing variants
    Copy(CopyStatement),
}

pub struct CopyStatement {
    pub token: Token,
    pub table_name: Identifier,
    pub columns: Vec<Identifier>,
    pub file_path: String,
    pub format: CopyFormat,
    pub header: bool,
    pub delimiter: u8,
    pub null_string: Option<String>,
}

pub enum CopyFormat {
    Csv,
    Json,
}
```

This contract defines the hand-off from the `parser` module to the `executor` module.