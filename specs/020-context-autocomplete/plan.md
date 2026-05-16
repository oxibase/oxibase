# Implementation Plan: context-autocomplete

**Branch**: `021-context-autocomplete` | **Date**: May 16, 2026 | **Spec**: [specs/020-context-autocomplete/spec.md](specs/020-context-autocomplete/spec.md)
**Input**: Feature specification from `specs/020-context-autocomplete/spec.md`

## Summary

Add a context-aware autocomplete to the `oxibase` CLI by implementing the `rustyline::completion::Completer` interface. The autocomplete will suggest standard SQL keywords and dynamically fetch table names from the active `Database` instance (using `information_schema.tables`) when the cursor is in a table-related context (e.g., after `FROM`, `INTO`, `UPDATE`).

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `rustyline`
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency, autocomplete response under 50ms.
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? Yes, modifies the CLI built into the binary.
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? Yes, utilizes existing `Database::query()` which is MVCC safe.
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? Yes, reusing the Database instance connection (`Arc` clone) and efficient string parsing.
- [x] **Safe Rust**: Are errors properly propagated? Yes, any database errors during autocomplete are silently ignored (returning no suggestions) rather than panicking.
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? Yes, unit tests for the completer parsing logic.

## Project Structure

### Documentation (this feature)

```text
specs/020-context-autocomplete/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

**Structure Decision**: This feature primarily impacts `src/bin/oxibase.rs` where the CLI loop resides. We will introduce a `SqlHelper` struct that acts as the `rustyline` helper for autocompletion.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
