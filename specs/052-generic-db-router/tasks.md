# Tasks: Generic Database-Driven Router

**Input**: Design documents from `/specs/052-generic-db-router/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Covered by Axum integration tests executed via `cargo nextest`.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Path Conventions

- `src/server/` for web server routing, fallback handler, and parameter mapping
- `src/bin/workspace/` for workspace seed data, templates, and database-side SQL logic
- `tests/` for integration and server endpoint tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify the baseline project compilation and testing state before making modifications.

- [X] T001 Verify project compiles before changes in `Cargo.toml`
- [X] T002 Run current tests with `cargo nextest` to verify baseline state in `tests/server_test.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement the generic matching and parameter binding infrastructure in the server fallback handler. These are blocking prerequisites for all subsequent user stories.

- [X] T003 [P] Implement path matching helper function segment-by-segment in `src/server/handlers.rs`
- [X] T004 [P] Implement HTTP query and JSON body extraction into common parameters in `src/server/handlers.rs`
- [X] T005 Bind unified parameters as `NamedParams` and query the database via `db.query_named` inside `dynamic_route_handler` in `src/server/handlers.rs`
- [X] T006 Insert rows as `'data'` and parameters as `'params'` inside Jinja render context in `src/server/handlers.rs`

---

## Phase 3: User Story 1 - Generic Router Dynamic Fallback Rendering (Priority: P1) 🎯 MVP

**Goal**: Enable completely generic fallback routing for simple static and parameterized dashboard pages, eliminating hardcoded Rust endpoints.

**Independent Test**: `tests/server_test.rs` (verifying dynamic routes `/workspace` and `/workspace/sidebar/data` render successfully through fallback)

### Tests for User Story 1
- [X] T010 [P] [US1] Create dynamic routing integration test verifying dynamic route matching in `tests/server_test.rs`

### Implementation for User Story 1
- [X] T011 [US1] Clean up and remove hardcoded workspace endpoints in `src/server/mod.rs`
- [X] T012 [US1] Clean up and remove hardcoded workspace handlers in `src/server/handlers.rs`
- [X] T013 [US1] Run `cargo nextest` to verify fallback router handles `/workspace` and `/workspace/sidebar/data` routes in `tests/server_test.rs`

---

## Phase 4: User Story 2 - Request Variable Binding & Parameter Forwarding (Priority: P1)

**Goal**: Extract request parameters from path, query string, and JSON body and forward to database.

**Independent Test**: `tests/server_test.rs` (verifying parameterized routing and named parameter binding)

### Tests for User Story 2
- [X] T020 [P] [US2] Create integration test verifying named parameter binding for dynamic routes in `tests/server_test.rs`

### Implementation for User Story 2
- [X] T021 [US2] Match route patterns like `/workspace/traces/{trace_id}` and extract `'trace_id'` parameter dynamically in `src/server/handlers.rs`
- [X] X022 [US2] Verify dynamic parameters are parsed from JSON bodies on POST routes in `src/server/handlers.rs`
- [X] T023 [US2] Ensure all extracted variables are injected into Jinja template context under the `'params'` key in `src/server/handlers.rs`

---

## Phase 5: User Story 3 - Database-Driven Telemetry & Domain Logic (Priority: P2)

**Goal**: Port all log counting, filtering, trace grouping, histograms and Gantt chart timelines entirely to database views and SQL queries.

**Independent Test**: `tests/server_test.rs` (verifying workspace functionality of logs and traces via database-driven queries)

### Implementation for User Story 3
- [X] T030 [P] [US3] Relocate trace timeline aggregation logic from `workspace_trace_view` to database view or SQL query in `src/bin/workspace/mod.rs`
- [X] T031 [P] [US3] Relocate log histogram counts logic from `workspace_observe_logs` to SQL query inside database in `src/bin/workspace/mod.rs`
- [X] T032 [P] [US3] Update log search and filtering query logic to use named params (`:level`, `:search`, `:limit`, `:offset`) inside database in `src/bin/workspace/mod.rs`
- [X] T033 [US3] Clean up and ensure no references to `'logs'`, `'traces'`, `'Gantt'` exist in `src/server/handlers.rs`
- [X] T034 [US3] Execute workspace seed installer and verify full application functions via dynamic router in `src/bin/workspace/mod.rs`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Code cleanup, formatting, license checks, and final verification.

- [X] T040 [P] Run `make lint` and resolve any Clippy/rustfmt warnings in `src/server/handlers.rs`
- [X] T041 Verify `make license` passes on all source files
- [X] T042 Perform final sanity check that no `unwrap()` or `expect()` calls were introduced inside `src/server/`
