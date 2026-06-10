# Implementation Plan: Fix System Tables Creation

**Branch**: `038-fix-system-tables-creation` | **Date**: 2026-06-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/038-fix-system-tables-creation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

The current initialization logic uses `CREATE TABLE AS SELECT` for system tables like `system.functions`, which strips away critical schema constraints like `PRIMARY KEY` and `UNIQUE` and fails silently on fresh boots. The plan is to introduce dedicated `ensure_*_table_exists` initialization methods that execute the explicit schema definitions (`CREATE_FUNCTIONS_SQL`, etc.), just like `system.cron`. After ensuring strict creation, old metadata tables (`_sys_*`) will be migrated into the new schema via `INSERT INTO ... SELECT *` to preserve compatibility.

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
specs/038-fix-system-tables-creation/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
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

**Structure Decision**: This feature exclusively impacts `src/executor/mod.rs` where the engine boots up and provisions its internal schemas.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A       | N/A        | N/A                                 |
