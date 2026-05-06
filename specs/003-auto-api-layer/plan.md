# Implementation Plan: Auto-API Layer

**Branch**: `003-auto-api-layer` | **Date**: 2026-05-06 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/003-auto-api-layer/spec.md`

## Summary

Implement an automatic REST API layer for Oxibase that dynamically exposes CRUD operations based on the underlying database schema. This feature leverages the `information_schema` to discover tables and endpoints on the fly. The server will be built using Axum and run as part of a new `oxibase serve` CLI command.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `axum`, `tokio`, `tower-http`, `serde_json`, `clap`
**Testing**: cargo nextest
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Fast dynamic routing, minimal serialization overhead.
**Constraints**: No `unwrap()`, strict ACID compliance via the existing `Database` API, must pass `make lint`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, the API is embedded directly into the database binary).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, it delegates to the existing `Database` engine).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations? (Yes, standard Axum routing and JSON serialization).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, using Axum's error handling and standard Rust Result types).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes).

## Project Structure

### Documentation (this feature)

```text
specs/003-auto-api-layer/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── api.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── bin/
│   └── oxibase.rs       # Refactor CLI to use clap subcommands
└── server/              # NEW MODULE
    ├── mod.rs           # Axum setup and routing
    └── handlers.rs      # Dynamic GET/POST handlers
Cargo.toml               # Add optional dependencies
```

**Structure Decision**: The CLI refactoring happens in `src/bin/oxibase.rs`. The new Axum server logic is encapsulated in a new `src/server/` module, only compiled when the `server` feature is enabled.
