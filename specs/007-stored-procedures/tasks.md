# Implementation Tasks: Stored Procedures (CREATE PROCEDURE / CALL)

**Feature Branch**: `007-stored-procedures`  
**Generated**: 2026-05-08  
**Based on**:
- spec.md
- plan.md
- data-model.md
- research.md

## Implementation Strategy

This feature will be implemented incrementally, starting with core AST and catalog components, followed by `CALL` execution with the default Rhai backend, and culminating with the native PL/SQL interpreter.

## Phase 1: Setup

- [x] T001 Initialize system.procedures catalog script in `src/storage/procedures.rs` based on existing functions implementation

## Phase 2: Foundational Components (AST & Storage)

*Goal: Extend parser and internal models to represent stored procedures.*

- [x] T002 Implement `ParameterMode` enum (In, Out, InOut) in `src/parser/ast.rs`
- [x] T003 Implement `ProcedureParameter` struct in `src/parser/ast.rs`
- [x] T004 Implement `CreateProcedureStatement` AST node in `src/parser/ast.rs`
- [x] T005 Implement `CallStatement` AST node in `src/parser/ast.rs`
- [x] T006 Implement parsing logic for `CREATE OR REPLACE PROCEDURE` in `src/parser/statements.rs`
- [x] T007 Implement parsing logic for `CALL` in `src/parser/statements.rs`
- [x] T008 [P] Implement `StoredProcedureParameter` and `StoredProcedure` structs in `src/storage/procedures.rs`
- [x] T009 Add system catalog table creation SQL for `system.procedures` to `src/storage/procedures.rs`

## Phase 3: Create and Execute a Stored Procedure [US1]

*Goal: As a database user, I want to define a stored procedure with custom logic and execute it later.*
*Independent Test*: Verify procedure creation and execution with no parameters using Rhai.

- [x] T010 [US1] Implement `execute_create_procedure` handler in `src/executor/ddl.rs`
- [x] T011 [US1] Integrate `ScriptingBackend::validate_code` into `execute_create_procedure` before persistence
- [x] T012 [US1] Implement procedure registry cache in `src/functions/registry.rs`
- [x] T013 [US1] Implement `execute_call` execution loop in `src/executor/execute.rs` to invoke parameterless procedures
- [x] T014 [US1] Write integration test for basic CREATE PROCEDURE and CALL execution in `tests/procedure_tests.rs`

## Phase 4: Procedure with Arguments [US2]

*Goal: As a database user, I want to pass arguments to a stored procedure.*
*Independent Test*: Verify passing IN arguments and retrieving OUT/INOUT values via single-row return.

- [x] T015 [US2] Extend `ScriptingBackend` traits to handle parameter modes or introduce `ProcedureBackend` adapter in `src/functions/backends.rs`
- [x] T016 [US2] Update `execute_call` to map `OUT` and `INOUT` values to a returned `Row` in `src/executor/execute.rs`
- [x] T017 [US2] Update `rhai` backend adapter to handle `OUT` parameter mutations in `src/functions/backends/rhai.rs`
- [x] T018 [US2] Write integration tests for IN, OUT, and INOUT parameter procedures in `tests/procedure_tests.rs`

## Phase 5: PL/SQL-like Procedural Logic [US3]

*Goal: As a database user, I want to write procedures using a native, standard PL/SQL-like language.*
*Independent Test*: Verify PL/SQL specific syntax (IF, variables) parses and executes correctly.

- [x] T019 [US3] Create new module `src/functions/plsql/mod.rs`
- [x] T020 [P] [US3] Implement PL/SQL AST nodes (Block, If, Assignment, etc.) in `src/functions/plsql/ast.rs`
- [x] T021 [US3] Implement PL/SQL parser in `src/functions/plsql/parser.rs`
- [x] T022 [US3] Implement PL/SQL execution `Environment` (stack frames, variables) in `src/functions/plsql/env.rs`
- [x] T023 [US3] Implement PL/SQL interpreter evaluation logic in `src/functions/plsql/interpreter.rs`
- [x] T024 [US3] Implement `ScriptingBackend` trait for the PL/SQL engine in `src/functions/plsql/backend.rs`
- [x] T025 [US3] Register PL/SQL backend in `src/functions/backends.rs` for `LANGUAGE sql` and `LANGUAGE pl/sql`
- [x] T026 [US3] Write integration tests for PL/SQL execution in `tests/procedure_plsql_tests.rs`

## Phase 6: Additional Scripting Backends (JS & Python) [US4]

*Goal: As a database user, I want to write procedures using Javascript and Python backends.*
*Independent Test*: Verify that procedures can be executed with `LANGUAGE js` and `LANGUAGE python`.

- [ ] T027 [US4] Implement procedure execution logic for `LANGUAGE js` using the `boa` backend adapter in `src/functions/backends/boa.rs`
- [ ] T028 [US4] Implement procedure execution logic for `LANGUAGE python` using the `rustpython` backend adapter in `src/functions/backends/python.rs`
- [ ] T029 [US4] Write integration tests for JS and Python execution in `tests/procedure_multilang_tests.rs`

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T030 Run `make lint` and fix all warnings
- [ ] T031 Run `make license` to verify headers in new PL/SQL module
- [ ] T032 Execute full test suite `make test-all`
