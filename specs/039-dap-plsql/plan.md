# Implementation Plan: DAP Support for PL/SQL Procedures

**Branch**: `039-dap-plsql` | **Date**: 2026-06-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/039-dap-plsql/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

This feature integrates the native PL/SQL engine with the system's new Debug Adapter Protocol (DAP) architecture. It modifies the PL/SQL AST to retain source location metadata, injects an execution hook into the `PlSqlInterpreter` loop to pause execution, maps internal `Environment` state to standard DAP variables, and wires the engine up to the `DebugController`. This enables developers to connect IDEs via DAP, set breakpoints, step through code, and inspect state in PL/SQL procedures.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `thiserror`, `anyhow`
**Testing**: `cargo nextest` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Negligible overhead when debugging is inactive.
**Constraints**: No `unwrap()`, preserve existing AST cloning semantics efficiently, integrate seamlessly without affecting normal non-debug execution speeds.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, the DAP server is part of the embedded DB and no new external components are strictly required besides the client IDE).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, paused procedures will hold transaction locks; this is an expected developer debugging consequence, but does not violate ACID).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, line numbers are lightweight `usize` fields).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, standard `Result` logic applied).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, DAP integration tests are mandated).

## Project Structure

### Documentation (this feature)

```text
specs/039-dap-plsql/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (future)
```

### Source Code (repository root)

```text
src/
└── functions/
    └── plsql/
        ├── ast.rs          # Modified to add line numbers
        ├── parser.rs       # Modified to inject line numbers into AST
        ├── interpreter.rs  # Modified to add DAP hook evaluation
        └── env.rs          # Modified to export DAP variables
```

**Structure Decision**: This feature is almost exclusively contained within the `src/functions/plsql/` module, with minor integration hooks with the global `DebugController` (wherever it resides as per task #33).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
