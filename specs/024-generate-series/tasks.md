# Tasks: generate_series Table-Valued Function

**Input**: Design documents from `/specs/024-generate-series/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

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

- [ ] T001 Initialize the test file `tests/generate_series_test.rs` with necessary db setup scaffolding
- [ ] T002 Verify project compiles (`cargo build`)

---

## Phase 2: Foundational 

**Purpose**: Blocking prerequisites for all user stories. In this case, setting up the AST support for TVFs.

- [ ] T003 [P] Introduce `FunctionTableSource` structure to `src/parser/ast.rs`
- [ ] T004 Implement `parse_function_table_source` in `src/parser/statements.rs` and hook it into table parsing in the FROM clause.

---

## Phase 3: User Story 1 - Generate Series with Start and Stop (Priority: P1) 🎯 MVP

**Goal**: Users want to query a sequence of numbers starting from a given value up to a stop value with a default step of 1.

**Independent Test**: Port the basic `generate_series(start, stop)` tests from `stoolap` to `tests/generate_series_test.rs`

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T005 [US1] Create failing tests for `test_generate_series_basic` and `test_generate_series_single_value` in `tests/generate_series_test.rs`

### Implementation for User Story 1

- [ ] T006 [US1] Implement TVF iteration logic for `generate_series` in a new module like `src/functions/tvf.rs` (supporting start, stop, default step 1).
- [ ] T007 [US1] Implement `execute_tvf_source` in `src/executor/query.rs` (or similar execution file) to route execution to the iteration logic without pre-allocating a large `Vec`.
- [ ] T008 [US1] Update function registry in `src/functions/registry.rs` to recognize `generate_series`.
- [ ] T009 [US1] Run `make lint` and fix any warnings
- [ ] T010 [US1] Run `make test` to verify passing integration tests for basic start/stop.

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Generate Series with Start, Stop, and Step (Priority: P2)

**Goal**: Users want to specify a custom step value when generating a series, allowing them to increment by numbers other than 1.

**Independent Test**: Port step, descending, error, and other extended tests from `stoolap` to `tests/generate_series_test.rs`

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T011 [US2] Create failing tests `test_generate_series_with_step`, `test_generate_series_descending`, `test_generate_series_auto_descending`, and `test_generate_series_zero_step_error` in `tests/generate_series_test.rs`

### Implementation for User Story 2

- [ ] T012 [US2] Extend the TVF iteration logic in `src/functions/tvf.rs` to correctly handle custom steps, descending directions, and validation for 0 step.
- [ ] T013 [US2] Extend logical and physical plan components to correctly propagate the third argument to the iterator state.
- [ ] T014 [US2] Run `make lint` and fix any warnings
- [ ] T015 [US2] Run `make test` to verify all integration tests for complex steps and descending sequences pass.

**Checkpoint**: At this point, User Story 2 should be fully functional and testable independently

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T016 Verify `make license` passes and all new `.rs` files have proper headers.
- [ ] T017 Verify `unwrap()` and `expect()` are not used inappropriately in the new parsing and execution paths.
- [ ] T018 Code cleanup, refactoring, and verifying zero-allocation (avoiding `.clone()` for large sequences).

## Implementation Strategy & Dependencies

**Dependencies:**
- User Story 1 depends on Foundational Phase (AST parser support).
- User Story 2 depends on User Story 1 (Basic executor integration).

**Parallel Opportunities:**
- Writing AST structures (T003) and test scaffolding (T001) can happen in parallel.
- Basic test authoring (T005) can be done while foundational AST work is being implemented.