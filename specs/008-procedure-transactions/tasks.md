# Implementation Tasks: Transaction Management in Procedures

**Feature Branch**: `008-procedure-transactions`
**Generated**: 2026-05-09
**Based on**:
- plan.md
- spec.md
- research.md
- data-model.md

## Implementation Strategy

We will implement transaction management incrementally. First, we will establish the foundational `SqlRunner` interface and the ability for the `Executor` to intercept implicit vs. explicit transaction bounds. Then, we will expose this API to the scripting backends one by one, concluding with native PL/SQL syntax support.

## Phase 1: Foundational API (Executor & SqlRunner)

*Goal: Extend the internal execution interfaces to support transaction control and manage transaction contexts during `CALL` execution.*

- [ ] T001 Extend `SqlRunner` trait in `src/functions/backends.rs` with `commit()`, `rollback()`, and `begin()` returning `Result<()>`
- [ ] T002 Add `is_explicit_tx` flag or state tracking to `ActiveTransaction` or the transaction execution context in `src/executor/mod.rs`
- [ ] T003 Implement `SqlRunner` transaction methods for `Executor` in `src/executor/mod.rs` (throw error if explicit transaction, otherwise end current and start new)
- [ ] T004 Update `execute_call` in `src/executor/query.rs` to wrap procedure execution in a tracked transaction context

## Phase 2: Expose to Scripting Backends [US1] [US2]

*Goal: Make transaction management available to user-defined stored procedures written in Rhai, Python, and JavaScript.*
*Independent Test: Verify calling `commit()` or `rollback()` in a Rhai/JS/Python script correctly persists or discards prior database changes.*

- [ ] T005 [P] [US1] Inject `commit()`, `rollback()`, and `begin()` (no-op) global functions into Rhai engine in `src/functions/backends/rhai.rs` mapping to the `SqlRunner`
- [ ] T006 [P] [US1] Inject `commit()`, `rollback()`, and `begin()` global functions into Boa context in `src/functions/backends/boa.rs` mapping to the `SqlRunner`
- [ ] T007 [P] [US1] Inject `commit()`, `rollback()`, and `begin()` into the `oxibase` native module in `src/functions/backends/python.rs` mapping to the `SqlRunner`
- [ ] T008 [US1] Write integration tests for JS/Rhai/Python procedure transaction control in `tests/procedure_tests.rs`

## Phase 3: PL/SQL Syntax Support [US3]

*Goal: Support `COMMIT`, `ROLLBACK`, and `BEGIN` tokens directly in native PL/SQL block syntax.*
*Independent Test: Ensure `CALL` on a PL/SQL procedure successfully commits/rolls back when hitting those AST nodes.*

- [ ] T009 [P] [US3] Add `Commit`, `Rollback`, and `BeginTransaction` variants to `PlSqlStatement` enum in `src/functions/plsql/ast.rs`
- [ ] T010 [US3] Update `PlSqlParser::parse_statement` in `src/functions/plsql/parser.rs` to recognize `COMMIT`, `ROLLBACK`, and `BEGIN` tokens
- [ ] T011 [US3] Update `PlSqlInterpreter::evaluate_statement` in `src/functions/plsql/interpreter.rs` to delegate `Commit` and `Rollback` to `SqlRunner`, and treat `BeginTransaction` as a no-op
- [ ] T012 [US3] Write integration tests covering PL/SQL transaction blocks, including error generation on explicit nested transactions (`BEGIN; CALL proc()`) in `tests/procedure_tests.rs`

## Phase 4: Polish & Cross-Cutting Concerns

- [ ] T013 Run `make lint` to ensure no warnings or unsafe code
- [ ] T014 Run `make test-all` to ensure all cross-backend tests pass successfully
