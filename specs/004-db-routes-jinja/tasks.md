# Tasks: Database-Driven Routes and Jinja Templates

## Phase 1: Setup
- [x] T001 Add `minijinja` dependency under the `server` feature flag in `Cargo.toml`

## Phase 2: Foundational
- [x] T002 Implement system tables initialization (create `routes.definitions` and `templates.source` tables on startup) in `src/server/mod.rs`
- [x] T003 Create template module file at `src/server/template.rs`
- [x] T004 Expose `template.rs` by adding `pub mod template;` in `src/server/mod.rs`

## Phase 3: User Story 1 (View dynamically rendered page based on database route)
*Goal*: Serve dynamically rendered HTML using templates stored in the database.
*Independent Test Criteria*: Add a template and route via SQL, send HTTP GET to the path, and assert HTML content matches the rendered output.

- [x] T005 [US1] Implement a custom `minijinja` loader to fetch template source from the database in `src/server/template.rs`
- [x] T006 [US1] Implement `dynamic_route_handler` to match requested method/path against `routes.definitions` in `src/server/handlers.rs`
- [x] T007 [US1] Extend `dynamic_route_handler` to execute `context_query` (if present) and serialize the result to JSON in `src/server/handlers.rs`
- [x] T008 [US1] Integrate the `minijinja` environment to render the matched template with JSON context in `src/server/handlers.rs`
- [x] T009 [US1] Register `dynamic_route_handler` as a fallback route to the Axum router in `src/server/mod.rs`
- [x] T010 [P] [US1] Write integration tests to verify static template rendering via HTTP in `tests/server_test.rs`
- [x] T011 [P] [US1] Write integration tests to verify dynamic template rendering with context query in `tests/server_test.rs`

## Phase 4: User Story 2 (Manage web routes and templates via SQL)
*Goal*: Instantly apply updates to routes and templates when modified via SQL.
*Independent Test Criteria*: Perform SQL updates to an existing template and route, verify the next HTTP request immediately reflects the new changes.

- [x] T012 [US2] Write integration tests to verify immediate visibility of template updates without server restart in `tests/server_test.rs`
- [x] T013 [P] [US2] Write integration tests to verify deleted routes return 404 in `tests/server_test.rs`

## Phase 5: Polish & Cross-Cutting Concerns
- [x] T014 Ensure error handling in `dynamic_route_handler` maps SQL and rendering errors to HTTP 500 without panicking in `src/server/handlers.rs`
- [x] T015 Run `make lint` and fix any warnings or code formatting issues across all modified files
