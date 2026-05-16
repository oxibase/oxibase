# Implementation Plan: PL/SQL Scalar Functions

**Branch**: `feat/plsql-functions` | **Date**: 2026-05-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/021-plsql-scalar-functions/spec.md`

## Summary

Implement native PL/SQL scalar functions by extending the `RETURN` AST node to capture an optional expression, updating the parser to parse this expression, and modifying the interpreter to yield the evaluated value, finally executing it via the `execute` trait in the PL/SQL backend.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: thiserror, anyhow, parking_lot, dashmap, tokio (if pg-server), rhai, boa_engine (js), rustpython-vm (python)
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, relies on internal engine, removes need for external script engines for basic logic)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, functions execute within the existing transaction context)
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, passes around existing `Value` instances)
- [x] **Safe Rust**: Are errors properly propagated? (Yes, using `Result` instead of `unwrap()`)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, integration tests via `make test`)

## Project Structure

### Documentation (this feature)

```text
specs/021-plsql-scalar-functions/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (to be created)
```

### Source Code (repository root)

```text
src/
├── functions/
│   └── plsql/
│       ├── ast.rs         # Update PlSqlStatement::Return enum
│       ├── parser.rs      # Parse RETURN with optional expression
│       ├── interpreter.rs # Bubble up ExecutionStatus::Return(Value)
│       └── backend.rs     # Implement execute() method
tests/                     # Integration tests for PL/SQL functions
```

**Structure Decision**: This feature is highly isolated to the `src/functions/plsql/` module.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

None.
