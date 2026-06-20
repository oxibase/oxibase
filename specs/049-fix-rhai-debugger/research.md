# Research: Rhai Debugger Fix

## 1. Enabling Debugger Hooks Dynamically
- **Decision**: Remove the `#[cfg(debug_assertions)]` guard around `register_debugger` in `src/functions/backends/rhai.rs`.
- **Rationale**: Debugging capabilities should be compiled in both debug and release configurations so they are available in production deployments or standard releases without requiring custom build flags.
- **Alternatives considered**: Keeping the guard but defining a custom compile-time feature (e.g., `features = ["debugging"]`). Rejected because our workspace doesn't use standard features to gate general debugging, and the impact of compile-time presence is negligible.

## 2. Setting Initial Debugger Status
- **Decision**: Set the debugger's initial status to `DebuggerStatus::StepInto` in the `on_init` callback of `register_debugger`.
- **Rationale**: Rhai's engine defaults to `DebuggerStatus::Continue`. In this state, Rhai completely bypasses invoking the `on_step` callback unless breakpoints are registered in Rhai's *internal* breakpoint set. Because we manage breakpoints externally inside `DebugController` (to share state cleanly with PL/SQL and Python backends), we must run in `StepInto` mode so that the `on_step` hook is evaluated on every statement.
- **Alternatives considered**: Synchronizing external breakpoints from `DebugController` into Rhai's internal `Debugger` breakpoint collection. Rejected because of the unnecessary complexity of translating line numbers per-procedure dynamically and maintaining synchrony.

## 3. Command Mapping on Resume
- **Decision**: Map the `ResumeAction` returned by `DebugController::pause_execution` inside the debugger callback to the correct Rhai `DebuggerCommand`. Specifically, `ResumeAction::Continue` must map back to `DebuggerCommand::StepInto` to keep triggering the callback for subsequent statements (to hit other breakpoints or allow stepping).
- **Rationale**: If we returned `DebuggerCommand::Continue` after a pause, Rhai's debugger status would permanently transition to `Continue` for that script execution, meaning all subsequent breakpoints would be completely ignored.
- **Alternatives considered**: Always returning `DebuggerCommand::StepInto` unconditionally. Yes, this is exactly what we will do whenever the script is resumed, unless a `Disconnect` action is requested.
