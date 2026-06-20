# Implementation Plan: fix-rhai-debugger

**Branch**: `049-fix-rhai-debugger` | **Date**: 2026-06-20 | **Spec**: [specs/049-fix-rhai-debugger/spec.md](./spec.md)
**Input**: Feature specification from `/specs/049-fix-rhai-debugger/spec.md`

## Summary

This plan addresses the issue where the Rhai execution completely fails to pause at breakpoints. By modifying the Rhai engine's debugging configuration to unconditionally compile and initialize with `StepInto` status, the debugger hook evaluates all lines against the external `DebugController` breakpoints. The plan also properly maps `ResumeAction` values back to Rhai `DebuggerCommand` structures.

## Technical Context

**Language/Version**: Rust 1.85+ 
**Primary Dependencies**: rhai (for scripting backend), serde, serde_json
**Testing**: `cargo nextest` / `make test`
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Avoid statement-level evaluation overhead in standard execution by only checking debugger hooks when a `DebugController` context is registered.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, debugging does not impact storage layers).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations? (Yes).
- [x] **Safe Rust**: Are errors properly propagated? (Yes).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes).

## Project Structure

### Documentation (this feature)

```text
specs/049-fix-rhai-debugger/
├── spec.md              # Feature specification
├── plan.md              # This implementation plan
├── research.md          # Research findings and decisions
├── data-model.md        # State transition details
└── quickstart.md        # Quickstart and verification instructions
```

### Source Code (repository root)

```text
src/
└── functions/
    └── backends/
        └── rhai.rs      # Rhai backend engine registration and debugging
```

**Structure Decision**: The primary implementation changes will reside in `src/functions/backends/rhai.rs` to fix the debugger registration, status setup, and command mapping. We will also add a dedicated integration test to `tests/rhai_scripting_test.rs`.
