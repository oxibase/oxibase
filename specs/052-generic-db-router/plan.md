# Implementation Plan: Generic Database-Driven Router

**Branch**: `052-generic-db-router` | **Date**: Sun Jun 21 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/052-generic-db-router/spec.md`

## Summary

This feature refactors the Oxibase Server to support a pure, database-driven router architecture. It eliminates all workspace-specific hardcoded HTTP endpoints from the Rust server codebase (`src/server/handlers.rs` and `src/server/mod.rs`), replacing them with a single robust fallback template rendering handler (`dynamic_route_handler`). This handler parses and maps all request variables (path parameters, query filters, POST payloads) dynamically and forwards them as SQL parameters (`NamedParams`) to the template's context query execution.

## Technical Context

**Language/Version**: Rust 1.85+  
**Primary Dependencies**: axum, tower-http, minijinja, serde_json, anyhow  
**Testing**: cargo nextest (via `make test` / `make test-all`)  
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)  
**Performance Goals**: Zero-Copy memory efficiency, fast route lookup via segment-by-segment path pattern matching.  
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic, must pass `make lint` and `make license`.

## Constitution Check

*GATE: Passed before Phase 0 research. Re-checked after Phase 1 design.*

- [x] **Mainframe Monolith**: Yes, maintains the embedded/monolith architecture. No new external microservices/APIs are added.
- [x] **ACID & MVCC**: Yes, database operations executed inside routes run cleanly using the established storage/query engine.
- [x] **Memory Efficiency**: Yes, uses references and minimal allocations for path split parsing.
- [x] **Safe Rust**: Yes, errors are propagated correctly using `Result` and `anyhow`. No new unsafe blocks.
- [x] **Tests First**: Yes, covered by integration tests under `tests/` verifying dynamic route resolution and rendering.

## Project Structure

### Documentation (this feature)

```text
specs/052-generic-db-router/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── routes.md        # Schema contract for interface.routes
└── tasks.md             # Phase 2 tasks (not created by speckit.plan)
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
├── server/        # Axum web server and dynamic handlers (Main target)
└── bin/           # CLI binary and Workspace Seed Installer
```

**Structure Decision**: This feature primarily impacts `src/server/` (handlers, mod) and `src/bin/workspace/mod.rs` (workspace installers).

## Complexity Tracking

*No Constitution Check violations or unjustified complexities exist. The design follows the simplest possible path.*
