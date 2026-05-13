# Implementation Plan: Scripting Backend Context Refactor (oxibase.ctx)

**Branch**: `012-scripting-context` | **Date**: May 13 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/012-scripting-context/spec.md`

## Summary

This feature refactors how the `old` and `new` row proxies are exposed to the embedded scripting engines (Python, JS/Boa, and Rhai) inside triggers. It removes the magic global `OLD` and `NEW` variables and encapsulates them cleanly under an `oxibase.ctx` object context (`oxibase.ctx.old` and `oxibase.ctx.new`).

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `rhai`, `boa_engine` (js), `rustpython-vm` (python)
**Testing**: `cargo nextest run --features js,python`
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: No added memory overhead from the nested dictionary/object abstraction.
**Constraints**: No `unwrap()`, proper error propagation via `Result` through the JS and Python object interaction boundaries.

## Constitution Check

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? Yes, this refines internal scripting without breaking architecture.
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? Yes, the underlying row proxies are unchanged; only the pointer paths change.
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations? Yes, we are simply allocating basic context dictionaries/objects once per execution instead of polluting the global scope.
- [x] **Safe Rust**: Are errors properly propagated? Yes, we will rigorously handle `PyResult` and `JsResult` instead of using `unwrap()`.
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? Yes, tests will be explicitly modified and run to verify this refactoring.

## Project Structure

### Documentation (this feature)

```text
specs/012-scripting-context/
├── plan.md              # This file
├── research.md          # N/A
├── data-model.md        # N/A
├── contracts/           # API Contract Documentation
└── tasks.md             # Task Breakdown
```

### Source Code Impacts

```text
src/
└── functions/
    └── backends/
        ├── rhai.rs      # Modifying global scope injection
        ├── boa.rs       # Modifying global object property assignment and extraction
        └── python.rs    # Modifying PyDict creation and injection into the sys_module
```

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None      | N/A        | N/A                                 |

