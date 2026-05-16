# Phase 0: Research & Clarifications

## Topic 1: AST Structure for DROP PROCEDURE
**Decision**: Create a `DropProcedureStatement` struct in `src/parser/ast.rs`.
**Rationale**: Aligns with existing AST structures like `DropTableStatement` and `DropFunctionStatement`. It should contain the `name` (an `ObjectName` or `Identifier`) and an `if_exists` boolean flag.

## Topic 2: Parser Integration
**Decision**: Update `src/parser/statements.rs` to parse `DROP PROCEDURE [IF EXISTS] <name>`.
**Rationale**: `DROP` statements are already handled in `parse_drop_statement()`. We will add a branch for `PROCEDURE` that calls a new `parse_drop_procedure_statement()` method.

## Topic 3: Executor Implementation
**Decision**: Add `execute_drop_procedure` in `src/executor/ddl.rs`.
**Rationale**: The procedure definition is likely stored in the database's catalog or system tables (like `oxibase.procedures` or similar, depending on the engine). We need to verify how `execute_create_procedure` works to know how to remove it. We'll implement `execute_drop_procedure` to remove the procedure by name, handling `IF EXISTS` logic.
