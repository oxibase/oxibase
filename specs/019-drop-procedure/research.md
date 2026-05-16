# Research: Drop Procedure

## Clarifications from Technical Context

1. **Where are procedures currently stored in the system? (e.g., system catalog table vs memory map).**
   - **Decision**: They are stored in the `pg_proc` system catalog.
   - **Rationale**: `CREATE PROCEDURE` execution (in `src/executor/ddl.rs`) inserts rows into `pg_proc`. The database manages stored procedures using the `SystemCatalog` interface.
   - **Alternatives considered**: None, this is the existing mechanism for procedures.

2. **How are procedures uniquely identified? (by name, or name + argument types like PostgreSQL?).**
   - **Decision**: Currently identified by `(schema_name, procedure_name)`.
   - **Rationale**: Based on existing `SystemCatalog` methods like `get_procedure` which accept schema and name. Overloading by argument types is not currently implemented in this monolithic version.
   - **Alternatives considered**: N/A, we are following the current catalog design.

3. **How does the parser currently handle `CREATE PROCEDURE` and where is the AST definition for statements located?**
   - **Decision**:
     - AST definition is in `src/parser/ast.rs`. `CreateProcedureStatement` exists, and actually, `DropProcedureStatement` already exists in `ast.rs` along with `Statement::DropProcedure`.
     - Parser handles it in `src/parser/statements.rs`. `parse_drop_procedure_statement` is already implemented.
     - Execution is handled in `src/executor/ddl.rs`. `execute_drop_procedure` is partially or fully implemented.
   - **Rationale**: `grep` results show that `DropProcedureStatement` and its parser/executor hooks are already in the codebase.
   - **Alternatives considered**: N/A. The feature to "implement DROP PROCEDURE" seems to be mostly or fully implemented already in the parser/AST. I need to verify its completeness in `src/executor/ddl.rs`.
