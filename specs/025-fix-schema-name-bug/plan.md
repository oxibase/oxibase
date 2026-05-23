# Implementation Plan: Fix Schema Name Bug

**Branch**: `025-fix-schema-name-bug` | **Date**: 2026-05-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/025-fix-schema-name-bug/spec.md`

## Summary

The Oxibase engine currently suffers from a "Schema Name Bug", where tables created in non-default schemas (e.g., `system.cron_runs`) are incorrectly stored in the `public` schema's hash map with the schema name prepended directly to the table name string. This feature will update the core `Schema` struct to maintain `schema_name` independently, and enhance the `MVCCEngine` methods and DDL executor to dynamically parse fully qualified table names (e.g., splitting by `.`), correctly targeting the specific schema namespace while maintaining backwards compatibility. 

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: rustc-hash, chrono
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: String parsing (`split`) should be minimal. Cache lookups should use lowercase variants.
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, pure internal Rust code change)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, the version stores continue to operate within the specific schema bucket)
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, leveraging existing Arc mechanisms, string splitting instead of cloning where possible)
- [x] **Safe Rust**: Are errors properly propagated? (Yes, utilizing Oxibase's core Error types)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes)

## Project Structure

### Documentation (this feature)

```text
specs/025-fix-schema-name-bug/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

**Structure Decision**: This feature impacts three primary modules:
1. `src/core/schema.rs` - Core schema definition
2. `src/storage/mvcc/engine.rs` - MVCC Engine schema resolution
3. `src/executor/ddl.rs` - DDL parsing/execution logic

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |