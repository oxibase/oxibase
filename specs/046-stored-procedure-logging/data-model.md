# Data Models & AST Representation

## 1. Target Schema: `system.logs`

The procedural logging APIs write indirectly to the internal `system.logs` system table.

| Column | Data Type | Description |
| :--- | :--- | :--- |
| `id` | `INTEGER` | Auto-incrementing primary key |
| `timestamp` | `TIMESTAMP` | Event timestamp (UTC) |
| `level` | `TEXT` | Log severity: `'ERROR'`, `'WARN'`, `'INFO'`, `'DEBUG'`, `'TRACE'` |
| `target` | `TEXT` | Target of the log, populated with the executing stored procedure name (or `'user_procedure'` fallback) |
| `message` | `TEXT` | Log message |
| `json_fields` | `TEXT` | Optional structured JSON attributes |
| `trace_id` | `TEXT` | OpenTelemetry standard active correlation trace ID |
| `span_id` | `TEXT` | OpenTelemetry standard active correlation span ID |

## 2. PL/SQL AST Node Representation

The PL/SQL abstract syntax tree (AST) in `src/functions/plsql/ast.rs` will be extended with a new statement node:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // ... existing variants
    Log {
        level: String,      // Parsed as identifier (INFO, WARN, etc.)
        message: Expression, // Evaluates to text/string
    },
}
```
This represents a parsed `LOG <level>, <expression>;` statement.
During interpretation, the `message` expression is evaluated to a string, and the `log_message` helper is called with the evaluated level and message.
