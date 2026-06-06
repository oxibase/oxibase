---
description: "Task list for Implement System Metrics"
---

# Tasks: Implement System Metrics

**Input**: Design documents from `/specs/033-implement-metrics/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), data-model.md, contracts/metrics.md

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Verify project compiles (`cargo build`)

---

## Phase 2: User Story 1 - Emit and Persist Internal Metrics (Priority: P1) 🎯 MVP

**Goal**: Automatically collect and persist internal performance metrics (like query counts) into the `system.metrics` table.

**Independent Test**: `tests/metrics_ingestion_test.rs`

### Tests for User Story 1 ⚠️

- [x] T010 [P] [US1] Create integration test in `tests/metrics_ingestion_test.rs` to verify metrics emission and persistence

### Implementation for User Story 1

- [x] T011 [P] [US1] Create `src/common/metrics.rs` module and define `MetricEvent` entity.
- [x] T012 [P] [US1] Implement `SystemMetricsLayer` in `src/common/metrics.rs` to intercept `tracing::info!` events and send them to a channel.
- [x] T013 [P] [US1] Implement `start_metrics_flusher` thread in `src/common/metrics.rs` that reads from the channel and inserts into `system.metrics` via `MVCCEngine`.
- [x] T014 [US1] Register `metrics` module in `src/common/mod.rs`.
- [x] T015 [US1] Wire up `SystemMetricsLayer` and `start_metrics_flusher` in `src/bin/oxibase.rs`.
- [x] T016 [US1] Instrument query execution in `src/executor/mod.rs` to emit a `queries_total` counter metric.
- [x] T017 [US1] Run `make lint` and fix any warnings.
- [x] T018 [US1] Run `make test` to verify passing integration test and full test suite.

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently.

---

## Phase 3: User Story 2 - Query System Metrics (Priority: P2)

**Goal**: Query the `system.metrics` table using standard SQL to aggregate and analyze the internal performance metrics over time.

**Independent Test**: The test implemented in Phase 2 (`tests/metrics_ingestion_test.rs`) already covers querying the `system.metrics` table using `db.query()`.

### Implementation for User Story 2

- [x] T020 [US2] Verify that existing `executor` logic handles querying `system.metrics` correctly (schema is standard `TEXT`, `FLOAT`, `TIMESTAMP`, so no new logic needed).
- [x] T021 [US2] The `tests/metrics_ingestion_test.rs` acts as the verification for this user story.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T030 [P] Verify `make license` passes
- [x] T031 Verify `unwrap()` and `expect()` are not used inappropriately in library code (`src/common/metrics.rs`)
- [x] T032 Code cleanup and refactoring
