# Implementation Plan: Script Engine Event Triggers (Boa and RustPython)

**Branch**: `011-script-engines` | **Date**: 2026-05-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/011-script-engines/spec.md`

## Summary

Expand the Event Triggers feature (implemented in `010-event-triggers`) to fully support the optional JavaScript (Boa) and Python (RustPython) scripting engines. This requires implementing engine-specific proxy objects (`NEW` and `OLD`) that securely bind to the thread-local execution context, allowing scripts to read and mutate transaction rows natively and with zero-copy overhead.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `boa_engine` (js), `rustpython-vm` (python)
**Testing**: `cargo nextest` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`.

### Unknowns to Resolve (Phase 0)
- **NEEDS CLARIFICATION**: How exactly do we implement `__getattr__` and `__setattr__` dynamically on a rustpython `#[pyclass]` when the properties aren't known at compile time, but rather depend on the `Schema` inside the thread local?
- **NEEDS CLARIFICATION**: How do we implement a Javascript `Proxy` object using the `boa_engine` Rust API to intercept dynamic property access for the `NEW` and `OLD` row representations?

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, extends internal engines)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, relies on existing thread-local trigger isolation)
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, strictly using lazy-evaluated proxies pointing to memory)
- [x] **Safe Rust**: Are errors properly propagated? (Yes)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes)

## Project Structure

### Documentation (this feature)

```text
specs/011-script-engines/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
src/
├── functions/backends/python.rs  # Add RustPython proxy classes and execution injections
├── functions/backends/boa.rs     # Add Boa JS Proxy logic and execution injections
└── functions/backends/mod.rs     # Minor exposure of trigger context if necessary
```

**Structure Decision**: This feature exclusively impacts the `python.rs` and `boa.rs` backend adapters.
