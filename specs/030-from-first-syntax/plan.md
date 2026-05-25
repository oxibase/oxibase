# Implementation Plan: FROM-First Syntax

**Branch**: `030-from-first-syntax` | **Date**: 2026-05-25 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/030-from-first-syntax/spec.md`

## Summary

Add support for DuckDB-style `FROM`-first queries to Oxibase. This allows the `FROM` clause to lead a statement, optionally followed by standard clauses like `SELECT`, `WHERE`, `ORDER BY`, etc., in any order. If `SELECT` is omitted, the parser will implicitly project `SELECT *`. The technical approach relies entirely on parser-level AST rewriting: the parser will produce a standard `SelectStatement` node for `FROM`-first queries, preventing any required changes to the logical planner, optimizer, or execution engine.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `thiserror`, `anyhow`
**Testing**: `cargo nextest` (via `make test` / `make test-all`)
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
specs/030-from-first-syntax/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (to be created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── parser/        # SQL parser (lexer, AST, parser) <-- Primary impact
tests/             # Integration tests <-- New tests required
```

**Structure Decision**: The feature primarily impacts `src/parser/` (specifically `statements.rs` and potentially `ast.rs` if minor display adjustments are needed). Integration tests will be added to the test suite to verify AST construction and semantic equivalence.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None      | N/A        | N/A                                 |
