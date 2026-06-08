# Tasks: Trace Hierarchy Grouping

## Phase 1: Setup
*(No setup tasks needed as `tracing` and `tracing-opentelemetry` are already in `Cargo.toml` and functioning.)*

## Phase 2: Foundational
*(No foundational tasks needed for this feature.)*

## Phase 3: Database Query Execution Tracing [US1]
**Goal**: Wrap core public API methods with root spans so all inner execution logic connects to them.
**Independent Test Criteria**: A query executed via `Database::execute` or `Database::query` shows a root span `db.execute` or `db.query` that encompasses the `executor.execute` operations.

- [x] T001 [US1] Create a helper function `truncate_sql(sql: &str) -> &str` in `src/api/database.rs` to limit sql strings to 1024 chars.
- [x] T002 [US1] Add `tracing::info_span!("db.execute", sql = %truncate_sql(sql)).entered();` to `Database::execute` in `src/api/database.rs`.
- [x] T003 [US1] Add `tracing::info_span!("db.query", sql = %truncate_sql(sql)).entered();` to `Database::query` in `src/api/database.rs`.
- [x] T004 [US1] Add `tracing::info_span!("db.execute_with_timeout", sql = %truncate_sql(sql)).entered();` to `Database::execute_with_timeout` in `src/api/database.rs`.
- [x] T005 [US1] Add `tracing::info_span!("db.query_with_timeout", sql = %truncate_sql(sql)).entered();` to `Database::query_with_timeout` in `src/api/database.rs`.
- [x] T006 [US1] Add `tracing::info_span!("db.execute_named", sql = %truncate_sql(sql)).entered();` to `Database::execute_named` in `src/api/database.rs`.
- [x] T007 [US1] Add `tracing::info_span!("db.query_named", sql = %truncate_sql(sql)).entered();` to `Database::query_named` in `src/api/database.rs`.
- [x] T008 [US1] Create an integration test `tests/trace_hierarchy_test.rs` that validates root spans and parent linkages.

## Phase 4: Background Job Tracing [US2]
**Goal**: Wrap internal scheduled background tasks with a root span.
**Independent Test Criteria**: Firing a job results in a `job.execute` span with `job_id`.

- [x] T009 [US2] In `src/executor/scheduler.rs`, add `let _span = tracing::info_span!("job.execute", job_id = job_id, job_name = name).entered();` at the beginning of `JobScheduler::execute_job`.

## Phase 5: Network RPC/SQL Endpoint Tracing [US3]
**Goal**: Support distributed tracing by extracting OpenTelemetry headers in Axum and setting up overarching network request spans.
**Independent Test Criteria**: Sending a request with a `traceparent` header correctly nests the database spans under the provided trace context.

- [x] T010 [US3] In `src/server/handlers.rs`, update `get_table` to accept `axum::http::HeaderMap` and extract the `traceparent` using `opentelemetry::global::get_text_map_propagator`. Wrap execution in a `network.request` span inheriting the context.
- [x] T011 [US3] In `src/server/handlers.rs`, update `insert_row` identically to extract headers and set a root `network.request` span.
- [x] T012 [US3] In `src/server/handlers.rs`, update `update_row` identically to extract headers and set a root `network.request` span.
- [x] T013 [US3] In `src/server/handlers.rs`, update `delete_row` identically to extract headers and set a root `network.request` span.
- [x] T014 [US3] In `src/server/handlers.rs`, update `invoke_procedure` to use the extracted `headers` to set the parent tracing context explicitly using `tracing_opentelemetry::OpenTelemetrySpanExt`.
- [x] T015 [US3] In `src/server/handlers.rs`, update `execute_sql` to accept `axum::http::HeaderMap` and set the `network.request` root span context.
- [x] T016 [US3] In `src/server/handlers.rs`, update `workspace_execute_sql` identically to set the root span context.

## Phase 6: Polish
**Goal**: Ensure performance regressions are mitigated and tracing overhead is low.

- [x] T017 Ensure `make lint` and `cargo nextest run` pass across all modified components.
- [x] T018 Run micro-benchmarks to ensure < 2% overhead from `info_span` creations.

## Implementation Strategy

1. **MVP (Phase 3)**: Instrument `src/api/database.rs`. This delivers the highest value by immediately structuring all application-initiated SQL execution traces.
2. **Background Context (Phase 4)**: Isolate background noise from user traffic.
3. **Distributed Context (Phase 5)**: Link the system to external services.

## Dependencies

- Phase 3 (Database API) is independent.
- Phase 4 (Background Jobs) is independent.
- Phase 5 (Network Endpoints) depends on Phase 3, since the network handlers call the Database API and rely on those spans becoming children of the network context.

## Parallel Execution Opportunities

- T002-T007 in Phase 3 can all be implemented in parallel.
- T010-T016 in Phase 5 can all be implemented in parallel.
- Phase 3 and Phase 4 can be implemented in parallel.
