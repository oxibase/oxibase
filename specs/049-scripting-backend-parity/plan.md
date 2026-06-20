# Implementation Plan: Scripting Backend Parity

**Branch**: `049-scripting-backend-parity` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/049-scripting-backend-parity/spec.md`

## Summary

Align the PL/SQL and Python scripting backends with the Rhai backend by adding native support for JSON and TIMESTAMP data types, as well as the `random()` function in PL/SQL. Ensure Python natively marshalls JSON into dicts/lists and TIMESTAMPs into datetime objects.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: rustpython-vm (python), rhai, rand
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
specs/049-scripting-backend-parity/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── functions/
│   ├── backends/
│   │   ├── python.rs    # Python backend marshalling logic
│   ├── plsql/
│   │   ├── interpreter.rs # PL/SQL JSON/TIMESTAMP and random() logic
tests/
├── python_scripting_test.rs # Python parity tests
├── procedure_plsql_tests.rs # PL/SQL parity tests
```

**Structure Decision**: This feature directly impacts `src/functions/backends/python.rs` and `src/functions/plsql/interpreter.rs`, with corresponding integration tests in `tests/`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
