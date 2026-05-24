# Implementation Plan: Metadata API and Workspace App

**Branch**: `feat/028-create-workspace` | **Date**: 2026-05-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/028-metadata-api-workspace/spec.md`

## Summary

Implement a full RESTful Metadata CRUD backend (`/api/meta/*`) supporting comprehensive schema object management (schemas, tables, views, columns, functions, procedures, triggers, constraints, indexes). Namespace existing data operations to `/api/data/*`, add a raw SQL execution endpoint (`/api/sql`), and deploy a self-contained Unpoly-driven HTML Workspace app directly from the database's template engine to provide a DataGrip-like GUI using server-side Minijinja rendering.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `axum`, `serde`, `serde_json`, `minijinja`
**Testing**: `cargo nextest run` via `make test`
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? Yes, the Unpoly app is served directly from the Oxibase server, maintaining a single deployable binary and allowing the server to handle all rendering logic.
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? Yes, the `/api/sql` and metadata operations map directly to engine calls that respect MVCC.
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? Yes.
- [x] **Safe Rust**: Are errors properly propagated? Yes, API errors will return structured JSON responses with appropriate HTTP status codes.
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? Yes, new API endpoints will have integration tests.

## Project Structure

### Documentation (this feature)

```text
specs/028-metadata-api-workspace/
├── plan.md              
├── research.md          
├── data-model.md        
├── quickstart.md        
└── tasks.md             
```

### Source Code (repository root)

```text
src/
├── server/
│   ├── mod.rs           # Update router for /api/meta, /api/data, /api/sql
│   ├── handlers.rs      # Add metadata CRUD handlers, sql handler, fix table_exists
│   └── meta.rs          # (New) Dedicated logic for metadata payload parsing -> SQL DDL
tests/
├── server_meta_tests.rs # (New) Integration tests for Metadata APIs
└── server_sql_tests.rs  # (New) Integration tests for raw SQL endpoint
src/bin/oxibase.rs       # (Update) Add `install-workspace` command to the CLI
```

**Structure Decision**: This feature primarily impacts the `server` module (Axum routes and handlers). We will introduce a new `src/server/meta.rs` to keep `handlers.rs` clean, separating the DDL translation logic from the raw HTTP handling.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | | |