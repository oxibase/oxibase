# Implementation Tasks: DAP Support for PL/SQL Procedures

**Branch**: `039-dap-plsql` | **Date**: 2026-06-11
**Input**: Design documents from `/specs/039-dap-plsql/`
**Prerequisites**: plan.md, spec.md, data-model.md

## Dependencies & Execution Order

- **Phase 1** must be completed first (Setup & AST changes).
- **Phase 2 (US1)** depends on Phase 1 (Core hooking and pausing).
- **Phase 3 (US2)** depends on Phase 2 (Environment mapping requires pause context).
- **Phase 4 (US3)** depends on Phase 2 (Stepping requires pause context).

*Parallel Execution Example*:
- Bob works on T005 (Parser modifications).
- Alice works on T006 (AST line number tracking) simultaneously.

## Phase 1: Setup & Foundational AST Modifications

**Goal**: Update the PL/SQL AST to retain source location (line number) metadata, so the debugger knows where execution is.

- [ ] T001 Define `Spanned<T>` or inject `line_number: usize` into AST nodes (e.g. `AssignmentStatement`, `IfStatement`, `WhileStatement`, `Sql`, `Return`) in `src/functions/plsql/ast.rs`.
- [ ] T002 Modify `PlSqlParser` methods to record line numbers when parsing statements in `src/functions/plsql/parser.rs`.
- [ ] T003 Fix any compiler errors arising from AST changes across the PL/SQL execution engine (`src/functions/plsql/interpreter.rs`, `src/functions/plsql/backend.rs`).
- [ ] T004 Run `make test` to ensure PL/SQL logic is unaffected by the AST structural changes.

## Phase 2: Attach and Pause at Breakpoint (US1 - Priority P1)

**Goal**: As a developer, I can connect via DAP, set a breakpoint, and execution pauses precisely before evaluating the statement at that line.

**Independent Test**: Integration test in `tests/procedure_plsql_tests.rs` or a new debug integration test simulating DAP `setBreakpoints` and receiving a `stopped` event.

- [ ] T005 [P] [US1] Define `DebugAdapterHook` trait (or use existing `DebugController` traits) in `src/functions/plsql/interpreter.rs` (or a dedicated debug module).
- [ ] T006 [P] [US1] Add `debug_hook: Option<Arc<dyn DebugAdapterHook>>` to `PlSqlInterpreter` in `src/functions/plsql/interpreter.rs`.
- [ ] T007 [US1] Update `PlSqlInterpreter::evaluate_statement` in `src/functions/plsql/interpreter.rs` to invoke `debug_hook.on_statement_before_eval(line_number, env)` before executing the inner logic.
- [ ] T008 [US1] Implement integration test in `tests/procedure_plsql_tests.rs` simulating a breakpoint hit.

## Phase 3: Inspect Local Variables and State (US2 - Priority P2)

**Goal**: As a developer debugging a paused procedure, I can inspect local variables and arguments via DAP requests.

**Independent Test**: Integration test hitting a breakpoint, executing a mock `variables` request, and validating the returned environment data.

- [ ] T009 [P] [US2] Implement `env_to_dap_scopes(env: &Environment)` (or equivalent method) to convert PL/SQL `Environment` variables to DAP variable standard structures in `src/functions/plsql/env.rs`.
- [ ] T010 [US2] Wire the variable retrieval logic to the `DebugAdapterHook` implementation, allowing the DAP server to query state while the interpreter thread is blocked.
- [ ] T011 [US2] Implement integration test verifying variables are correctly exposed during a paused state.

## Phase 4: Step Through Execution (US3 - Priority P3)

**Goal**: As a developer, I can "Step Over" and "Continue" through my PL/SQL procedure.

**Independent Test**: Integration test issuing `next` and `continue` commands to the `DebugAdapterHook` and verifying execution resumes correctly.

- [ ] T012 [US3] Implement logic within the `DebugAdapterHook` / DAP Server handler to process `next` (Step Over) by setting a temporary step breakpoint for the next line, then unblocking the interpreter thread.
- [ ] T013 [US3] Implement logic to process `continue` by clearing step state and unblocking the interpreter thread.
- [ ] T014 [US3] Implement integration test verifying step and continue execution flows.

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T015 Run `make lint` to format code and fix clippy warnings.
- [ ] T016 Run `make license` to ensure Apache-2.0 headers are on all new/modified files.
- [ ] T017 Run `make test-all` to ensure all cross-feature compilations (js, python, etc.) pass without issue.
