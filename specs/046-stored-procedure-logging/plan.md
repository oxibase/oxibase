# Implementation Plan: Formal Stored Procedure Logging API

**Branch**: `046-stored-procedure-logging` | **Date**: June 19, 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/046-stored-procedure-logging/spec.md`

## Summary

This feature establishes a formal, unified, and secure API for logging inside user-defined stored procedures and scripts. Instead of directly executing SQL insert commands on system tables (which bypassed namespace security and is now strictly forbidden), procedures written in Rhai, Python, and PL/SQL will utilize a formal logging interface.
- Rhai scripts: `oxibase::log(level, msg)`
- Python scripts: `oxibase.log(level, msg)`
- PL/SQL scripts: `LOG <level>, <expression>;`

These log calls route through Rust's native `tracing` ecosystem, ensuring asynchronous, non-blocking delivery to `system.logs`, full OpenTelemetry span and trace correlation, and correct handling of thread isolation.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `tracing`, `rhai`, `rustpython-vm`
**Testing**: `cargo nextest run` (via `make test`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Negligible latency and zero blocking on the hot transaction/query path by reusing the established async telemetry ring buffer logging layer.
**Constraints**: Avoid `unwrap()` / `expect()`, maintain no-warnings code, ensure full compatibility across features (`js`, `python`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, logs are persisted inside the monolith's standard `system.logs` telemetry table).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, by routing to standard logging channels rather than executing untracked SQL inserts, we avoid table lock-ups).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, leveraging zero-copy dynamic properties and references).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, utilizing Oxibase's `Result` type and returning errors to the scripting execution frame when needed).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, custom tests are planned for each procedural language backend).

## Project Structure

### Documentation (this feature)

```text
specs/046-stored-procedure-logging/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Research and design decisions
├── data-model.md        # Data models and attributes
├── contracts/
│   └── logging.md       # API and syntax contracts
└── quickstart.md        # Guide on how to write logs in each language
```

### Source Code (repository root)

```text
src/
├── common/
│   └── logging.rs       # Added central `log_message` dispatcher
├── executor/
│   └── dml.rs           # REVERT the direct insert workaround on system.logs
├── functions/
│   ├── backends/
│   │   ├── rhai.rs      # Register `oxibase::log`
│   │   └── python.rs    # Register `oxibase.log`
│   └── plsql/
│       ├── parser.rs    # AST parsing for `LOG` keyword
│       ├── ast.rs       # Node representation
│       └── interpreter.rs # Statement evaluation
```

**Structure Decision**: A central, thread-safe `log_message` helper is defined in `src/common/logging.rs` so that all backends dispatch logs identically. The Rhai and Python modules are updated to expose this helper, and the PL/SQL parser is extended with a new AST statement node and interpreted execution branch.
