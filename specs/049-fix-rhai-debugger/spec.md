# Feature Specification: fix-rhai-debugger

**Feature Branch**: `049-fix-rhai-debugger`
**Created**: 2026-06-20
**Status**: Draft
**Input**: User description: "Can you audit the debuggin capabilities of Rhai, i'm unable to make the execution pause at the breakpoint"

## Clarifications

### Session 2026-06-20
- Q: Why is execution not pausing at Rhai breakpoints? → A: The debugger registration is conditionally compiled out via `#[cfg(debug_assertions)]`. Additionally, Rhai's debugger defaults to `DebuggerStatus::Continue`, bypassing the `on_step` callback unless breakpoints are registered in Rhai's internal collection. Since we track breakpoints externally inside `DebugController`, we must set the initial status to `StepInto` to evaluate lines dynamically.
- Q: Should we verify debugging a custom function and procedure from the Workstation UI using chrome-devtools? → A: Yes, we will add success criteria to verify that both Rhai functions and procedures can be created via the web workstation UI and successfully paused and resumed at breakpoints.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Rhai Breakpoint Pause and Resume (Priority: P1)

A database developer debugging a Rhai stored procedure or trigger wants the execution to successfully pause at set breakpoints so they can inspect variables and control the flow of execution.

**Why this priority**: Core debugging functionality. Currently, Rhai execution runs to completion without hitting any breakpoints.

**Independent Test**: Can be independently tested via `cargo nextest run --test rhai_scripting_test --features rhai` with a new integration test simulating a debug session.

**Acceptance Scenarios**:

1. **Given** a Rhai procedure execution, **When** a breakpoint is set in `DebugController` for line 3 of the procedure, **Then** execution must block at line 3, trigger a `stopped` event, and register the local scope variables.
2. **Given** execution is paused at line 3, **When** a `continue` command is issued, **Then** execution must resume and complete successfully.
3. **Given** execution is paused, **When** a `disconnect` command is issued, **Then** execution must run to completion without any further pauses or hangs.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST enable Rhai debugger capabilities unconditionally (removing `#[cfg(debug_assertions)]`).
- **FR-002**: System MUST configure Rhai debugger's initial status to `StepInto` during `on_init` to ensure the `on_step` callback is executed for every line.
- **FR-003**: System MUST check for breakpoints using `DebugController::has_breakpoint` inside Rhai's debugger callback.
- **FR-004**: System MUST map the `ResumeAction` returned by `DebugController::pause_execution` to the correct Rhai `DebuggerCommand` (e.g. `ResumeAction::Continue` maps to `DebuggerCommand::StepInto` to allow hitting subsequent breakpoints).
- **FR-005**: Workstation UI MUST support creating and debugging Rhai functions and procedures.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new integration test `test_rhai_debugger` executes and passes, proving that breakpoints pause the thread, expose scope, and resume correctly.
- **SC-002**: Standard tests (`make test`) and lints (`make lint`) compile and pass without regressions.
- **SC-003**: The debugger compiles and is fully functional in both debug and release profiles.
- **SC-004**: End-to-end UI verification using chrome-devtools can successfully create a Rhai function and a Rhai procedure in the workstation, set a breakpoint on each, trigger their execution, verify the pause, and resume them successfully.

## Assumptions

- Stepping overhead is minimal during production runtimes because the debugger logic is bypassed unless a thread-local `DebugController` is registered for the execution.
