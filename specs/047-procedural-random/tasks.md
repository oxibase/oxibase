# Tasks: Procedural Random Support

**Input**: Design documents from `/specs/047-procedural-random/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Test tasks are included as requested by the specification to ensure robust TDD implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- `src/functions/backends/` for Rhai & Python scripting engines
- `src/functions/plsql/` for PL/SQL parser and interpreter
- `tests/` for integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and validation

- [X] T001 Verify project compiles and standard test suite runs (`cargo nextest run`)

---

## Phase 2: User Story 1 - Rhai Procedural Random (Priority: P1) 🎯 MVP

**Goal**: Native random float generator in Rhai stored procedures using `oxibase::random()`.

**Independent Test**: Tested in `tests/procedure_tests.rs`

### Tests for User Story 1
- [X] T002 [P] [US1] Create a failing integration test in `tests/procedure_tests.rs` calling `oxibase::random()` and asserting range `[0.0, 1.0)`

### Implementation for User Story 1
- [X] T003 [P] [US1] Register `random` function returning `f64` on `oxibase_module` under `RhaiBackend` in `src/functions/backends/rhai.rs` using `rand::rng().random::<f64>()`
- [X] T004 [US1] Run `make lint` and fix any formatting / clippy warnings
- [X] T005 [US1] Run `cargo nextest run --test procedure_tests` to verify the passing test

**Checkpoint**: At this point, User Story 1 is fully functional and testable independently.

---

## Phase 3: User Story 2 - Python Procedural Random (Priority: P2)

**Goal**: Native random float generator in Python stored procedures using `oxibase.random()`.

**Independent Test**: Tested in `tests/procedure_multilang_tests.rs`

### Tests for User Story 2
- [X] T006 [P] [US2] Create a failing integration test in `tests/procedure_multilang_tests.rs` calling `oxibase.random()` and asserting range `[0.0, 1.0)`

### Implementation for User Story 2
- [X] T007 [P] [US2] Register `random` as a `#[pyfunction]` returning `rustpython_vm::PyObjectRef` inside `oxibase_py_module` in `src/functions/backends/python.rs`
- [X] T008 [US2] Run `make lint` and fix any formatting / clippy warnings
- [X] T009 [US2] Run `cargo nextest run --features python --test procedure_multilang_tests` to verify the passing test

**Checkpoint**: At this point, User Story 2 is fully functional and testable independently.

---

## Phase 4: User Story 3 - PL/SQL Procedural Random & Built-in Functions (Priority: P2)

**Goal**: Generic `Expression::FunctionCall` support inside PL/SQL interpreter evaluating database scalar functions, dynamically exposing `random()`.

**Independent Test**: Tested in `tests/plsql_functions.rs`

### Tests for User Story 3
- [X] T010 [P] [US3] Create a failing integration test in `tests/plsql_functions.rs` using standard PL/SQL syntax calling `random()`

### Implementation for User Story 3
- [X] T011 [P] [US3] Rename `_function_registry` to `function_registry` to make it accessible in `src/functions/plsql/interpreter.rs`
- [X] T012 [P] [US3] Add `Expression::FunctionCall` evaluation logic in `PlSqlInterpreter::eval_expr` in `src/functions/plsql/interpreter.rs` retrieving scalar functions dynamically from the database's `FunctionRegistry`
- [X] T013 [US3] Run `make lint` and fix any formatting / clippy warnings
- [X] T014 [US3] Run `cargo nextest run --test plsql_functions` to verify the passing test

**Checkpoint**: At this point, User Story 3 is fully functional and testable independently.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Overall verification, license compliance, and cleanup

- [X] T015 [P] Verify `make license` passes across all source files
- [X] T016 Run formatting and full project clippy suite `make lint`
- [X] T017 Run full test suite with features enabled `make test-all`

---

## Dependencies & Completion Order

```text
       [Phase 1: Setup]
              │
              ▼
[Phase 2: Rhai Procedural Random (US1)]
         /         \
        ▼           ▼
[Phase 3 (US2)]   [Phase 4 (US3)]
        \           /
         ▼         ▼
[Phase 5: Polish & Cross-cutting]
```

## Parallel Execution Examples

The three user stories run completely in parallel since they reside in separate modules and files:
- **Stream A (Rhai)**: `T002` -> `T003` -> `T005`
- **Stream B (Python)**: `T006` -> `T007` -> `T009`
- **Stream C (PL/SQL)**: `T010` -> `T011` -> `T012` -> `T014`

## Implementation Strategy

1. **MVP First**: Establish Rhai procedural random support first as the foundational proof-of-concept.
2. **Incremental Delivery**: Roll out Python support and generic PL/SQL function call evaluation once the Rhai MVP is verified.
3. **Cross-backend testing**: Validate thread-safety and feature flags as part of Phase 5 Polish.
