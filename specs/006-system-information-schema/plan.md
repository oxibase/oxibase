# Implementation Plan: System Information Schema

**Branch**: `006-system-information-schema` | **Date**: May 07, 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/006-system-information-schema/spec.md`

## Summary

The goal is to provide deep visibility into oxibase's internal state by exposing the in-memory metadata catalog (`MVCCEngine::schemas`) as queryable virtual tables under a new `system` schema. Concurrently, a standardized read-only `information_schema` will be implemented to expose this metadata in a way compatible with external SQL tools. Both of these schemas will act as "syntactic sugar" (virtual views) over the high-performance memory structures, avoiding the massive performance penalty of physically persisting schemas as table rows.

## Technical Context

**Language/Version**: Rust
**Primary Dependencies**: `thiserror`, `anyhow`, `parking_lot`/`dashmap` (for concurrency), existing parser/executor dependencies.
**Testing**: `cargo nextest` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero performance impact on the core engine. Virtual table generation must only allocate when the tables are actively queried.
**Constraints**:
- Strict adherence to ACID and MVCC principles.
- No `unwrap()`, `expect()`, `todo!()`, or `unimplemented!()` calls.
- Adherence to `make lint` and `make license`.
- Read-only restrictions for `information_schema` and `system` namespaces.

### Unresolved Aspects (NEEDS CLARIFICATION)
*(All resolved in Phase 0: Metadata architecture remains memory-first, exposed virtually.)*

## Constitution Check

- [x] **Mainframe Monolith**: Yes. This deeply integrates metadata into the core database engine itself rather than relying on external files or configurations.
- [x] **ACID & MVCC**: Yes. The core engine's transaction guarantees are untouched, as this change merely surfaces internal state virtually.
- [x] **Memory Efficiency**: Yes. We will ensure metadata querying avoids unnecessary allocations.
- [x] **Safe Rust**: Yes. Standard error propagation (`Result`) will be used.
- [x] **Tests First**: Yes. The changes will be validated via new integration tests.

## Project Structure

### Documentation (this feature)

```text
specs/006-system-information-schema/
├── plan.md              # This file
├── research.md          # Research on current metadata storage & bootstrap
├── data-model.md        # Schema definitions for system & information_schema
├── contracts/           # Not strictly needed (internal API only)
└── quickstart.md        # Guide on querying system metadata
```

### Source Code Impacts

```text
src/
├── parser/        # May need updates to ensure 'system' and 'information_schema' are reserved or recognized namespaces.
├── executor/      # Create dynamic virtual table generators for `system` schema. Update existing `information_schema.rs` to add missing standard columns. Update DML handlers to explicitly reject writes to these schemas.
```

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None currently identified | N/A | N/A |
