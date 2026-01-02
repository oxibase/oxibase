# Plan: Adding Stored Procedures with Debug Support to Oxibase-Rhai

Based on analysis of the codebase and provided resources, here's a comprehensive plan to add stored procedures and ensure debug capability.

## Current State
- Oxibase supports user-defined functions (UDFs) via pluggable backends (Rhai, Deno, Python)
- DDL syntax: `CREATE FUNCTION name(params) RETURNS type LANGUAGE lang AS 'code'`
- Rhai backend executes scripts and returns single values
- No stored procedure support yet (procedures vs. functions: procedures execute statements without returning values)

## Key Assumptions
- "Stored procedures" means Rhai scripts stored in the database that can execute multiple SQL statements within transactions (similar to PostgreSQL's plpgsql or MySQL procedures, but using Rhai syntax)
- Procedures should support transaction control (BEGIN/COMMIT/ROLLBACK) internally
- Debugging focuses on Rhai script execution using the provided DAP/Zed resources

## Implementation Steps

1. **Extend Function System for Procedures**
   - Add `Procedure` to `FunctionType` enum in `src/functions/mod.rs`
   - Create `StoredProcedure` trait similar to `ScalarFunction` but for void-returning execution
   - Extend `BackendRegistry` to support `ProceduralBackend` trait with `execute_procedure` method

2. **Add DDL Support**
   - Extend parser (`src/parser/statements.rs`) for `CREATE PROCEDURE` and `CALL`/`EXECUTE` statements
   - Add `CreateProcedureStatement` and `CallProcedureStatement` AST nodes
   - Update executor DDL logic (`src/executor/ddl.rs`) to handle procedure creation/storage

3. **Implement Procedure Execution**
   - Add transaction context to backends for executing statements within procedures
   - Extend Rhai backend to parse and execute multiple statements (DDL/DML) from procedure body
   - Add procedure execution path in main executor, parallel to function calls

4. **Set Up Rhai Debugging Infrastructure**
   - Create DAP adapter binary that wraps Rhai's `Engine::register_debugger`
   - Implement DAP JSON-RPC over stdin/stdout for launch/attach modes
   - Map DAP requests to Rhai debugger commands and Rhai events to DAP responses

5. **Zed Integration**
   - Package DAP adapter as Zed extension with `extension.toml`
   - Configure for stored procedure debugging with breakpoints and variable inspection
   - Test integration using existing Rhai test files

## Debugging Workflow
1. Developer writes Rhai procedure script
2. Uses `CREATE PROCEDURE` to store it
3. Launches debugger in Zed with DAP adapter
4. Sets breakpoints in procedure code
5. Calls procedure via `CALL` statement
6. Steps through execution, inspects variables using DAP protocol

## Testing & Validation
- Unit tests for procedure parsing/execution
- Integration tests with transaction scenarios
- Debug adapter tests with DAP clients
- Performance benchmarks vs. equivalent SQL

## Potential Challenges
- Transaction isolation in procedures (nested transactions?)
- Error handling and rollback on procedure failures
- Variable scoping between procedure calls and SQL context

## Questions for Clarification
1. Should procedures support nested transactions, or only access the calling transaction?
2. Any specific Rhai features you want to leverage (e.g., custom types, modules)?
3. Priority: full SQL procedure syntax or simplified Rhai-only procedures?

This plan leverages the existing modular architecture and extends it naturally.