---
description: "Task list for PL/SQL Scalar Functions feature implementation"
---

# Tasks: PL/SQL Scalar Functions

**Input**: Design documents from `/specs/021-plsql-scalar-functions/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/plsql-ast.md

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Project initialization and basic structure

- [ ] T001 Verify project compiles (`cargo build`) before starting
- [ ] T002 Run existing tests (`make test`) to ensure a clean baseline

---

## Phase 2: Foundational

**Purpose**: Foundational AST changes required by all features.

- [ ] T010 Update `PlSqlStatement::Return` in `src/functions/plsql/ast.rs` to accept `Option<Expression>` alongside the `Token`.
- [ ] T011 Update `ExecutionStatus` in `src/functions/plsql/interpreter.rs` to include `Return(Option<Value>)`.

---

## Phase 3: User Story 1 - Define and Execute a PL/SQL Scalar Function (Priority: P1) 🎯 MVP

**Goal**: Users can write inline scalar functions using `LANGUAGE plsql` or `LANGUAGE sql` without having to rely on external scripting engines.

**Independent Test**: Integration test in `tests/plsql_functions.rs`

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T020 [US1] Create failing integration test in `tests/plsql_functions.rs` evaluating a basic PL/SQL scalar function using `RETURN <expr>;`.
- [ ] T021 [US1] Create failing integration test in `tests/plsql_functions.rs` verifying a PL/SQL function with control flow (e.g. IF/ELSE) and multiple `RETURN` paths.

### Implementation for User Story 1

- [ ] T022 [US1] Modify parser in `src/functions/plsql/parser.rs` to parse an optional expression after `RETURN` keyword and before the `;` token.
- [ ] T023 [US1] Modify interpreter `execute()` logic in `src/functions/plsql/interpreter.rs` to evaluate the return expression and bubble up `ExecutionStatus::Return(Some(val))`.
- [ ] T024 [US1] Implement `execute` method in `src/functions/plsql/backend.rs` to: parse the code, setup environment with arguments, run the interpreter, and extract the returned `Value`.
- [ ] T025 [US1] Verify that `tests/plsql_functions.rs` integration tests pass successfully with `make test`.

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and code quality.

- [ ] T030 [P] Run `make lint` and fix any formatting/clippy warnings introduced in the PL/SQL module.
- [ ] T031 [P] Verify `make license` passes to ensure all `.rs` files have the correct Apache-2.0 copyright header.
- [ ] T032 Verify `unwrap()` and `expect()` are not used inappropriately in the new parsing and interpretation logic.
