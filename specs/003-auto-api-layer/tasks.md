# Implementation Tasks: Auto-API Layer

**Feature**: Auto-API Layer
**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Phase 1: Setup

**Goal**: Prepare the project structure, CLI entrypoint, and dependencies.

- [ ] T001 Update `Cargo.toml` with `server` feature and dependencies (`axum`, `tokio`, `tower-http`, `serde_json`).
- [ ] T002 Refactor `src/bin/oxibase.rs` to use `clap::Subcommand` for `repl` and `serve` commands.
- [ ] T003 Implement the `serve` command in `src/bin/oxibase.rs` (placeholder that prints "Server starting...").
- [ ] T004 Create `src/server/mod.rs` and configure the Axum `Router` and `AppState` (sharing an `Arc<Database>`).
- [ ] T005 Wire up the `serve` command to start the Axum server on the specified host and port.

## Phase 2: Foundational (Schema Validation & JSON)

**Goal**: Implement core utilities needed by all handlers.

- [ ] T006 Create `src/server/handlers.rs` and implement the `AppState` extractor.
- [ ] T007 Implement a helper function `table_exists` in `src/server/handlers.rs` that queries `information_schema.tables` to validate table existence.
- [ ] T008 Implement a helper function `value_to_json` (or extract existing one from CLI) to convert `oxibase::Value` to `serde_json::Value`.

## Phase 3: Scenario 2 & 3 - Endpoint Discovery & Invalid Routes (GET Base)

**Goal**: Support basic `GET /api/:table` to return all rows.

- [ ] T009 Implement `get_table` handler in `src/server/handlers.rs` taking `:table` as a path parameter.
- [ ] T010 Wire the `get_table` handler to the `GET /api/:table` route in `src/server/mod.rs`.
- [ ] T011 Update `get_table` to use `table_exists` and return 404 if not found.
- [ ] T012 Update `get_table` to execute `SELECT * FROM <table>` and serialize the result set to JSON.

## Phase 4: Scenario 5 - GET with Pagination, Selection, and Filtering

**Goal**: Support advanced GET query parameters (`limit`, `offset`, `select`, `order`, `[col]=eq.[val]`).

- [ ] T013 Create query parameter parsing structs in `src/server/handlers.rs` (e.g., `GetQueryParams`).
- [ ] T014 Update `get_table` to dynamically construct the `SELECT` clause based on the `select` parameter (default `*`).
- [ ] T015 Update `get_table` to dynamically construct the `WHERE` clause for `[col]=eq.[val]` exact match filtering.
- [ ] T016 Update `get_table` to dynamically construct the `ORDER BY` clause based on the `order` parameter.
- [ ] T017 Update `get_table` to dynamically construct `LIMIT` and `OFFSET` clauses.

## Phase 5: Scenario 4 - Inserting Data (POST)

**Goal**: Support `POST /api/:table` for creating rows.

- [ ] T018 Implement `insert_row` handler in `src/server/handlers.rs` for `POST /api/:table` taking a JSON body.
- [ ] T019 Update `insert_row` to validate table existence and return 404 if not found.
- [ ] T020 Update `insert_row` to dynamically construct and execute an `INSERT INTO <table> (cols) VALUES (vals)` statement.
- [ ] T021 Return 201 Created with the number of affected rows on success.

## Phase 6: Scenario 6 & 7 - Updating and Deleting Data (PATCH / DELETE)

**Goal**: Support `PATCH` and `DELETE` operations using exact match filtering.

- [ ] T022 [P] Implement `update_row` handler in `src/server/handlers.rs` for `PATCH /api/:table`.
- [ ] T023 [P] Update `update_row` to require `[col]=eq.[val]` query parameters, construct an `UPDATE` statement, and execute it.
- [ ] T024 [P] Implement `delete_row` handler in `src/server/handlers.rs` for `DELETE /api/:table`.
- [ ] T025 [P] Update `delete_row` to require `[col]=eq.[val]` query parameters, construct a `DELETE` statement, and execute it.

## Phase 7: Polish & Integration Tests

**Goal**: Ensure quality and test coverage.

- [ ] T026 Write integration tests in `tests/server_test.rs` to verify the full CRUD flow using Axum's testing utilities or a real HTTP client against a bound port.
- [ ] T027 Run `make lint` and `make test-all` to ensure no regressions and proper error propagation.
