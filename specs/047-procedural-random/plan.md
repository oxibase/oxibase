# Implementation Plan: Procedural Random Support

**Branch**: `047-procedural-random` | **Date**: June 19, 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/047-procedural-random/spec.md`

## Summary

This feature adds native random number generation support (`random()`) across all three procedural language scripting backends (Rhai, Python, PL/SQL).
- In Rhai: `oxibase::random()`
- In Python: `oxibase.random()`
- In PL/SQL: `random()` (fully evaluated dynamically via the database's `FunctionRegistry` using generic `Expression::FunctionCall` support).

This ensures standard procedural parity and native, lock-free, thread-safe, and high-performance random generation during query execution.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `rand = "0.10"`, `rhai`, `rustpython-vm`
**Testing**: `cargo nextest run` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Lock-free thread-local RNG (`rand::rng()`) to prevent concurrent query contention.
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic unaffected, must pass `make lint` and `make license`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, scripting logic and random execution run completely inside the database engine).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, RNG calls have no database side-effects and do not interfere with MVCC concurrency).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, leveraging primitive types and native bindings).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, utilizing Result with standard error-mapping and avoiding standard macros like `unwrap()`).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, targeted integration tests for each procedural language backend).

## Project Structure

### Documentation (this feature)

```text
specs/047-procedural-random/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── random.md        # Interface contracts
└── checklists/
    └── requirements.md  # Quality checklists
```

### Source Code (repository root)

```text
src/
├── functions/
│   ├── backends/
│   │   ├── rhai.rs      # Expose `oxibase::random()`
│   │   └── python.rs    # Expose `oxibase.random()`
│   └── plsql/
│       └── interpreter.rs # Add `Expression::FunctionCall` support in `eval_expr`
```

**Structure Decision**: Exposes random function within the Rhai and Python backend modules, and enhances the PL/SQL interpreter to natively support scalar function calls by looking them up dynamically in the `FunctionRegistry`.

## Complexity Tracking

*No constitutional violations or complex workarounds needed. Simple and clean implementation.*
