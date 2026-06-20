# Tasks: PL/SQL Table Type and FOR Loop Iteration

**Input**: Design documents from `/specs/050-plsql-table-iteration/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: Test tasks are included as requested by the specification to ensure robust TDD implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- `src/functions/backends/` for Rhai & Python scripting engines
- `src/functions/plsql/` for PL/SQL parser and interpreter
- `src/functions/scalar/` for SQL built-in utility functions
- `tests/` for integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and validation

- [X] T001 Verify project compiles and standard test suite runs (`cargo nextest run`)

---

## Phase 2: User Story 1 - Executing SQL and Storing Results in Procedural Languages (Priority: P1) 🎯 MVP

**Goal**: Expose unified querying APIs across Rhai (`oxibase::query`), Python (`oxibase.query`), and built-in database functions (`QUERY_VALUE` & `QUERY_ROWS`) to allow storing query results into variables.

**Independent Test**: Tested in `tests/rhai_scripting_test.rs` and `tests/procedure_multilang_tests.rs`.

### Tests for User Story 1
- [X] T002 [P] [US1] Create a failing integration test in `tests/rhai_scripting_test.rs` calling `oxibase::query("SELECT 100 as val;")` and asserting returned mapping is `100`.
- [X] T003 [P] [US1] Create a failing integration test in `tests/procedure_multilang_tests.rs` calling `oxibase.query("SELECT 200 as val;")` and asserting returned mapping is `200`.

### Implementation for User Story 1
- [X] T004 [P] [US1] Register `query` returning `rhai::Array` of `rhai::Map` on `oxibase_module` under `RhaiBackend` in `src/functions/backends/rhai.rs` using `execute_sql_query`.
- [X] T005 [P] [US1] Register `query` returning `rustpython_vm::PyObjectRef` containing list of dicts on `oxibase_py_module` under `PythonBackend` in `src/functions/backends/python.rs` using `execute_sql_query`.
- [X] T006 [P] [US1] Implement `QUERY_VALUE` scalar function in `src/functions/scalar/utility.rs` executing a query via `execute_sql_query` and returning the first column of the first row.
- [X] T007 [P] [US1] Implement `QUERY_ROWS` scalar function in `src/functions/scalar/utility.rs` executing a query via `execute_sql_query` and returning all rows serialized as a JSON array.
- [X] T008 [P] [US1] Register `QueryValueFunction` and `QueryRowsFunction` in `src/functions/registry.rs`.
- [X] T009 [US1] Run `make lint` and fix any formatting / clippy warnings.
- [X] T010 [US1] Run `cargo nextest run --test rhai_scripting_test` and `cargo nextest run --features python --test procedure_multilang_tests` to verify passing tests.

**Checkpoint**: At this point, User Story 1 is fully functional and testable independently.

---

## Phase 3: User Story 2 - PL/SQL `TABLE` Type Declaration (Priority: P1)

**Goal**: Support a sugar-syntax type named `TABLE` in PL/SQL variable declarations, which is aliased to `JSON` under the hood.

**Independent Test**: Tested in `tests/plsql_functions.rs`.

### Tests for User Story 2
- [X] T011 [P] [US2] Create a failing integration test in `tests/plsql_functions.rs` declaring a variable of type `TABLE` (e.g., `DECLARE v_rows TABLE; BEGIN RETURN 1; END;`).

### Implementation for User Story 2
- [X] T012 [P] [US2] Modify variable declaration parsing in `PlSqlParser::parse_variable_declaration` inside `src/functions/plsql/parser.rs` to recognize `"TABLE"` as a valid type name and alias it as `"JSON"`.
- [X] T013 [US2] Run `cargo nextest run --test plsql_functions` to verify that variable declarations compiling with type `TABLE` succeeds.

**Checkpoint**: At this point, User Story 2 is fully functional and testable independently.

---

## Phase 4: User Story 3 - PL/SQL `FOR ... IN ... LOOP` Iteration (Priority: P1)

**Goal**: Support native `FOR variable IN table_expr LOOP ... END LOOP;` statement iteration, as well as dot-notation variable access and compound field assignments.

**Independent Test**: Tested in `tests/plsql_functions.rs`.

### Tests for User Story 3
- [X] T014 [P] [US3] Create a failing integration test in `tests/plsql_functions.rs` executing a loop `FOR row_var IN v_rows LOOP` and validating dot-notation read/write properties.

### Implementation for User Story 3
- [X] T015 [P] [US3] Add `PlSqlStatement::ForLoop` variant and `ForLoopStatement` struct in `src/functions/plsql/ast.rs`.
- [X] T016 [P] [US3] Implement parser logic for the new `FOR` loop statement in `PlSqlParser::parse_statement` and sub-helper `parse_for_loop_statement` in `src/functions/plsql/parser.rs`.
- [X] T017 [P] [US3] Implement interpreter execution logic for `PlSqlStatement::ForLoop` in `PlSqlInterpreter::execute_statement` inside `src/functions/plsql/interpreter.rs`.
- [X] T018 [P] [US3] Add dot-notation field assignment update resolution (e.g., `row.age := ...`) in `PlSqlStatement::Assignment` evaluation inside `src/functions/plsql/interpreter.rs`.
- [X] T019 [P] [US3] Add dot-notation field read resolution in `Expression::QualifiedIdentifier` evaluation inside `src/functions/plsql/interpreter.rs`.
- [X] T020 [US3] Run `make lint` and fix any formatting / clippy warnings.
- [X] T021 [US3] Run `cargo nextest run --test plsql_functions` to verify passing tests.

**Checkpoint**: At this point, User Story 3 is fully functional and testable independently.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Overall verification, license compliance, and cleanup

- [X] T022 [P] Verify `make license` passes across all source files
- [X] T023 Run formatting and full project clippy suite `make lint`
- [X] T024 Run full test suite with features enabled `make test-all`

---

## Dependencies & Completion Order

```text
       [Phase 1: Setup]
              │
              ▼
   [Phase 2: User Story 1]  ◄─── (Rhai/Python/PLSQL Query Engine APIs)
              │
              ▼
   [Phase 3: User Story 2]  ◄─── (PL/SQL TABLE declaration sugar)
              │
              ▼
   [Phase 4: User Story 3]  ◄─── (PL/SQL FOR loop statement and dot-notation)
              │
              ▼
    [Phase 5: Polish]
```

---

## Parallel Execution Opportunities

- **T002, T003, T004, T005**: Query integration and backend bindings can be worked on concurrently as they target different files (`rhai_scripting_test.rs` vs `procedure_multilang_tests.rs` and `backends/rhai.rs` vs `backends/python.rs`).
- **T006, T007**: Developing scalar database functions is fully isolated within `scalar/utility.rs` and can run parallel to scripting bindings.
- **T012**: Modifying variable declaration parser is isolated to `plsql/parser.rs`.
- **T015, T016, T017**: Can be prepared in parallel with AST definitions before tying them together in parser and interpreter.
