# Data Model

The FROM-First Syntax feature does not introduce new persistent data models or storage formats. It relies entirely on rewriting the Abstract Syntax Tree (AST) during the parsing phase.

## AST Mapping

- **Target Entity**: `SelectStatement` (defined in `src/parser/ast.rs`)
- **Mapping rules for `FROM tbl` (without `SELECT`)**:
  - `distinct`: `false`
  - `columns`: `vec![Expression::Star(...)]`
  - `table_expr`: `Some(Box::new(Expression::TableSource(...)))` for `tbl`
  - Other clauses (`where_clause`, `group_by`, etc.): As parsed or default `None`/empty.

- **Mapping rules for `FROM tbl SELECT x`**:
  - `distinct`: Depends on if `DISTINCT` is present after `SELECT`
  - `columns`: Parsed from the `SELECT` clause (e.g., `vec![x]`)
  - `table_expr`: `Some(Box::new(Expression::TableSource(...)))` for `tbl`
  - Other clauses (`where_clause`, `group_by`, etc.): As parsed or default `None`/empty.
