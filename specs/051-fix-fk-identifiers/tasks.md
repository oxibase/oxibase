# Tasks: Schema-Qualified Foreign Keys

**Input**: Design documents from `/specs/051-fix-fk-identifiers/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are requested and defined to cover cross-schema foreign keys in `tests/foreign_key_test.rs`.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- `src/parser/` for AST definitions and parsing
- `src/executor/` for DDL, DML, and constraint verification
- `tests/` for integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify the codebase state and verify current test suite functionality before changes.

- [x] T001 Run `make lint` and `cargo nextest run` to ensure codebase is clean and currently passing.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core AST modifications to transition from `Identifier` to `TableName`.

- [x] T002 Refactor `TableConstraint::ForeignKey` and `ColumnConstraint::References` definitions in `src/parser/ast.rs` to use `TableName` instead of `Identifier` for the foreign table parameter.
- [x] T003 Update display formatting and pattern matches in `src/parser/ast.rs` to format `TableName` and compile correctly.

---

## Phase 3: User Story 1 - Create Table-level and Column-level Foreign Key Constraints referencing Tables in different Schemas (Priority: P1) 🎯 MVP

**Goal**: Support creating table-level and column-level cross-schema foreign keys.

**Independent Test**: `tests/foreign_key_test.rs`

### Implementation for User Story 1

- [x] T004 [P] [US1] Update SQL parsing logic in `src/parser/statements.rs` within `parse_column_or_constraint` (around line 1811) to parse the `foreign_table` as a `TableName` using `self.parse_table_name()?` instead of expecting an `Identifier`.
- [x] T005 [P] [US1] Update SQL parsing logic in `src/parser/statements.rs` within `ColumnConstraint::References` parsing (around line 2030) to parse the referenced table as a `TableName` using `self.parse_table_name()?` instead of expecting an `Identifier`.
- [x] T006 [US1] Update `create_table` constraint verification logic in `src/executor/ddl.rs` (around line 180) to resolve `referencing_schema`, and compute fully-qualified `foreign_full_name` and `referencing_full_name` for lookup, validation, metadata building, and `referenced_by` storage.
- [x] T007 [US1] Update `handle_referential_actions` delete/update verification in `src/executor/dml.rs` (around line 1801) to support both simple and fully-qualified referencing names.
- [x] T008 [US1] Add integration tests `test_cross_schema_foreign_key_and_cascade` in `tests/foreign_key_test.rs` covering cross-schema DDL validation and cascade verification.
- [x] T009 [US1] Run `make lint` and fix any formatting or warning issues.
- [x] T010 [US1] Run `cargo nextest run` to verify that all foreign key and cross-schema tests pass successfully.

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently.

---

## Phase 4: User Story 2 - Add Cross-Schema Foreign Key Constraints via ALTER TABLE (Priority: P2)

**Goal**: Support adding cross-schema foreign key constraints to an existing table using `ALTER TABLE`.

**Independent Test**: `tests/foreign_key_test.rs`

### Implementation for User Story 2

- [x] T011 [US2] Update `execute_alter_table` in `src/executor/ddl.rs` under `AlterTableOperation::AddConstraint` to resolve `referencing_schema` and build fully-qualified `foreign_full_name` and `referencing_full_name` for checking, querying, metadata building, and referenced catalog updates.
- [x] T012 [US2] Add integration test `test_alter_table_add_cross_schema_fk` in `tests/foreign_key_test.rs` to verify adding cross-schema constraint with existing/empty data.
- [x] T013 [US2] Run `cargo nextest run` to verify ALTER TABLE behavior across schemas.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Verifying standard compilation, formatting, and licensing requirements.

- [x] T014 Run `./scripts/fix_copyrights.sh` or `make license` to verify all license headers are present.
- [x] T015 Verify that no unwrap() or expect() statements were introduced in library code.
