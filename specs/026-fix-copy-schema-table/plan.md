# Implementation Plan: Fix COPY Schema-Qualified Table Syntax

**Branch**: `026-fix-copy-schema-table` | **Date**: 2026-05-23 | **Spec**: [specs/026-fix-copy-schema-table/spec.md](spec.md)
**Input**: Feature specification from `specs/026-fix-copy-schema-table/spec.md`

## Summary

The oxibase SQL parser needs to be updated to support schema-qualified table names (like `cdm.concept`) in the `COPY ... FROM` statement. This involves modifying `parse_copy_statement` to parse a `TableName` instead of a simple `Identifier`, and updating the `CopyStatement` AST node to reflect this change.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: None new required (using existing AST components)
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? Yes.
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? Yes (no data engine changes, only parser/ast).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? Yes.
- [x] **Safe Rust**: Are errors properly propagated? Yes.
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? Yes, new parser tests will be added.

## Project Structure

### Documentation (this feature)

```text
specs/026-fix-copy-schema-table/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

**Structure Decision**: This feature primarily impacts `src/parser/statements.rs` (the parser logic) and `src/parser/ast.rs` (the `CopyStatement` node), as well as any usage in `src/executor/copy.rs` (the execution engine processing the statement).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None      | N/A        | N/A                                 |