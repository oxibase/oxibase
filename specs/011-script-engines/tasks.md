# Implementation Tasks: Script Engine Event Triggers (Boa and RustPython)

## Phase 1: Foundational 
**Goal**: Verify existing thread-local states from `010-event-triggers` are public or accessible to the Python and Boa backend implementations.
- [x] T001 Ensure `CURRENT_NEW_ROW`, `CURRENT_OLD_ROW`, and `CURRENT_SCHEMA` in `src/functions/backends/triggers.rs` are properly exported for use in `boa.rs` and `python.rs`

## Phase 2: User Story 1 & 3 - Python Proxies & Triggers (BEFORE INSERT/UPDATE/DELETE)
**Goal**: Implement `PyNewRowProxy` and `PyOldRowProxy` for RustPython to support `LANGUAGE python`.
**Independent Test**: Create a `BEFORE INSERT` trigger in Python throwing an error on negative input; verify the insert aborts cleanly. Test mutation of `NEW` record values.
- [x] T002 [US1] Define `PyNewRowProxy` and `PyOldRowProxy` structs with `#[pyclass]` in `src/functions/backends/python.rs`
- [x] T003 [US1] Implement `__getattr__` and `__setattr__` via `#[pymethod(magic)]` for `PyNewRowProxy` interfacing with `CURRENT_NEW_ROW` and `CURRENT_SCHEMA`
- [x] T004 [US1] Implement `__getattr__` via `#[pymethod(magic)]` for `PyOldRowProxy` interfacing with `CURRENT_OLD_ROW` and `CURRENT_SCHEMA`
- [x] T005 [US1] Inject `PyNewRowProxy` and `PyOldRowProxy` into the global execution scope in `execute_procedure` within `src/functions/backends/python.rs`
- [x] T006 [US1] Add a Python-specific integration test in `tests/` to verify Python triggers correctly intercept inserts and update row values

## Phase 3: User Story 2 & 3 - JavaScript Proxies & Triggers (AFTER UPDATE/DELETE)
**Goal**: Implement `JsProxy` logic for Boa to support `LANGUAGE js`.
**Independent Test**: Create an `AFTER UPDATE` trigger in JS that logs data via `oxibase.execute`.
- [x] T007 [P] [US2] Create native `get` and `set` trap functions (`new_row_get`, `new_row_set`, `old_row_get`) in `src/functions/backends/boa.rs`
- [x] T008 [P] [US2] Map JS Proxy traps to `CURRENT_NEW_ROW`, `CURRENT_OLD_ROW`, and `CURRENT_SCHEMA` with appropriate `JsValue` coercion
- [x] T009 [P] [US2] Inject `JsProxy` instances for `NEW` and `OLD` into the global execution context in `execute_procedure` within `src/functions/backends/boa.rs`
- [x] T010 [US2] Add a JS-specific integration test in `tests/` to verify JS triggers can intercept DML operations and perform side effects via `oxibase.execute`

## Phase 4: Polish & Cross-Cutting Concerns
**Goal**: Ensure performance constraints and clean code integration.
- [x] T011 Run `cargo clippy --all-features` and ensure the proxy injections don't trigger lifetime or reference errors
- [x] T012 Run full test suite with `--features python,js` to verify zero-copy proxies do not cause panics under heavy DML loads
