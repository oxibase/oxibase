# Implementation Plan: Drop Procedure

**Branch**: `feat/plsql-functions` | **Date**: 2026-05-16 | **Spec**: `/Users/gabriel.maeztu/repos/oxibase-plsql-function/specs/019-drop-procedure/spec.md`
**Input**: Feature specification from `/specs/019-drop-procedure/spec.md`

## Summary

Implementation of the `DROP PROCEDURE` SQL statement, allowing users to delete existing stored procedures from the database. The implementation involves updating the SQL parser to recognize the syntax (including optional `IF EXISTS`), adding it to the AST, and updating the execution engine to handle removing the procedure from storage.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: thiserror, anyhow
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

**Unknowns**:
- **NEEDS CLARIFICATION**: Where are procedures currently stored in the system? (e.g., system catalog table vs memory map).
- **NEEDS CLARIFICATION**: How are procedures uniquely identified? (by name, or name + argument types like PostgreSQL?).
- **NEEDS CLARIFICATION**: How does the parser currently handle `CREATE PROCEDURE` and where is the AST definition for statements located?

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? Yes, this is a core SQL feature.
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? Yes, dropping a procedure will be a transactional operation.
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? Yes.
- [x] **Safe Rust**: Are errors properly propagated? Yes.
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? Yes, integration tests will be added.

## Project Structure

### Documentation (this feature)

```text
specs/019-drop-procedure/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
└── quickstart.md        # Phase 1 output
```

### Source Code (repository root)

```text
src/
├── parser/        # AST and parser modifications for DROP PROCEDURE
├── executor/      # Execution logic to handle DropProcedure statement
├── storage/       # Removal of procedure from catalog/storage
└── api/           # Any necessary database API updates
```

**Structure Decision**: This feature impacts `src/parser/` (syntax/AST), `src/executor/` (execution of the drop), and `src/storage/` (catalog removal).
