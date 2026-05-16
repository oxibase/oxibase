# Implementation Tasks: Drop Procedure

**Goal**: Implementation of the `DROP PROCEDURE` SQL statement, allowing users to delete existing stored procedures from the database, optionally supporting `IF EXISTS` to ignore non-existent procedures silently.

## Phase 1: Setup
*(No setup tasks needed as this builds directly on existing AST/Parser/Executor infrastructure.)*

## Phase 2: Foundational
*(No foundational dependencies required before US1.)*

## Phase 3: Basic Deletion [US1]
**Story Goal**: Users can successfully delete procedures using standard SQL syntax.
**Independent Test Criteria**: Creating a procedure, then executing `DROP PROCEDURE my_proc` successfully removes the procedure, causing subsequent calls to fail. Dropping a non-existent procedure should throw an error.

- [x] T001 [P] [US1] Add `DropProcedureStatement` struct to AST in `src/parser/ast.rs`
- [x] T002 [P] [US1] Add `DropProcedure(DropProcedureStatement)` to `Statement` enum in `src/parser/ast.rs`
- [x] T003 [US1] Implement parsing for `DROP PROCEDURE` (handling `IF EXISTS`) in `src/parser/statements.rs`
- [x] T004 [US1] Add parser routing logic for `DROP PROCEDURE` in `parse_drop_statement` in `src/parser/statements.rs`
- [x] T005 [US1] Implement AST Display traits for `DropProcedureStatement` in `src/parser/ast.rs`
- [x] T006 [P] [US1] Add `execute_drop_procedure` execution logic in `src/executor/ddl.rs`
- [x] T007 [US1] Route `Statement::DropProcedure` to `execute_drop_procedure` in `src/executor/mod.rs`
- [x] T008 [US1] Add integration tests in `tests/procedure_tests.rs` (or similar) verifying CREATE, DROP, and CALL behaviors.

## Phase 4: Polish
- [x] T009 Ensure clippy (`make lint`) passes with new AST and Executor code
- [x] T010 Verify `make test` runs green for the new procedure tests

---

## Dependencies
- **Phase 3 [US1]**: Basic Deletion depends on Phase 1 & 2 (Empty)

## Implementation Strategy
Start by implementing the AST modifications (T001, T002, T005). Once the AST supports the statement, move to the parser (T003, T004). Finally, connect the parsed statement to the execution engine (T006, T007) and add integration tests (T008) to prove it works as expected. All tasks are essentially part of a single flow, but T001/T002 and T006 can be initiated in parallel conceptually before bringing them together.