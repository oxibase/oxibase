# Implementation Plan: autoincrement-alter

**Branch**: `022-autoincrement-alter` | **Date**: May 16 2026 | **Spec**: [specs/022-autoincrement-alter/spec.md](spec.md)
**Input**: Feature specification from `specs/022-autoincrement-alter/spec.md`

## Summary

Add support for the `AUTOINCREMENT`, `UNIQUE`, and `CHECK` constraints in `ALTER TABLE ... MODIFY COLUMN` statements. The parser already handles these syntax elements correctly and emits them inside the `ColumnDefinition` node for the target column. This feature extends the execution engine (`src/executor/ddl.rs`) and storage layer (`src/storage/...`) so that modifying a column properly applies these constraints. Specifically:
- Applying `AUTOINCREMENT` sets the column schema's auto-increment flag (validating it is an Integer type).
- Applying `UNIQUE` triggers the creation of a unique index on that column.
- Applying `CHECK` parses and stores the constraint expression in the column definition.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: thiserror, anyhow, parking_lot, dashmap
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
specs/022-autoincrement-alter/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── core/          # Core types (Schema, SchemaColumn)
├── executor/      # Query execution engine (ddl.rs)
├── storage/       # Storage engine and MVCC (engine.rs, table.rs, transaction.rs)
```

**Structure Decision**: This feature primarily impacts `src/executor/ddl.rs` to process the parsed constraints during ALTER TABLE, and `src/core/schema.rs` and `src/storage/*` modules to track the new configuration values through the `modify_column` workflow.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
