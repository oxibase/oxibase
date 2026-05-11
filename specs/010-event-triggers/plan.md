# Implementation Plan: Event Triggers (BEFORE/AFTER INSERT, UPDATE, DELETE)

**Branch**: `010-event-triggers` | **Date**: 2026-05-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/010-event-triggers/spec.md`

## Summary

Implement row-level event triggers (`BEFORE`/`AFTER` for `INSERT`, `UPDATE`, `DELETE`) within the Oxibase monolithic database engine. This involves parsing trigger definitions, hooking into the DML execution pipeline (`src/executor/`), and exposing `NEW`/`OLD` row context to the embedded procedural engines (Rhai, JS, Python) while respecting strict ACID properties and preventing recursion panics.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `thiserror`, `anyhow`, `rhai` (default), `boa_engine` (js), `rustpython-vm` (python)
**Testing**: `cargo nextest` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`.

### Unknowns to Resolve (Phase 0)
- **NEEDS CLARIFICATION**: How are procedural language environments instanced and cached? Are they per-transaction, per-statement, or globally pooled? (Impacts performance if we instantiate a VM per row in a bulk insert).
- **NEEDS CLARIFICATION**: How is the AST for `CREATE TRIGGER` represented in `src/parser/` and how is trigger metadata stored in the catalog (`src/storage/` or `src/core/`)?
- **NEEDS CLARIFICATION**: How do we pass `Row` structs (which might contain zero-copy references) into the procedural engines safely and efficiently?

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, triggers run inside the engine).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, hooks must tie into transaction state).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, zero-copy goals apply to row context).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, strict no `unwrap()` policy).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes).

## Project Structure

### Documentation (this feature)

```text
specs/010-event-triggers/
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
├── parser/        # Add CREATE TRIGGER / DROP TRIGGER parsing
├── executor/      # Inject trigger hooks into insert/update/delete pipelines
├── storage/       # Update catalog to store trigger definitions
├── core/          # Trigger metadata structures
└── procedures/    # (Assumed or existing module) Bindings for NEW/OLD context
```

**Structure Decision**: This feature heavily impacts `src/parser/` (DDL), `src/storage/` (catalog), and `src/executor/` (hook injection).
