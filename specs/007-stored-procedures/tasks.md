# Implementation Tasks: Stored Procedures (CREATE PROCEDURE / CALL)

**Feature Branch**: `007-stored-procedures`  
**Generated**: 2026-05-08  
**Based on**:
- spec.md
- plan.md
- data-model.md
- research.md

## Implementation Strategy

This feature will be implemented incrementally, starting with core AST and catalog components, followed by `CALL` execution with the default Rhai backend, and culminating with the native PL/pgSQL interpreter.

## Phase 1: Setup

- [ ] T001 Initialize system.procedures catalog script in `src/storage/procedures.rs` based on existing functions implementation

## Phase 2: Foundational Components (AST & Storage)

*Goal: Extend parser and internal models to represent stored procedures.*

- [ ] T002 Implement `ParameterMode` enum (In, Out, InOut) in `src/parser/ast.rs`
- [ ] T003 Implement `ProcedureParameter` struct in `src/parser/ast.rs`
- [ ] T004 Implement `CreateProcedureStatement` AST node in `src/parser/ast.rs`
- [ ] T005 Implement `CallStatement` AST node in `src/parser/ast.rs`
- [ ] T006 Implement parsing logic for `CREATE OR REPLACE PROCEDURE` in `src/parser/statements.rs`
- [ ] T007 Implement parsing logic for `CALL` in `src/parser/statements.rs`
- [ ] T008 [P] Implement `StoredProcedureParameter` and `StoredProcedure` structs in `src/storage/procedures.rs`
- [ ] T009 Add system catalog table creation SQL for `system.procedures` to `src/storage/procedures.rs`

## Phase 3: Create and Execute a Stored Procedure [US1]

*Goal: As a database user, I want to define a stored procedure with custom logic and execute it later.*
*Independent Test*: Verify procedure creation and execution with no parameters using Rhai.

- [ ] T010 [US1] Implement `execute_create_procedure` handler in `src/executor/ddl.rs`
- [ ] T011 [US1] Integrate `ScriptingBackend::validate_code` into `execute_create_procedure` before persistence
- [ ] T012 [US1] Implement procedure registry cache in `src/functions/registry.rs`
- [ ] T013 [US1] Implement `execute_call` execution loop in `src/executor/execute.rs` to invoke parameterless procedures
- [ ] T014 [US1] Write integration test for basic CREATE PROCEDURE and CALL execution in `tests/procedure_tests.rs`

## Phase 4: Procedure with Arguments [US2]

*Goal: As a database user, I want to pass arguments to a stored procedure.*
*Independent Test*: Verify passing IN arguments and retrieving OUT/INOUT values via single-row return.

- [ ] T015 [US2] Extend `ScriptingBackend` traits to handle parameter modes or introduce `ProcedureBackend` adapter in `src/functions/backends.rs`
- [ ] T016 [US2] Update `execute_call` to map `OUT` and `INOUT` values to a returned `Row` in `src/executor/execute.rs`
- [ ] T017 [US2] Update `rhai` backend adapter to handle `OUT` parameter mutations in `src/functions/backends/rhai.rs`
- [ ] T018 [US2] Write integration tests for IN, OUT, and INOUT parameter procedures in `tests/procedure_tests.rs`

## Phase 5: PL/pgSQL-like Procedural Logic [US3]

*Goal: As a database user, I want to write procedures using a native, standard PL/pgSQL-like language.*
*Independent Test*: Verify PL/pgSQL specific syntax (IF, variables) parses and executes correctly.

- [ ] T019 [US3] Create new module `src/functions/plpgsql/mod.rs`
- [ ] T020 [P] [US3] Implement PL/pgSQL AST nodes (Block, If, Assignment, etc.) in `src/functions/plpgsql/ast.rs`
- [ ] T021 [US3] Implement PL/pgSQL parser in `src/functions/plpgsql/parser.rs`
- [ ] T022 [US3] Implement PL/pgSQL execution `Environment` (stack frames, variables) in `src/functions/plpgsql/env.rs`
- [ ] T023 [US3] Implement PL/pgSQL interpreter evaluation logic in `src/functions/plpgsql/interpreter.rs`
- [ ] T024 [US3] Implement `ScriptingBackend` trait for the PL/pgSQL engine in `src/functions/plpgsql/backend.rs`
- [ ] T025 [US3] Register PL/pgSQL backend in `src/functions/backends.rs` for `LANGUAGE sql` and `LANGUAGE plpgsql`
- [ ] T026 [US3] Write integration tests for PL/pgSQL execution in `tests/procedure_plpgsql_tests.rs`

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T027 Run `make lint` and fix all warnings
- [ ] T028 Run `make license` to verify headers in new PL/pgSQL module
- [ ] T029 Execute full test suite `make test-all`
