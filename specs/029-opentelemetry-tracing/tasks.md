---
description: "Task list for OpenTelemetry Tracing System implementation"
---

# Tasks: OpenTelemetry Tracing System

**Input**: Design documents from `/specs/029-opentelemetry-tracing/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Includes `cargo nextest` integration or unit tests for the feature. 
**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format Conventions

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- File paths are explicitly mentioned.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependencies

- [ ] T001 Add `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry`, `crossbeam-channel`, and `tokio` to `Cargo.toml`
- [ ] T002 Define `SpanEvent` data model in `src/common/tracing.rs`
- [ ] T003 Define `IS_TELEMETRY_THREAD` thread-local variable in `src/common/tracing.rs`

---

## Phase 2: Foundational (Trace Layer & Queue)

**Purpose**: Establish the internal tracing layer and lock-free queue

- [ ] T004 Implement `SystemTraceLayer` skeleton implementing `tracing_subscriber::Layer` in `src/common/tracing.rs`
- [ ] T005 Setup `crossbeam-channel` creation and plumbing for tracing in database initialization in `src/api/database.rs`

---

## Phase 3: User Story 1 - Trace Export Configuration (Priority: P1) 🎯 MVP

**Goal**: Configure database to export query traces to standard external endpoint if configured.

**Independent Test**: Verify tracing registry initializes OTLP exporter when env var is set.

### Tests for User Story 1 ⚠️

- [ ] T010 [P] [US1] Create test verifying OTLP exporter initialization based on env var in `tests/tracing_export.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Implement global `tracing_subscriber` registry configuration in `src/common/tracing.rs`
- [ ] T012 [US1] Add logic to detect `OTEL_EXPORTER_OTLP_ENDPOINT` and attach OTLP pipeline to registry in `src/common/tracing.rs`
- [ ] T013 [US1] Initialize the global tracing registry during database startup in `src/api/database.rs`

---

## Phase 4: User Story 2 - Query Lifecycle Instrumentation (Priority: P1)

**Goal**: Automatically instrument core query lifecycle phases (Parsing, Planning, Execution) with spans.

**Independent Test**: Unit tests verifying `tracing::info_span!` emission during query steps.

### Tests for User Story 2 ⚠️

- [ ] T020 [P] [US2] Create test verifying parser, planner, and executor emit spans with metadata in `tests/tracing_instrumentation.rs`

### Implementation for User Story 2

- [ ] T021 [P] [US2] Instrument `Parser::parse_program` with `#[tracing::instrument]` or inline spans in `src/parser/mod.rs`
- [ ] T022 [P] [US2] Instrument `QueryPlanner::create_plan` with spans in `src/optimizer/mod.rs`
- [ ] T023 [P] [US2] Instrument `Executor::execute_statement` and critical DML paths with spans in `src/executor/mod.rs`
- [ ] T024 [US2] Ensure metadata (e.g., query string, transaction ID) is explicitly attached to the relevant spans across `src/parser/mod.rs`, `src/optimizer/mod.rs`, and `src/executor/mod.rs`

---

## Phase 5: User Story 3 - Background Trace Ingestion into System Tables (Priority: P2)

**Goal**: Automatically capture tracing spans and ingest them into the internal `system.traces` table.

**Independent Test**: Integration test querying `system.traces` after query execution.

### Tests for User Story 3 ⚠️

- [ ] T030 [P] [US3] Create integration test querying `system.traces` to verify span ingestion in `tests/tracing_ingestion.rs`
- [ ] T031 [P] [US3] Create test verifying recursive trace emission is prevented by executing internal loops in `tests/tracing_ingestion.rs`

### Implementation for User Story 3

- [ ] T032 [US3] Implement `on_close` method in `SystemTraceLayer` to extract span data, check `IS_TELEMETRY_THREAD`, and push `SpanEvent` to the channel in `src/common/tracing.rs`
- [ ] T033 [US3] Implement trace flusher background thread processing the `crossbeam-channel` in `src/common/tracing.rs`
- [ ] T034 [US3] In the flusher thread, set `IS_TELEMETRY_THREAD = true` around SQL insert execution in `src/common/tracing.rs`
- [ ] T035 [US3] Implement batch insertion logic into `system.traces` via internal database connection in `src/common/tracing.rs`
- [ ] T036 [US3] Spawn the trace flusher background thread during database startup in `src/api/database.rs`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T040 Verify `make lint` passes with no warnings and `make license` passes
- [ ] T041 Verify `unwrap()` and `expect()` are not used inappropriately in tracing code
- [ ] T042 Run `make test` to ensure all new and existing tests pass cleanly