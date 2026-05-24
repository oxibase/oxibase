# Implementation Tasks: Elaborate Existing Logs & Create Telemetry Tables

**Feature Directory**: `specs/027-elaborate-existing-logs`

## Phase 1: Setup
- [x] T001 Verify project structure and test execution environment.

## Phase 2: Foundational (Storage Definitions)
- [x] T002 Update `src/storage/logs.rs` schema to include `trace_id` and `span_id`.
- [x] T003 Create `src/storage/traces.rs` with `system.traces` table schema.
- [x] T004 Create `src/storage/metrics.rs` with `system.metrics` table schema.
- [x] T005 Update `src/storage/mod.rs` to export the new `traces` and `metrics` modules.

## Phase 3: Telemetry Initialization & Migration
**Story Goal**: Ensure telemetry tables are created or migrated upon database initialization.
**Independent Test**: Database starts successfully, `system.logs` has new columns, and querying `system.traces` or `system.metrics` returns empty results instead of "table not found".
- [x] T006 Update `src/executor/mod.rs` to add `ensure_traces_table_exists()` and `ensure_metrics_table_exists()`.
- [x] T007 Update `src/executor/mod.rs` `ensure_logs_table_exists()` logic to automatically migrate the table and add the trace columns.
- [x] T008 Update `ensure_system_schema_and_migrations()` in `src/executor/mod.rs` to call the new ensure methods.

## Phase 4: Internal Log Emission Fixes
**Story Goal**: Propagate empty trace context fields through the existing log flushing mechanism to avoid database insertion errors.
**Independent Test**: `cargo nextest run test_internal_log_capture` passes without column mismatch errors.
- [x] T009 Update `LogEntry` struct in `src/common/logging.rs` to include nullable `trace_id` and `span_id`.
- [x] T010 Update `InternalLogLayer::on_event` in `src/common/logging.rs` to populate `trace_id` and `span_id` with `None`.
- [x] T011 Update `insert_log_batch` in `src/common/logging.rs` to persist `trace_id` and `span_id` as part of the `row` vector.
- [x] T012 Update `tests/internal_logging_test.rs` to initialize `LogEntry` correctly with `None` values.

## Phase 5: Verification
- [x] T013 Run test suite (`cargo check && make test`) to verify there are no compilation errors or regressions.

---
**Note**: All tasks for this feature have already been completed and committed to the branch `027-elaborate-existing-logs`.
