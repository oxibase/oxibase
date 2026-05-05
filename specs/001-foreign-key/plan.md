# Implementation Plan: Foreign Key Constraints

**Branch**: `feat/fk` | **Date**: May 05, 2026 | **Spec**: [specs/001-foreign-key/spec.md](spec.md)
**Input**: Feature specification from `/specs/001-foreign-key/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

This feature adds support for foreign key constraints to enforce referential integrity within the database. It allows defining single-column foreign keys via `CREATE TABLE` and `ALTER TABLE`, and immediately validates `INSERT`, `UPDATE`, and `DELETE` operations to ensure data consistency, including support for `CASCADE` and `SET NULL` actions.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: None. Avoiding new Rust libraries per user request.
**Testing**: cargo nextest (via `make test` / `make test-all`). All code lines will be covered by a testing plan.
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency, < 15% regression on `INSERT`
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

**Structure Decision**: 
- `src/parser/`: Update lexer/parser for `FOREIGN KEY`, `REFERENCES`, `ON DELETE`, `ON UPDATE`, `CASCADE`, `SET NULL`.
- `src/core/`: Update `Schema` structures to hold foreign key metadata.
- `src/storage/`: Integrate constraint checks into table mutation logic.
- `src/executor/`: Implement execution logic for referential actions (`CASCADE`, `SET NULL`).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., Unsafe Code Used] | [Performance critical loop] | [Safe abstraction was measured to be X% slower in benchmark Y] |
| [e.g., Memory Allocation] | [External library requirement] | [No zero-copy alternatives exist for dependency Z] |
