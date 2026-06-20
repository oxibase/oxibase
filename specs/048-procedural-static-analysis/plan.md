# Implementation Plan: AST-in-AST Static Analysis for Related Objects Detection

**Branch**: `048-procedural-static-analysis` | **Date**: June 20, 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/048-procedural-static-analysis/spec.md`

## Summary

This feature implements an AST-in-AST pipeline to statically extract and detect referenced database objects (Tables, Procedures, Functions) inside scripting procedures (Rhai, Python, PL/SQL/SQL) without executing them.
- Rhai scripts: Parsed to Rhai AST, walking `FnCall` nodes of `oxibase` calls to extract literal SQL, then running the native SQL visitor.
- Python scripts: Parsed to Python AST (via `rustpython_vm`), walking `Call` nodes of `oxibase` calls to extract literal SQL, then running the native SQL visitor.
- PL/SQL scripts: Directly parsed to SQL Statements and walked using our native SQL visitor.
- Gracefully detects dynamic string construction in database calls and appends a `"Dynamic"` object reference marker.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `rhai` (with `internals`), `rustpython-vm` (with compiler & AST), `serde`
**Testing**: `cargo nextest run` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Fast, read-only static analysis on the fly using zero-copy matching where possible.
**Constraints**: Avoid `unwrap()` / `expect()`, propagate errors cleanly, must pass `make lint` and `make license`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, script analysis is performed directly in the database runtime memory).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, analysis is a read-only metadata operation with no transactions or mutations).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, recursively traversing the ASTs directly using references).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, utilizing Result with standard error-mapping and avoiding standard macros like `unwrap()`).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, comprehensive unit and integration tests).

## Project Structure

### Documentation (this feature)

```text
specs/048-procedural-static-analysis/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── analysis.md      # Interface contracts
└── checklists/
    └── requirements.md  # Quality checklists
```

### Source Code (repository root)

```text
src/
├── parser/
│   ├── mod.rs               # Register visitor module
│   └── visitor.rs           # SQL AST Visitor & DependencyExtractor
├── functions/
│   ├── mod.rs               # Register analyzer module
│   └── analyzer.rs          # Orchestrator and backend-specific AST walking
└── api/
    ├── mod.rs               # Re-export RelatedObject
    └── database.rs          # Expose Database::analyze_script
```

**Structure Decision**: Place general SQL visiting logic in `src/parser/visitor.rs`, backend AST walking in `src/functions/analyzer.rs`, and expose the public read-only API method in `src/api/database.rs` to maintain high cohesion and separation of concerns.

## Complexity Tracking

*No constitutional violations or complex workarounds needed. Simple and clean implementation.*
