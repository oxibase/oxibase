# Implementation Plan: [FEATURE]

**Branch**: `019-drop-procedure` | **Date**: 2026-05-16 | **Spec**: [link](../spec.md)
**Input**: Feature specification from `specs/019-drop-procedure/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implementation of the `DROP PROCEDURE` SQL statement, allowing users to delete existing stored procedures from the database, optionally supporting `IF EXISTS` to ignore non-existent procedures silently.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: thiserror, anyhow, parking_lot, dashmap, tokio (if pg-server), rhai, boa_engine (js), rustpython-vm (python)
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (No new external microservices/APIs)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity?
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)?
- [x] **Safe Rust**: Are errors properly propagated? (No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or unjustified `unsafe`)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`?

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── api/           # Public Database API
├── core/          # Core types (Value, Row, Schema, Error)
├── executor/      # Query execution engine
├── functions/     # Built-in functions (scalar, aggregate, window)
├── optimizer/     # Cost-based query optimizer
├── parser/        # SQL parser (lexer, AST, parser)
├── storage/       # Storage engine and MVCC
└── bin/           # CLI binary
tests/             # Integration tests
```

**Structure Decision**: This feature primarily impacts `src/parser/` for AST and parsing additions (`ast.rs` and `statements.rs`) and `src/executor/` for handling execution in DDL logic (`ddl.rs`).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., Unsafe Code Used] | [Performance critical loop] | [Safe abstraction was measured to be X% slower in benchmark Y] |
| [e.g., Memory Allocation] | [External library requirement] | [No zero-copy alternatives exist for dependency Z] |
