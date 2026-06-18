# Tasks: Scripting Stdout Interception

**Feature Branch**: `043-stdout-interception`  
**Plan**: [plan.md](plan.md)  
**Spec**: [spec.md](spec.md)

## Strategy

We will deliver this feature in two main increments matching the prioritized user stories:
1. **US1 (Rhai Hook)**: A straightforward hook registration in the Rhai engine initialization, allowing developers to immediately debug Rhai scripts.
2. **US2 (PL/SQL Support)**: A more involved task requiring extending the parser (AST and token handling) and the interpreter, allowing standard database logic to be traced natively.

---

## Phase 1: Setup

*(No project initialization setup tasks required as changes are contained to existing files.)*

---

## Phase 2: Foundational

*(No foundational data model changes outside the specific stories.)*

---

## Phase 3: User Story 1 - Rhai Script Output Capture (Priority: P1) 🎯 MVP

**Goal**: As a database developer writing Rhai scripts, I want my script's `print()` statements to be captured and appended to the execution logs.

**Independent Test**: Can be fully tested by integration tests executing a script with `print("hello")` and asserting the resulting execution context contains "hello" in its stdout buffer.

- [x] T001 [US1] Add `engine.on_print` hook in `RhaiBackend::new` to forward output to `crate::functions::context::append_stdout` in `src/functions/backends/rhai.rs`
- [x] T002 [US1] Write integration test to verify a Rhai script with `print("test");` correctly populates the execution context's stdout buffer in `tests/procedure_rhai_tests.rs` or equivalent.

---

## Phase 4: User Story 2 - PL/SQL Output Capture (Priority: P1)

**Goal**: As a database user writing PL/SQL procedures, I want to use `PRINT` or `RAISE NOTICE` to output messages so that I can trace execution flow and debug my procedures.

**Independent Test**: Can be fully tested by parsing and executing PL/SQL blocks containing `PRINT` and `RAISE NOTICE`, and checking the resulting execution context logs.

- [x] T003 [US2] Add `Print(Token, Expression)` variant to the `PlSqlStatement` enum in `src/functions/plsql/ast.rs`
- [x] T004 [US2] Add logic to parse `PRINT` keyword and an expression, returning `PlSqlStatement::Print` in `src/functions/plsql/parser.rs`
- [x] T005 [US2] Add logic to parse `RAISE NOTICE` keywords and an expression, returning `PlSqlStatement::Print` in `src/functions/plsql/parser.rs`
- [x] T006 [US2] Implement evaluation for `PlSqlStatement::Print` in `PlSqlInterpreter::execute_statement` to call `crate::functions::context::append_stdout` with the evaluated expression's string representation in `src/functions/plsql/interpreter.rs`
- [x] T007 [US2] Write integration test to verify a PL/SQL script with `PRINT 'test';` and `RAISE NOTICE 'test2';` correctly populates the execution context's stdout buffer in `tests/procedure_plsql_tests.rs`

---

## Phase 5: Polish & Cross-Cutting Concerns

**Goal**: Ensure the entire system is robust, documented, and meets quality standards.

- [x] T008 Run `make lint` and fix any formatting or clippy warnings introduced by these changes.
- [x] T009 Run `make test` to ensure all existing and new tests pass.

---

## Dependencies & Execution Order

- **T001** and **T003** can be executed in parallel.
- **T004**, **T005**, and **T006** depend on **T003**.
- **T008** and **T009** must be executed last.

**Parallel Opportunities**:
- Developer A implements Rhai support (T001 - T002).
- Developer B implements PL/SQL support (T003 - T007).