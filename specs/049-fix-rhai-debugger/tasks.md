# Implementation Tasks: fix-rhai-debugger

**Branch**: `049-fix-rhai-debugger` | **Date**: 2026-06-20
**Input**: Design documents from `/specs/049-fix-rhai-debugger/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

## Dependencies & Execution Order

- **Phase 1** must be completed first (Setup & compilation check).
- **Phase 2 (Foundational)** defines core debugger setups.
- **Phase 3 (US1)** depends on Phase 2 (Core Rhai debugging fixes).
- **Phase 4 (US2)** depends on Phase 3 (Workstation / Chrome DevTools E2E verification).
- **Phase 5** contains general Polish & formatting (Runs after all implementation).

*Parallel Execution Example*:
- Alice works on T004 (Rhai on_step and status checks).
- Bob works on T006 (Integration test template/skeleton in tests/rhai_scripting_test.rs).

## Phase 1: Setup

**Goal**: Verify the current state of Rhai scripting tests to establish a baseline before making changes.

- [x] T001 Verify existing Rhai scripting tests pass via cargo nextest run --test rhai_scripting_test --features rhai

## Phase 2: Foundational Rhai Debugger Enablement

**Goal**: Unconditionally compile Rhai's debugger and configure its initial status so that execution always evaluates statement hooks correctly.

- [x] T002 Enable Rhai debugger unconditionally by removing the `#[cfg(debug_assertions)]` guard in `src/functions/backends/rhai.rs`
- [x] T003 Configure Rhai debugger's initial status to `DebuggerStatus::StepInto` in `src/functions/backends/rhai.rs`

## Phase 3: Pause and Resume execution (US1 - Priority P1)

**Goal**: Ensure Rhai's `on_step` hook pauses execution at breakpoints, registers local scope variables, and maps resume actions back to Rhai's execution engine.

**Independent Test**: Simulating a breakpoint pause and resume in `tests/rhai_scripting_test.rs`.

- [x] T004 [US1] Implement breakpoint lookup and thread pause handling via `DebugController` in the `on_step` callback in `src/functions/backends/rhai.rs`
- [x] T005 [US1] Map the `ResumeAction` returned by the `DebugController` back to Rhai's `DebuggerCommand` inside the debugger callback in `src/functions/backends/rhai.rs`
- [x] T006 [P] [US1] Create a comprehensive integration test `test_rhai_debugger` in `tests/rhai_scripting_test.rs`
- [x] T007 [US1] Build the database and verify clean compilation via `cargo check --all-targets --all-features`

## Phase 4: Workstation & Chrome DevTools Verification (US2 - Priority P2)

**Goal**: Verify that custom functions and procedures can be created in the workstation web UI and successfully debugged with the simulated browser console/DAP interface.

**Independent Test**: Use workstation API and Chrome DevTools simulation to create, breakpoint, and step-through scripts.

- [x] T008 [P] [US2] Start the Oxibase server with `server` feature enabled in the background and establish a WebSocket connection via DAP handler
- [x] T009 [US2] Verify creating and debugging a custom Rhai function and procedure from the web workstation UI using Chrome DevTools interactions

## Phase 5: Polish & Cross-Cutting Concerns

- [x] T010 Run `make lint` to format code and fix clippy warnings
- [x] T011 Run `make license` to ensure Apache-2.0 headers are on all new/modified files
- [x] T012 Run `make test-all` to ensure all cross-feature compilations pass without issue
