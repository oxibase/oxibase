# Implementation Tasks: Formal Stored Procedure Logging API

**Branch**: `046-stored-procedure-logging`  
**Date**: June 19, 2026  
**Spec**: `specs/046-stored-procedure-logging/spec.md`  

## Phase 1: Setup

- [ ] T001 Implement the central, thread-safe dynamic `log_message` helper in `src/common/logging.rs`

## Phase 2: Foundational

- [ ] T002 Verify compilation baseline and cargo check passes across all targets

## Phase 3: User Story 1 - Scripting Engine Logging (Rhai/Python)

**Goal**: Expose a formal `oxibase::log(level, msg)` or `oxibase.log(level, msg)` built-in function to Rhai and Python scripting engines to safely log procedure runtimes.

**Independent Test**: Running target tests `tests/rhai_scripting_test.rs` and `tests/python_scripting_test.rs`.

**Tasks**:
- [ ] T003 [P] [US1] Register `oxibase::log` native function in RhaiBackend in `src/functions/backends/rhai.rs`
- [ ] T004 [P] [US1] Register `log` function under the Python `oxibase` module in `src/functions/backends/python.rs`
- [ ] T005 [US1] Add Rhai logging integration tests to `tests/rhai_scripting_test.rs`
- [ ] T006 [US1] Add Python logging integration tests to `tests/python_scripting_test.rs`

---

## Phase 4: User Story 2 - PL/SQL native `LOG` keyword

**Goal**: Extend PL/SQL syntax with a native `LOG <level>, <expression>;` statement, evaluate and route its execution through the central log flusher.

**Independent Test**: Running target tests `tests/procedure_plsql_tests.rs`.

**Tasks**:
- [ ] T007 [P] [US2] Update the PL/SQL AST Statement enum with the `Log` statement variant in `src/functions/plsql/ast.rs`
- [ ] T008 [P] [US2] Extend PL/SQL Statement parser to support the `LOG` statement keyword in `src/functions/plsql/parser.rs`
- [ ] T009 [P] [US2] Implement execution support for PL/SQL `Log` statement node in the interpreter in `src/functions/plsql/interpreter.rs`
- [ ] T010 [US2] Add PL/SQL logging integration tests to `tests/procedure_plsql_tests.rs`

---

## Phase 5: User Story 3 - System Table Namespace Protection

**Goal**: Remove the direct DML `system.logs` workaround check, locking down the namespace. Update SQL template file.

**Independent Test**: Query verification and namespace protection checks in `tests/internal_logging_test.rs`.

**Tasks**:
- [ ] T011 [US3] Revert the temporary `system.logs` insert whitelist exception in `src/executor/dml.rs`
- [ ] T012 [US3] Update the `pizza_demo` workflow script template to use the formal `oxibase::log` API instead of direct insert SQL in `src/bin/workspace/templates/pizza_demo.sql`
- [ ] T013 [US3] Add direct SQL insertion blockage test on `system.logs` in `tests/internal_logging_test.rs`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Tasks**:
- [ ] T014 Run `make lint` and solve formatting/clippy issues across modified files
- [ ] T015 Run the complete database test suite via `cargo nextest run` to ensure all 3000+ tests pass cleanly

---

## Dependencies

- Phase 1 must be completed before starting Phase 3 and Phase 4.
- Phase 3, 4, and 5 can be worked on in parallel once Phase 1 is done.
- Phase 6 (Polish) is done last to clean up compilation or linting warnings.
