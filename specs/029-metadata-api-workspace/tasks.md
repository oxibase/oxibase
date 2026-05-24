# Implementation Tasks: Metadata API and Workspace App

**Feature**: `feat/028-create-workspace`

## Phase 1: Setup

- [x] T001 Create `src/server/meta.rs` file and register it in `src/server/mod.rs`
- [x] T002 Update `src/server/mod.rs` to namespace existing routes under `/api/data/*` instead of `/api/*`
- [x] T003 Update `table_exists` in `src/server/handlers.rs` to parse `schema.table` dot notation and default to `public` schema

## Phase 2: Foundational

- [x] T004 Create `install-workspace` CLI command module in `src/bin/oxibase.rs` 
- [x] T005 Implement `install-workspace` logic to execute `CREATE SCHEMA` and `CREATE TABLE` for `routes` and `templates`
- [x] T006 Add `POST /api/sql` endpoint in `src/server/handlers.rs` for executing raw SQL queries
- [x] T007 Implement raw SQL execution logic (handling both row-returning and non-row-returning queries via `state.db.query` and `state.db.execute`)

## Phase 3: Database Schema Explorer [US1]

- [x] T008 [P] [US1] Add `GET /api/meta/schemas` endpoint in `src/server/meta.rs` to query `information_schema.schemata` (or hardcoded list if virtual table missing)
- [x] T009 [P] [US1] Add `GET /api/meta/tables` endpoint in `src/server/meta.rs` to query `information_schema.tables`
- [x] T010 [P] [US1] Add `GET /api/meta/views` endpoint in `src/server/meta.rs` to query `information_schema.views`
- [x] T011 [P] [US1] Add `GET /api/meta/columns` endpoint in `src/server/meta.rs` to query `information_schema.columns`
- [x] T012 [P] [US1] Add `GET /api/meta/functions` and `/api/meta/procedures` endpoints in `src/server/meta.rs` to query `information_schema.functions`
- [x] T013 [P] [US1] Add `GET /api/meta/triggers` endpoint in `src/server/meta.rs` to query `system.triggers`
- [x] T014 [P] [US1] Add `GET /api/meta/indexes` and `/api/meta/constraints` endpoints in `src/server/meta.rs` to query `information_schema.statistics`
- [x] T015 [US1] Create `workspace_layout.html` base template with Tailwind CSS and Unpoly CDNs
- [x] T016 [US1] Create `workspace_sidebar.html` template using Unpoly to fetch and render the schema explorer tree
- [x] T017 [US1] Update `install-workspace` in `src/bin/oxibase.rs` to seed `workspace_layout.html` and `workspace_sidebar.html` into `templates.source`

## Phase 4: Raw SQL Execution Editor [US2]

- [x] T018 [US2] Create `workspace_sql_editor.html` template with a textarea and an Unpoly form targeting a results `<div>`
- [x] T019 [US2] Create `workspace_sql_results.html` template to format the JSON response from `/api/sql` into an HTML table (requires an intermediate handler to convert JSON to HTML for Unpoly, or Unpoly X-Up-Target handling)
- [x] T020 [US2] Add a server-side route `POST /workspace/sql` in `src/server/handlers.rs` that accepts form data, runs the SQL, and returns `workspace_sql_results.html` rendered via Minijinja
- [x] T021 [US2] Update `install-workspace` in `src/bin/oxibase.rs` to seed these new templates and routes

## Phase 5: Visual Table and Column Management [US3]

- [x] T022 [P] [US3] Add `POST /api/meta/tables` endpoint in `src/server/meta.rs` to generate and execute `CREATE TABLE` SQL
- [x] T023 [P] [US3] Add `DELETE /api/meta/tables/{schema}.{name}` endpoint in `src/server/meta.rs` to generate and execute `DROP TABLE` SQL
- [x] T024 [P] [US3] Add `POST /api/meta/columns` endpoint in `src/server/meta.rs` to generate and execute `ALTER TABLE ... ADD COLUMN` SQL
- [x] T025 [US3] Create `workspace_table_create.html` Unpoly modal template with a form submitting to `POST /api/meta/tables`
- [x] T026 [US3] Update `install-workspace` in `src/bin/oxibase.rs` to include the table/column management templates

## Phase 6: Visual Data Management [US4]

- [x] T027 [US4] Create `workspace_data_grid.html` template to display data rows fetched from a table
- [x] T028 [US4] Add a server-side route `GET /workspace/data/{schema}/{table}` in `src/server/handlers.rs` that fetches data via `db.query()` and renders `workspace_data_grid.html` via Minijinja
- [x] T029 [US4] Update `install-workspace` in `src/bin/oxibase.rs` to seed the data grid template and route

## Phase 7: Automated App Deployment [US5]

- [x] T030 [US5] Finalize `install-workspace` CLI command in `src/bin/oxibase.rs` to include all HTML templates and route definitions in a single execution
- [x] T031 [US5] Write integration test in `tests/server_meta_tests.rs` to verify `/api/meta/*` endpoints work correctly
- [x] T032 [US5] Write integration test in `tests/server_sql_tests.rs` to verify `/api/sql` endpoint works correctly

## Phase 8: Polish & Cross-Cutting Concerns

- [x] T033 Handle errors gracefully in the Unpoly frontend (e.g., displaying an error flash message if the SQL query fails)
- [x] T034 Run `make lint` and `make license` to ensure all new code complies with standards
- [x] T035 Verify zero-copy/memory allocation constraints are maintained in the new `meta.rs` module

## Dependencies

- Phase 2 depends on Phase 1
- Phase 3 depends on Phase 2
- Phase 4 depends on Phase 3
- Phase 5 depends on Phase 3
- Phase 6 depends on Phase 3
- Phase 7 depends on all previous phases

## Parallel Execution

- T008 through T014 can be implemented in parallel as they are independent metadata GET endpoints.
- T022, T023, and T024 can be implemented in parallel as they are independent metadata POST/DELETE endpoints.
