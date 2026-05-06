# Implementation Plan: App Scaffolding and Seeding CLI Commands

**Branch**: `main` | **Date**: May 06 2026 | **Spec**: [specs/005-review-issue-39/spec.md](spec.md)
**Input**: Feature specification from `/specs/005-review-issue-39/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement two new CLI commands in the Oxibase CLI: `create-app` to generate a standard declarative application scaffold, and `seed` to read this directory structure and deterministically load schemas, data, templates, routes, and functions into the database within a single transaction.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `clap` for CLI argument parsing, `serde_json` for reading route definitions, `std::fs` for file operations.
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, reinforces "App as Data")
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, `seed` uses a transaction)
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes)
- [x] **Safe Rust**: Are errors properly propagated? (Yes, `Result` will be returned)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, integration tests)

## Project Structure

### Documentation (this feature)

```text
specs/005-review-issue-39/
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
└── bin/           # CLI binary (oxibase.rs will be modified)
tests/             # Integration tests (new test file for CLI commands)
```

**Structure Decision**: This feature exclusively impacts the `src/bin/oxibase.rs` CLI entry point, and will require new integration tests.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., Unsafe Code Used] | [Performance critical loop] | [Safe abstraction was measured to be X% slower in benchmark Y] |
| [e.g., Memory Allocation] | [External library requirement] | [No zero-copy alternatives exist for dependency Z] |
