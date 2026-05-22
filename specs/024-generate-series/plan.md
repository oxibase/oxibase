# Implementation Plan: generate_series Table-Valued Function

**Branch**: `024-generate-series` | **Date**: 2026-05-23 | **Spec**: specs/024-generate-series/spec.md
**Input**: Feature specification from `/specs/024-generate-series/spec.md`

## Summary

Implement the `generate_series` table-valued function by porting it from the `stoolap` repository. This entails adding AST support for `FunctionTableSource`, parsing logic for functions in the `FROM` clause, and iterator-based execution logic to return sequences of values.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: thiserror, anyhow
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency (iterator-based execution, no `Vec` pre-allocation for outputs).
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`.

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
specs/024-generate-series/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── api/           # Public Database API
├── core/          # Core types (Value, Row, Schema, Error)
├── executor/      # Query execution engine (execute_tvf_source)
├── functions/     # Built-in functions (tvf.rs)
├── optimizer/     # Cost-based query optimizer
├── parser/        # SQL parser (ast.rs, statements.rs)
├── storage/       # Storage engine and MVCC
└── bin/           # CLI binary
tests/             # Integration tests (generate_series_test.rs)
```

**Structure Decision**: The feature primarily impacts `src/parser/` for parsing `FunctionTableSource`, `src/executor/` for executing it, `src/functions/` for the actual iteration logic, and `tests/` for validation.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
