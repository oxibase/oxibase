---
description: "Task list for autoincrement-alter implementation"
---

# Tasks: autoincrement-alter

**Input**: Design documents from `/specs/022-autoincrement-alter/`
**Prerequisites**: plan.md, spec.md, data-model.md, quickstart.md

**Tests**: MUST include corresponding `cargo nextest` integration tests for new constraints added via `ALTER TABLE`.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Update the core data model parameters to support new constraints.

- [x] T001 Modify `Schema::modify_column` signature in `src/core/schema.rs` to accept `auto_increment` and `check_expr` parameters.
- [x] T002 Modify `Table::modify_column` signature in `src/storage/traits/table.rs` to match the new `Schema::modify_column` signature.
- [x] T003 Modify `modify_column` implementation in `src/storage/mvcc/table.rs` to use the new parameters and pass them down correctly.
- [x] T004 Modify `Engine::modify_column` signature in `src/storage/mvcc/engine.rs` to match the new signature.
- [x] T005 Modify `Engine::record_alter_table_modify_column` in `src/storage/traits/engine.rs` and `src/storage/mvcc/engine.rs` to serialize the new constraints.
- [x] T006 Update existing calls to `modify_column` and `record_alter_table_modify_column` in `src/executor/ddl.rs` and `src/storage/mvcc/transaction.rs` to pass `None` for the new parameters temporarily.

---

## Phase 2: User Story 1 - Add AUTOINCREMENT to existing column (Priority: P1) 🎯 MVP

**Goal**: Allow adding an `AUTOINCREMENT` constraint to an existing `INTEGER` primary key column using `ALTER TABLE`.

**Independent Test**: `tests/autoincrement_alter_test.rs`

### Tests for User Story 1 ⚠️

- [x] T010 [P] [US1] Create failing integration test in `tests/autoincrement_alter_test.rs` covering US1 acceptance scenarios.

### Implementation for User Story 1

- [x] T011 [US1] Update `src/executor/ddl.rs` inside `AlterTableOperation::ModifyColumn` to parse `ColumnConstraint::AutoIncrement` from `col_def.constraints`.
- [x] T012 [US1] In `src/executor/ddl.rs`, validate that if `AUTOINCREMENT` is requested, the `data_type` is an Integer type.
- [x] T013 [US1] In `src/executor/ddl.rs`, pass the extracted `auto_increment` flag to `table.modify_column(...)`.
- [x] T014 [US1] In `src/executor/ddl.rs`, pass the extracted `auto_increment` flag to `self.engine.record_alter_table_modify_column(...)`.
- [x] T015 [US1] Run `make lint` and fix any warnings.
- [x] T016 [US1] Run `make test` to verify the integration test for US1 passes.

**Checkpoint**: At this point, User Story 1 should be fully functional.

---

## Phase 3: User Story 2 - Add other constraints (e.g., UNIQUE, CHECK) via ALTER TABLE (Priority: P2)

**Goal**: Allow adding `UNIQUE` and `CHECK` constraints to existing columns via `ALTER TABLE`.

**Independent Test**: `tests/constraints_alter_test.rs`

### Tests for User Story 2 ⚠️

- [x] T020 [P] [US2] Create failing integration test in `tests/constraints_alter_test.rs` covering US2 acceptance scenarios (UNIQUE and CHECK).

### Implementation for User Story 2

- [x] T021 [US2] Update `src/executor/ddl.rs` inside `AlterTableOperation::ModifyColumn` to check for `ColumnConstraint::Check(expr)`.
- [x] T022 [US2] In `src/executor/ddl.rs`, pass the extracted `check_expr` string to `table.modify_column(...)` and `record_alter_table_modify_column(...)`.
- [x] T023 [US2] Update `src/executor/ddl.rs` inside `AlterTableOperation::ModifyColumn` to check for `ColumnConstraint::Unique`.
- [x] T024 [US2] In `src/executor/ddl.rs`, if `UNIQUE` is requested, call `table.create_index_with_type(...)` to create the unique index.
- [x] T025 [US2] Run `make lint` and fix any warnings.
- [x] T026 [US2] Run `make test` to verify the integration test for US2 passes.

**Checkpoint**: At this point, User Story 2 should be fully functional.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and code standards.

- [x] T030 Verify `make license` passes and fix headers if necessary (`./scripts/fix_copyrights.sh`).
- [x] T031 Verify `unwrap()` and `expect()` are not used inappropriately in any modified code.
- [x] T032 Verify no `todo!()` or `unimplemented!()` macros remain.
- [x] T033 Run `make test-all` to ensure no regressions in other modules or language backends.
