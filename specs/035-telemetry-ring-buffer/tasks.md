---
description: "Task list for Telemetry Ring Buffer feature implementation"
---

# Tasks: Telemetry Ring Buffer Table

**Input**: Design documents from `/specs/035-telemetry-ring-buffer/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- `src/api/` for database entrypoint logic
- `src/executor/` for query execution
- `src/storage/` for MVCC and engine state
- `tests/` for integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Initialize basic `SystemRingBufferTable` struct in `src/storage/mvcc/ring_buffer_table.rs`
- [ ] T002 Add `SystemRingBufferTable` to `src/storage/mvcc/mod.rs`
- [ ] T003 Verify project compiles (`cargo build`)

---

## Phase 2: User Story 1 - Non-blocking Telemetry Insertion (Priority: P1) 🎯 MVP

**Goal**: Insert internal telemetry without contending for MVCC locks, transactional overhead, or the WAL.

**Independent Test**: `tests/tracing_ingestion.rs`, `tests/logging_ingestion.rs`, `tests/metrics_ingestion_test.rs`

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T010 [P] [US1] Create or update tests to verify WAL size/activity does not increase during telemetry ingestion in `tests/tracing_ingestion.rs` (or create a new benchmark/integration test).

### Implementation for User Story 1

- [ ] T011 [US1] Implement the `Table` trait for `SystemRingBufferTable` in `src/storage/mvcc/ring_buffer_table.rs` (focusing on `insert`, returning empty for `get_pending_versions`, no-ops for `commit`/`rollback`).
- [ ] T012 [US1] Modify `MVCCEngine::get_table_for_transaction` or table creation in `src/storage/mvcc/engine.rs` to instantiate `SystemRingBufferTable` when `table_name` starts with `system.`.
- [ ] T013 [P] [US1] Update `SpanEvent` structure to defer JSON formatting in `src/common/tracing.rs`.
- [ ] T014 [P] [US1] Update `LogEntry` structure to defer JSON formatting in `src/common/logging.rs`.
- [ ] T015 [P] [US1] Update `MetricEvent` structure to defer JSON formatting in `src/common/metrics.rs`.
- [ ] T016 [US1] Update flusher threads (`start_trace_flusher`, `start_log_flusher`, `start_metrics_flusher`) to perform JSON serialization and construct `Row`s before calling `insert_batch`.
- [ ] T017 [US1] Run `make test` to verify passing integration tests for tracing, logging, and metrics ingestion.

**Checkpoint**: At this point, User Story 1 should be fully functional, telemetry should bypass the WAL and use the new ring buffer table.

---

## Phase 3: User Story 2 - Memory-Bounded Telemetry Storage (Priority: P1)

**Goal**: The internal system tables retain a fixed amount of recent telemetry, capping memory usage.

**Independent Test**: `tests/telemetry_capacity_test.rs`

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T020 [P] [US2] Create integration test `tests/telemetry_capacity_test.rs` that inserts 100,001 logs/traces and asserts `SELECT COUNT(*)` returns exactly 100,000.

### Implementation for User Story 2

- [ ] T021 [US2] Update `SystemRingBufferTable::insert` and `insert_batch` in `src/storage/mvcc/ring_buffer_table.rs` to drop the oldest elements (using `pop_front()`) when the internal `VecDeque` length exceeds the configured capacity.
- [ ] T022 [US2] Implement `row_count()` in `SystemRingBufferTable` to return the current length of the buffer.
- [ ] T023 [US2] Ensure `table.insert()` correctly maintains the fixed capacity without memory leaks.
- [ ] T024 [US2] Run `make test` to verify the capacity bounding works as expected.

**Checkpoint**: At this point, User Story 2 is complete. The system tables should strictly adhere to their maximum memory limits.

---

## Phase 4: User Story 3 - Telemetry Querying via Standard SQL (Priority: P2)

**Goal**: Query the system tables using standard SQL `SELECT` queries to diagnose issues.

**Independent Test**: `tests/telemetry_query_test.rs` (or leverage existing tests if they already do this).

### Tests for User Story 3 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T030 [P] [US3] Create or update `tests/telemetry_query_test.rs` to perform standard `SELECT` queries with `WHERE` and `ORDER BY` clauses against `system.logs` and `system.traces`.

### Implementation for User Story 3

- [ ] T031 [US3] Implement `SystemRingBufferTable::scan()` in `src/storage/mvcc/ring_buffer_table.rs` to return a `Scanner` over the ring buffer contents.
- [ ] T032 [US3] Implement `SystemRingBufferTable::collect_all_rows()` and `collect_projected_rows()` in `src/storage/mvcc/ring_buffer_table.rs` for optimized reads without iterators.
- [ ] T033 [US3] Implement `SystemRingBufferTable::collect_rows_with_limit()` to support efficient pagination and limits on telemetry queries.
- [ ] T034 [US3] Ensure the SQL executor correctly interacts with the new table implementation during read queries.
- [ ] T035 [US3] Run `make test` to verify SQL querying functions correctly on the ring buffer tables.

**Checkpoint**: At this point, users can query telemetry data with standard SQL without transactional overhead.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T040 [P] Ensure schema changes to `system.*` tables are rejected in `src/executor/ddl.rs` or `src/storage/mvcc/engine.rs`.
- [ ] T041 [P] Verify `make license` passes and all new files have the correct Apache-2.0 header.
- [ ] T042 Verify `unwrap()` and `expect()` are not used inappropriately in the new `SystemRingBufferTable`.
- [ ] T043 Run `make lint` and fix any clippy warnings.
- [ ] T044 Run `make coverage-check` to ensure test coverage has not dropped below the threshold.