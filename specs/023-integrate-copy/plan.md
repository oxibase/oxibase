# Implementation Plan: integrate-copy

**Branch**: `023-integrate-copy` | **Date**: 2026-05-22 | **Spec**: [specs/023-integrate-copy/spec.md](spec.md)
**Input**: Feature specification from `/specs/023-integrate-copy/spec.md`

## Summary

Integrate the `COPY FROM` command into Oxibase by porting the implementation from the `stoolap` fork. This provides an optimized fast-path for bulk data ingestion from CSV and JSON formats, bypassing standard per-row AST parsing. It utilizes `csv` for CSV handling and a custom streaming `JsonArrayStripper` over `serde_json` to maintain O(1) memory overhead.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `serde_json` (existing), `csv` (to be added)
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency (O(1) memory for parsing JSON and CSV).
**Constraints**: No `unwrap()`, strict ACID compliance, must pass `make lint` and `make license`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (No new external microservices/APIs)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, executes as an auto-commit transaction, rolls back on error).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, streaming parser, direct string to type coercion).
- [x] **Safe Rust**: Are errors properly propagated? (No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or unjustified `unsafe`)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`?

## Project Structure

### Documentation (this feature)

```text
specs/023-integrate-copy/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

**Structure Decision**: 
- `Cargo.toml`: Add `csv` dependency.
- `src/parser/ast.rs`: Add `CopyStatement` and `CopyFormat`.
- `src/parser/statements.rs`: Add parsing logic for `COPY FROM`.
- `src/executor/copy.rs`: Add the execution logic (ported from stoolap).
- `src/executor/mod.rs` & `src/executor/query.rs`: Integrate the `execute_copy` call.
- `tests/copy_from_test.rs`: Add integration tests.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A       | N/A        | N/A                                 |
