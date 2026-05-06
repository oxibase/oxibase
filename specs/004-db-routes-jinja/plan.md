# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement database-driven routing and HTML rendering by integrating `minijinja` into the Axum web server. The system will use wildcard routing to look up requested paths in a `routes.definitions` table, execute an associated context SQL query, and render a template from a `templates.source` table. This allows dynamic CMS-like behavior without server restarts. The Jinja engine will also be configured to allow template composition (e.g. `{% include %}`) by fetching other templates from `templates.source`.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `axum`, `tokio`, `serde_json`, `minijinja`
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency, sub-5ms rendering overhead.
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, the rendering engine is embedded directly into the database web server).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, template and route reads will use the existing transactional database engine).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, `minijinja` is selected specifically for its high performance and zero-copy capabilities).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, using standard Result types and Axum error mapping).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, through integration tests on the server module).

## Project Structure

### Documentation (this feature)

```text
specs/004-db-routes-jinja/
├── plan.md              # This file
├── research.md          # Research on Jinja engines
├── data-model.md        # routes.definitions and templates.source schemas
├── quickstart.md        # Guide to creating a route
├── contracts/
│   └── api.md           # Dynamic HTTP endpoint contract
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
src/
├── server/
│   ├── mod.rs           # Update router with wildcard fallback
│   ├── handlers.rs      # Implement dynamic route handler
│   └── template.rs      # [NEW] MiniJinja integration and execution
Cargo.toml               # Add minijinja dependency
```

**Structure Decision**: This feature primarily impacts the `server` module. A new `template.rs` (or similar logic inside `handlers.rs`) will be added to encapsulate `minijinja` logic. Dependency additions are isolated to the `server` feature flag.


