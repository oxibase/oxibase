# Implementation Plan: Public Schema Fallback

**Branch**: `018-public-schema-fallback` | **Date**: May 16, 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/018-public-schema-fallback/spec.md`

## Summary

The goal is to ensure that all database objects (tables, procedures, functions, views, sequences, triggers) default to the active session schema (which defaults to `public`) if a schema is not explicitly provided. Currently, procedures, functions, and triggers inserted into system tables without a schema default to `NULL`, and views/sequences are stored globally. This feature unifies schema resolution across all objects by leveraging `ctx.current_schema().unwrap_or("public")` in `src/executor/ddl.rs` and organizing `views` and `sequences` in `src/storage/mvcc/engine.rs` by schema.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: None new required.
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? Yes.
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? Yes.
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? Yes.
- [x] **Safe Rust**: Are errors properly propagated? Yes.
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? Yes.

## Project Structure

### Documentation (this feature)

```text
specs/018-public-schema-fallback/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (to be generated)
```

### Source Code Impact

**Structure Decision**: This feature primarily impacts the `executor` and `storage` modules.

- `src/executor/ddl.rs`: The DDL execution logic for `CREATE PROCEDURE`, `CREATE FUNCTION`, `CREATE TRIGGER`, `CREATE VIEW`, `CREATE SEQUENCE`, etc., will be updated to fall back to `ctx.current_schema().unwrap_or("public")`.
- `src/storage/mvcc/engine.rs`: The storage definitions for `views` and `sequences` will be refactored from `FxHashMap<String, Arc<T>>` to `FxHashMap<String, FxHashMap<String, Arc<T>>>` (where the outer map is the schema name).
- `src/storage/traits/engine.rs`: Updates to engine trait definitions to handle `schema` properly for views and sequences.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None      | N/A        | N/A                                 |
