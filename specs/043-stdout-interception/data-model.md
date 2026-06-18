# Data Model: Stdout Interception

This feature does not introduce new database tables or catalog structures, but it modifies the internal Logical Abstract Syntax Tree (AST) for PL/SQL execution.

## PL/SQL AST Additions

### `PlSqlStatement::Print`
- **Fields**:
  - `token`: `Token` - The `PRINT` or `RAISE` token for error reporting and positional tracking.
  - `expression`: `Expression` - The SQL expression to evaluate and print.
- **Relationships**: Part of the `PlSqlStatement` enum, representing a print or notice command inside a PL/SQL block.

## Context Tracking

### `Execution Context Output Buffer`
- Handled via `crate::functions::context::append_stdout`. It stores standard output generated during a script's execution. All Rhai and PL/SQL prints will be funneled to this existing structure.