# Implementation Tasks: Event Triggers (BEFORE/AFTER INSERT, UPDATE, DELETE)

## Phase 1: Setup & Data Models
**Goal**: Establish the AST representation and storage catalog required for triggers.
- [x] T001 Define `TriggerTiming` and `TriggerEvent` enums in `src/parser/ast.rs`
- [x] T002 Define `CreateTriggerStatement` and `DropTriggerStatement` AST nodes in `src/parser/ast.rs`
- [x] T003 Implement `CREATE TRIGGER` parsing logic in `src/parser/statements.rs`
- [x] T004 Implement `DROP TRIGGER` parsing logic in `src/parser/statements.rs`
- [x] T005 Create `_sys_triggers` catalog table schema and bootstrap logic in `src/storage/triggers.rs` (or equivalent catalog bootstrapper)
- [x] T006 Implement DDL executor for `CREATE TRIGGER` (inserting into `_sys_triggers`) in `src/executor/ddl.rs`
- [x] T007 Implement DDL executor for `DROP TRIGGER` (deleting from `_sys_triggers`) in `src/executor/ddl.rs`
- [ ] T008 Add unit tests for `CREATE/DROP TRIGGER` parsing and execution in `tests/`

## Phase 2: Foundational (Trigger Registry & Context)
**Goal**: Build the in-memory cache and safe thread-local mechanisms for exposing context without allocating.
- [x] T009 [P] Create the `TriggerRegistry` structure to cache active triggers indexed by `table_name` in `src/executor/triggers.rs`
- [x] T010 [P] Implement loading `TriggerRegistry` from `_sys_triggers` upon database startup in `src/executor/mod.rs`
- [x] T011 [P] Implement `TriggerRegistry` invalidation/updates during `CREATE/DROP TRIGGER` in `src/executor/ddl.rs`
- [x] T012 Define thread-locals (`CURRENT_NEW_ROW`, `CURRENT_OLD_ROW`, `CURRENT_SCHEMA`) in `src/functions/backends/triggers.rs` (or similar new module)
- [x] T013 Implement the proxy object `NewRowProxy`/`OldRowProxy` logic for Rhai backend using `register_getter_fallback` and `register_setter_fallback` in `src/functions/backends/rhai.rs`

## Phase 3: User Story 1 - Validation Trigger (BEFORE INSERT/UPDATE)
**Goal**: Enable pre-validation. If logic fails, abort the DML operation.
**Independent Test**: Create a `BEFORE INSERT` trigger throwing an error on negative input; verify the insert aborts cleanly.
- [x] T014 [US1] Inject `TriggerRegistry` into `src/executor/dml.rs` (`insert` and `update` pipelines)
- [x] T015 [US1] Implement `execute_before_triggers` hook in `src/executor/dml.rs` that looks up relevant triggers
- [x] T016 [US1] Wrap trigger execution in the thread-local context setter/clearer (`with_trigger_context`) inside `src/functions/backends/mod.rs`
- [x] T017 [US1] Ensure exceptions/errors returned from the script engine are mapped to database execution errors and bubble up to abort the transaction in `src/executor/dml.rs`
- [x] T018 [US1] Write an integration test for User Story 1 (Validation aborts insert) in `tests/`

## Phase 4: User Story 2 - Audit Logging (AFTER UPDATE/DELETE)
**Goal**: Enable side-effects tracking changes.
**Independent Test**: Create an `AFTER UPDATE` trigger that calls `execute("INSERT INTO audit...")`; verify records are created.
- [x] T019 [P] [US2] Implement `execute_after_triggers` hook in `src/executor/dml.rs` (`update` and `delete` pipelines)
- [x] T020 [P] [US2] Pass `OLD` row references safely into the thread-local context for `UPDATE` and `DELETE` events in `src/executor/dml.rs`
- [x] T021 [P] [US2] Implement a recursion depth counter (e.g., in a thread-local or execution context state) to prevent infinite loops (FR-006)
- [x] T022 [US2] Write an integration test for User Story 2 (Audit logging) in `tests/`

## Phase 5: User Story 3 - Data Transformation (BEFORE INSERT/UPDATE)
**Goal**: Enable modification of the `NEW` row prior to persistence.
**Independent Test**: Create a trigger that alters `NEW.column_a`; verify the altered value is saved.
- [x] T023 [US3] Ensure the proxy objects (`register_setter_fallback` in Rhai) correctly map mutations back to the underlying `&mut [Value]` in `src/functions/backends/rhai.rs`
- [x] T024 [US3] Write an integration test for User Story 3 (Data Transformation) in `tests/`

## Phase 6: Polish & Edge Cases
**Goal**: Address schema dependencies and performance.
- [x] T025 Implement `DROP TABLE` cascading behavior: when a table is dropped, also remove its triggers from `_sys_triggers` and `TriggerRegistry` in `src/executor/ddl.rs`
- [x] T026 Add benchmarks to verify that tables without triggers suffer `< 5%` performance regression in the DML pipeline
