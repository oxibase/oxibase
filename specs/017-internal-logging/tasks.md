# Implementation Tasks: Comprehensive Internal Logging System

**Feature**: Comprehensive Internal Logging System
**Branch**: `017-internal-logging`
**Spec**: [spec.md](./spec.md)
**Plan**: [plan.md](./plan.md)

## Task Generation Summary

- Total Tasks: 9
- Tasks by User Story:
  - Setup: 1
  - Foundational: 0
  - US1 (System Table Ingestion): 6
  - US2 (Structured Console Output): 1
  - Polish: 1
- **MVP Scope**: Complete US1 to establish the core internal routing before refining the console output.

## Dependency Graph

```text
[Setup]
  │
  ▼
[Phase 3: US1 - System Table Ingestion]
  │
  ▼
[Phase 4: US2 - Structured Console Output]
  │
  ▼
[Phase 5: Polish & Cross-Cutting]
```

## Implementation Strategy

We will build the core data structure and channel first (US1), integrate it into the executor to create the physical table, and finally wire up the background flusher. Once the internal logging is functional, we will adjust the CLI to format console output as JSON (US2).

---

## Phase 1: Setup

- [x] T001 Add `crossbeam-channel` dependency to `Cargo.toml`

## Phase 2: Foundational

*(No foundational tasks required for this feature)*

## Phase 3: User Story 1 (Priority: P1) - System Table Ingestion

**Goal**: Write high-severity internal logs to a `system.logs` system table.
**Independent Test Criteria**: Trigger an `INFO` trace and query `system.logs` to see the entry.

**Tasks**:
- [x] T002 [US1] Create `src/storage/logs.rs` defining the `system.logs` DDL string and constants
- [x] T003 [US1] Update `ensure_system_schema_and_migrations` in `src/executor/mod.rs` to create the `system.logs` table on startup
- [x] T004 [US1] Create `src/common/logging.rs` and define the `LogEntry` struct and the `IS_LOG_FLUSHER` thread-local flag
- [x] T005 [US1] Implement `InternalLogLayer` inside `src/common/logging.rs` extending `tracing_subscriber::Layer` to push entries to a `crossbeam_channel::Sender`
- [x] T006 [US1] Implement the background flusher thread logic in `src/common/logging.rs` to read from the channel receiver and batch-insert into `system.logs` via `MVCCEngine`
- [x] T007 [US1] Update `Executor::new` in `src/executor/mod.rs` to spawn the background flusher thread alongside other initializations

## Phase 4: User Story 2 (Priority: P2) - Structured Console Output

**Goal**: Format console output as structured JSON.
**Independent Test Criteria**: Run `RUST_LOG=info cargo run --features cli` and verify console logs are single-line JSON.

**Tasks**:
- [x] T008 [US2] Update `src/bin/oxibase.rs` to initialize `tracing_subscriber` using `fmt::layer().json()` composed with the new `InternalLogLayer`

## Phase 5: Polish & Cross-Cutting Concerns

- [x] T009 Refine the flusher's batching strategy (e.g., flush on 100 entries or every 1 second) to prevent delays and optimize insert performance in `src/common/logging.rs`
