# Implementation Plan: Stored Procedures (CREATE PROCEDURE / CALL)

**Branch**: `007-stored-procedures` | **Date**: 2026-05-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/007-stored-procedures/spec.md`

## Summary

Implement `CREATE OR REPLACE PROCEDURE` and `CALL` statements in the database parser and executor. Procedures will persist in a new system catalog (`system.procedures`). Unlike scalar functions, procedures don't return values directly via SELECT but through `OUT`/`INOUT` parameters (Postgres-style) and are invoked via `CALL`. We will support `LANGUAGE rhai` (default), `python`, `js`, and introduce a dedicated native PL/pgSQL interpreter (`LANGUAGE sql`/`plpgsql`) that is debugger-friendly for future DAP integration. The procedure's syntax will be strictly validated at creation time (`CREATE PROCEDURE`) before persistence.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: thiserror, anyhow, rhai (plus boa_engine/rustpython if features enabled).
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, no unnecessary allocations.

### Architecture Decisions (Research Phase Outcomes)

- **Storage**: Procedures will be stored in a new internal table `system.procedures` similar to `_sys_functions`, but reflecting `ParameterMode` (IN, OUT, INOUT) and lack of direct `return_type`.
- **Parser**: Add `CreateProcedureStatement` and `CallStatement` to `ast.rs`.
- **Validation**: During `execute_create_procedure`, the `ScriptingBackend::validate_code()` will be called. If it fails, the transaction aborts with a syntax error.
- **Execution**: The `CALL` execution will retrieve the procedure, execute it using the appropriate scripting backend or PL/pgSQL interpreter, and return a single-row result containing the updated `OUT`/`INOUT` parameter values.
- **PL/pgSQL Interpreter**: We will build a dedicated native interpreter for `LANGUAGE plpgsql`. The interpreter will maintain a `CallStack` and `Environment` to allow variable assignment, control flow (IF/WHILE), and make it easy to expose local state for a future Debug Adapter Protocol (DAP).

## Constitution Check

- [x] **Mainframe Monolith**: Yes. Logic remains embedded securely inside the database process via embedded engines.
- [x] **ACID & MVCC**: Yes. `CREATE PROCEDURE` uses standard DDL transactions. `CALL` operates within the caller's transaction scope for now.
- [x] **Memory Efficiency**: Yes. Procedure definitions are cached in memory (registry).
- [x] **Safe Rust**: Yes. Error handling propagates from the scripting engine via `Result`.
- [x] **Tests First**: Yes. `tests/` will be extended with `procedure_tests.rs`.

## Project Structure

### Documentation

```text
specs/007-stored-procedures/
├── plan.md              # This file
├── research.md          # Research/Architecture findings
├── data-model.md        # Procedure definition structure
└── contracts/           # Ast changes & Catalog structure
```

### Source Code Impacts

```text
src/
├── parser/ast.rs            # Add CreateProcedureStatement, CallStatement, ParameterMode
├── parser/statements.rs     # Parsing logic for CREATE PROCEDURE and CALL
├── storage/procedures.rs    # New! Handles system.procedures table and StoredProcedure
├── executor/ddl.rs          # Add execute_create_procedure
├── executor/execute.rs      # Add execution for CallStatement
├── functions/backends.rs    # Extend ScriptingBackend to handle OUT params or add ProcedureBackend trait
└── functions/plpgsql/       # New! Dedicated PL/pgSQL parser and interpreter
```

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
