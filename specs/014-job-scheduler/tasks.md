# Tasks: Built-in Job Scheduler for Procedures

**Input**: Design documents from `/specs/014-job-scheduler/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Includes corresponding `cargo nextest` integration or unit tests for the scheduler logic and system schema migration. 

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
- `src/parser/` for AST and parsing logic

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, dependency additions, and foundational system structures.

- [X] T001 Add `cron = "0.12"` dependency to `Cargo.toml`.
- [X] T002 [P] Create `src/storage/jobs.rs` and define constants for `system.cron` and `system.cron_runs` table names and schemas.

---

## Phase 2: User Story 3 - System Schema Migration (Priority: P2)

**Goal**: Migrate existing internal catalogs to the new `system` schema concept to set the stage for `system.cron`.

**Independent Test**: Can be tested by starting the database and asserting that standard procedures (e.g. `_sys_procedures`) reside in `system.procedures` instead.

### Tests for User Story 3 ⚠️

- [X] T003 [P] [US3] Create or update failing tests in `tests/system_tables.rs` verifying that data from `system.functions`, `system.procedures`, etc., can be read and that old `_sys_*` queries fail.

### Implementation for User Story 3

- [X] T004 [P] [US3] Update constants in `src/storage/procedures.rs` to use `system.procedures`.
- [X] T005 [P] [US3] Update constants in `src/storage/functions.rs` to use `system.functions`.
- [X] T006 [P] [US3] Update constants in `src/storage/statistics.rs` to use `system.table_stats` and `system.column_stats`.
- [X] T007 [P] [US3] Update constants in `src/storage/triggers.rs` to use `system.triggers`.
- [X] T008 [US3] Update `src/storage/mvcc/engine.rs` to create the `system` schema on boot and execute migration queries to copy old `_sys_*` data to `system.*`.
- [X] T009 [US3] Ensure all references inside `src/executor/ddl.rs`, `src/executor/mod.rs`, and `src/executor/information_schema.rs` point to the new `system.*` tables.
- [X] T010 [US3] Run `make test` to verify the tests created in T003 pass and engine boots correctly.

**Checkpoint**: At this point, the system schema correctly houses all metadata, ready for jobs.

---

## Phase 3: User Story 1 - Create and Manage Job Schedules (Priority: P1)

**Goal**: Implement the DDL syntax (`CREATE/ALTER/DROP SCHEDULE`) and store/remove configurations in the `system.cron` system table.

**Independent Test**: Can be tested via unit tests executing the DDL statements and verifying `system.cron` contents.

### Tests for User Story 1 ⚠️

- [X] T011 [P] [US1] Create a failing test in `tests/jobs.rs` or update parser tests in `src/parser/statements.rs` that tests parsing and execution of `CREATE SCHEDULE`, `ALTER SCHEDULE`, and `DROP SCHEDULE`.

### Implementation for User Story 1

- [X] T012 [P] [US1] Add `SCHEDULE`, `CRON`, `ACTIVE` to the lexer keywords in `src/parser/token.rs`.
- [X] T013 [P] [US1] Define `CreateScheduleStatement`, `AlterScheduleStatement`, and `DropScheduleStatement` structures in `src/parser/ast.rs`.
- [X] T014 [US1] Implement parsing logic for the new schedule statements in `src/parser/statements.rs`.
- [X] T015 [US1] Add handling for the new AST nodes in `src/executor/ddl.rs` to insert, update, and delete rows in the `system.cron` table.
- [X] T016 [US1] Run `make lint` and fix any warnings in parser/executor logic.
- [X] T017 [US1] Run `make test` to verify the tests created in T011 pass.

**Checkpoint**: At this point, the DDL is fully functional, and schedules are successfully persisted to disk.

---

## Phase 4: User Story 2 - Autonomous Job Execution (Priority: P1)

**Goal**: Spawn a background thread that periodically evaluates active schedules against the system time, executes the jobs, and logs the results.

**Independent Test**: Create an integration test in `tests/` that creates a highly frequent job (e.g., every second), waits, and asserts that `system.cron_runs` has entries and the target table is updated.

### Tests for User Story 2 ⚠️

- [X] T018 [P] [US2] Create a failing integration test in `tests/jobs_test.rs` that verifies end-to-end background execution and logging.

### Implementation for User Story 2

- [ ] T019 [P] [US2] Create the core scheduler logic (e.g. `src/executor/scheduler.rs` or `src/api/scheduler.rs`) containing the background worker thread.
- [ ] T020 [US2] Implement the loop inside the worker: read active jobs from `system.cron` via an internal `Executor` or direct engine scan.
- [ ] T021 [US2] Implement the time-matching logic using the `cron` crate to find the next execution time and efficiently `sleep` the thread.
- [ ] T022 [US2] Implement execution logging: insert `'RUNNING'` into `system.cron_runs`, execute the command using `Executor`, then update to `'SUCCESS'` or `'FAILED'`.
- [ ] T023 [US2] Update `Database::open` in `src/api/database.rs` to spawn the background thread and bind it to the `DatabaseInner` lifetime for clean shutdown.
- [ ] T024 [US2] Run `make test` to ensure the background thread operates flawlessly without blocking the main runtime.

**Checkpoint**: At this point, jobs execute autonomously based on their cron expressions.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and code quality.

- [ ] T025 [P] Verify `make license` passes and all new `.rs` files have the correct Apache-2.0 header.
- [ ] T026 Verify `unwrap()` and `expect()` are not used inappropriately in the new scheduler loop.
- [ ] T027 Code cleanup: Ensure thread shutdown signals are handled cleanly so test suites don't hang.
