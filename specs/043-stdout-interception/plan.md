# Implementation Plan: Scripting Stdout Interception

**Branch**: `043-stdout-interception` | **Date**: 2026-06-18 | **Spec**: [specs/043-stdout-interception/spec.md](spec.md)
**Input**: Feature specification from `specs/043-stdout-interception/spec.md`

## Summary

Implement standard output interception for Rhai and PL/SQL scripting backends. Rhai will utilize its native `on_print` hook, while the PL/SQL parser and interpreter will be extended to support `PRINT expression;` and `RAISE NOTICE expression;`, both evaluating the expression and appending the result to the execution context's standard output buffer.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `rhai`
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (No new external microservices/APIs)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity?
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)?
- [x] **Safe Rust**: Are errors properly propagated? (No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or unjustified `unsafe`)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`?

## Project Structure

### Documentation (this feature)

```text
specs/043-stdout-interception/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
└── functions/
    ├── backends/
    │   └── rhai.rs        # Rhai Engine on_print hook configuration
    └── plsql/
        ├── ast.rs         # Add Print statement to PlSqlStatement enum
        ├── parser.rs      # Parse PRINT and RAISE NOTICE
        └── interpreter.rs # Execute Print statements and forward to stdout
```

**Structure Decision**: This feature directly impacts the `src/functions/backends` and `src/functions/plsql` modules responsible for backend script execution.