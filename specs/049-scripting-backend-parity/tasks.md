# Tasks: Scripting Backend Parity

**Input**: Design documents from `/specs/049-scripting-backend-parity/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify the current state of tests before proceeding to ensure a baseline.

- [x] T001 Verify existing tests compile and pass via `cargo nextest run --profile ci` or `make test`.

---

## Phase 2: User Story 1 - JSON and Timestamp in PL/SQL (Priority: P1) 🎯 MVP

**Goal**: Support native declaration and assignment of `JSON` and `TIMESTAMP` variables in PL/SQL.

**Independent Test**: `tests/procedure_plsql_tests.rs`

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T002 [P] [US1] Create failing tests in `tests/procedure_plsql_tests.rs` for declaring and assigning `JSON` types.
- [x] T003 [P] [US1] Create failing tests in `tests/procedure_plsql_tests.rs` for declaring and assigning `TIMESTAMP` types.

### Implementation for User Story 1

- [x] T004 [US1] Modify `src/functions/plsql/interpreter.rs` inside `execute_statement` (specifically the `Declare` matching arm) to correctly initialize variables with data types containing "JSON" to `Value::Null(crate::core::DataType::Json)`.
- [x] T005 [US1] Modify `src/functions/plsql/interpreter.rs` inside `execute_statement` (specifically the `Declare` matching arm) to correctly initialize variables with data types containing "TIMESTAMP" to `Value::Null(crate::core::DataType::Timestamp)`.
- [x] T006 [US1] Verify that `make test` now passes the tests created in T002 and T003.

**Checkpoint**: PL/SQL can now declare and manipulate JSON and TIMESTAMP variables natively.

---

## Phase 3: User Story 2 - Random Number Generation in PL/SQL (Priority: P1)

**Goal**: Support the `random()` built-in function in PL/SQL blocks.

**Independent Test**: `tests/procedure_plsql_tests.rs`

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [P] [US2] Create failing test in `tests/procedure_plsql_tests.rs` that calls `random()` and prints the result, verifying it executes without error.

### Implementation for User Story 2

- [x] T008 [US2] Update `eval_expr` in `src/functions/plsql/interpreter.rs` under the `FunctionCall` arm. Add logic to intercept `func_name == "random"`.
- [x] T009 [US2] Inside the intercepted `random` block, use `rand::rng().random::<f64>()` (importing `rand::RngExt` locally if needed) and return `Value::Float`.
- [x] T010 [US2] Run `make test` to verify passing integration test from T007.

**Checkpoint**: PL/SQL supports `random()` generation matching Python/Rhai parity.

---

## Phase 4: User Story 3 - Python Native Type Marshalling (Priority: P2)

**Goal**: Transparently convert Oxibase `JSON` and `TIMESTAMP` Values into native Python `dict` and `datetime` objects and vice versa.

**Independent Test**: `tests/python_scripting_test.rs`

### Tests for User Story 3 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T011 [P] [US3] Create failing tests in `tests/python_scripting_test.rs` testing Python function receiving and returning a `JSON` argument mapped as a `dict`.
- [x] T012 [P] [US3] Create failing tests in `tests/python_scripting_test.rs` testing Python function receiving and returning a `TIMESTAMP` argument mapped as a `datetime`.

### Implementation for User Story 3

- [x] T013 [P] [US3] In `src/functions/backends/python.rs`, modify `convert_oxibase_to_python` to parse `Value::Json` string using `serde_json` and construct the equivalent `PyDict`/`PyList` via RustPython APIs (or evaluate a string using `json.loads` within the VM).
- [x] T014 [P] [US3] In `src/functions/backends/python.rs`, modify `convert_oxibase_to_python` to handle `Value::Timestamp` by returning a Python `datetime` object (e.g. via `datetime.fromisoformat`).
- [x] T015 [US3] In `src/functions/backends/python.rs`, modify `convert_python_to_oxibase` to check if a `PyObject` is a `dict` or `list`, and if so, serialize it back to a JSON string using Python's `json.dumps`.
- [x] T016 [US3] In `src/functions/backends/python.rs`, modify `convert_python_to_oxibase` to check if a `PyObject` is a `datetime` object, format it to ISO string via `.isoformat()`, and parse to `Value::Timestamp`.
- [x] T017 [US3] Verify `tests/python_scripting_test.rs` tests pass by running `cargo nextest run --features python python_scripting_test`.

**Checkpoint**: Python functions seamlessly handle JSON and Datetime natively without string casting.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and code hygiene.

- [x] T018 Verify `make license` passes.
- [x] T019 Verify `make lint` passes with no warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- [x] T020 Run the full suite `make test-all` to ensure no unexpected regressions in other components.
