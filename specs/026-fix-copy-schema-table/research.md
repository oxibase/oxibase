# Research: COPY Schema-Qualified Table Syntax

## Unknowns Addressed

No specific `NEEDS CLARIFICATION` markers were identified in the spec. The problem is a straightforward syntax parser bug.

## Implementation Strategy

### Parser Modification
- **Decision**: Update `Parser::parse_copy_statement()` to use `self.parse_table_name()` instead of manually parsing a single `Identifier`.
- **Rationale**: `parse_table_name()` already handles parsing simple names (`table`), schema-qualified names (`schema.table`), and quoted identifiers (`"schema"."table"`). Reusing this method ensures consistency with how table names are parsed elsewhere in the engine (e.g., for `SELECT`, `INSERT`, `DROP TABLE`, etc.).

### AST Update
- **Decision**: Update `CopyStatement` in `src/parser/ast.rs` to change the `table_name` field type from `Identifier` to `TableName`.
- **Rationale**: `TableName` is the existing AST struct designed to hold an optional schema and a table name.

### Executor Update
- **Decision**: Update `src/executor/executor.rs` or `src/executor/copy.rs` (or equivalent execution logic handling `CopyStatement`) to properly extract the schema and table name from the `TableName` struct within `CopyStatement`.
- **Rationale**: The executor needs to be aware of the schema to route the data to the correct storage location.