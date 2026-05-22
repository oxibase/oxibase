---
description: "Task list template for feature implementation"
---

# Tasks: integrate-copy

**Input**: Design documents from `/specs/023-integrate-copy/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/ast.md

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

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Update `Cargo.toml` to include the `csv` crate dependency (and verify `serde_json` is present).
- [x] T002 Verify project compiles (`cargo build`).

---

## Phase 2: Foundational (AST & Parsing)

**Goal**: Extend the SQL Parser to understand the `COPY FROM` syntax so that the rest of the engine can operate on it.

**Independent Test**: Can be tested via unit tests inside the parser module (or simply compilation).

- [x] T003 [P] Add `CopyFormat` enum and `CopyStatement` struct to `src/parser/ast.rs` (as per `contracts/ast.md`).
- [x] T004 [P] Add `Copy` variant to the `Statement` enum in `src/parser/ast.rs`.
- [x] T005 Update lexer (`src/parser/lexer.rs` / `src/parser/token.rs`) to ensure keywords `COPY`, `FORMAT`, `HEADER`, `DELIMITER`, and `CSV` / `JSON` are recognized correctly if not already present.
- [x] T006 Implement parsing logic for `COPY FROM` in `src/parser/statements.rs`.
- [x] T007 Add AST parsing unit tests in `src/parser/statements.rs` or relevant test file.

---

## Phase 3: User Story 1 - Bulk Loading Data from CSV (Priority: P1) 🎯 MVP

**Goal**: Implement the core executor for `COPY FROM` using CSV format, ensuring memory-efficient O(1) parsing.

**Independent Test**: `tests/copy_from_test.rs` - test parsing a basic CSV file and verifying the inserted rows.

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T010 [P] [US1] Create a basic CSV integration test in `tests/copy_from_test.rs`. Include tests for malformed data and constraint checks.

### Implementation for User Story 1

- [x] T011 [US1] Create `src/executor/copy.rs` and implement the base `execute_copy` dispatch method.
- [x] T012 [US1] Implement `copy_from_csv` in `src/executor/copy.rs` utilizing the `csv` crate.
- [x] T013 [US1] Implement the `parse_field` helper in `src/executor/copy.rs` to convert strings directly to Oxibase `Value`s without allocations where possible.
- [x] T014 [US1] Wire up `Statement::Copy` inside `src/executor/query.rs` to call `execute_copy`.
- [x] T015 [US1] Add file to the module tree in `src/executor/mod.rs` (`pub mod copy;`).
- [x] T016 [US1] Ensure the `COPY` transaction commits successfully and rolls back on errors or constraint checks (`validate_check_constraint` & `check_parent_exists`).
- [x] T017 [US1] Run `make test` to verify passing the CSV integration tests.

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently.

---

## Phase 4: User Story 2 - Bulk Loading Data from JSON (Priority: P1)

**Goal**: Extend the executor to handle JSON (arrays and lines) efficiently using the custom `JsonArrayStripper`.

**Independent Test**: `tests/copy_from_test.rs` - test parsing JSON Arrays and JSON Lines.

### Tests for User Story 2 ⚠️

- [x] T020 [P] [US2] Add JSON test cases to `tests/copy_from_test.rs` covering both JSON Arrays and JSON Lines formats. Include vector dimensionality tests.

### Implementation for User Story 2

- [x] T021 [P] [US2] Implement the `JsonArrayStripper` stream reader adapter in `src/executor/copy.rs`.
- [x] T022 [P] [US2] Implement the `json_value_to_stoolap` coercion helper in `src/executor/copy.rs`.
- [x] T023 [US2] Implement `copy_from_json` and `insert_json_row` in `src/executor/copy.rs`.
- [x] T024 [US2] Ensure vector dimension bounds are enforced (`validate_vector_dims`) within JSON ingestion.
- [x] T025 [US2] Run `make test` to verify the JSON integration tests pass.

---

## Phase 5: User Story 3 - Selective Column Ingestion (Priority: P2)

**Goal**: Support specifying column lists in the `COPY` statement to handle data files that don't strictly match the table schema.

**Independent Test**: `tests/copy_from_test.rs` - test `COPY users (id, name) FROM 'data.csv'` and JSON key-mapping behavior.

### Tests for User Story 3 ⚠️

- [x] T030 [P] [US3] Add tests for selective columns (CSV) and extra JSON keys to `tests/copy_from_test.rs`. Verify default values are correctly assigned to missing columns.

### Implementation for User Story 3

- [x] T031 [US3] Implement `build_default_row` helper in `src/executor/copy.rs` to properly evaluate schema defaults for missing columns.
- [x] T032 [US3] Update CSV logic to map headers to selective columns if specified.
- [x] T033 [US3] Update JSON logic to map specific JSON keys case-insensitively and ignore extra keys when column lists are provided.
- [x] T034 [US3] Run `make test` to verify all selective column ingestion scenarios.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories, ensuring constitution compliance.

- [x] T040 Verify semantic and subquery caches are correctly invalidated inside `execute_copy` upon a successful commit.
- [x] T041 Verify transaction isolation (ensuring `COPY` throws an error if called within an active transaction).
- [x] T042 Verify `unwrap()` and `expect()` are not used inappropriately in the new code.
- [x] T043 Add missing Apache-2.0 headers (can run `./scripts/fix_copyrights.sh`).
- [x] T044 Run `make lint` and fix any warnings (`cargo fmt` and `cargo clippy`).
- [x] T045 Run `make license` to ensure compliance.