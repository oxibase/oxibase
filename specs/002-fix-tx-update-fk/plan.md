# Implementation Plan: Fix Transaction Updates with Foreign Keys

**Branch**: `002-fix-tx-update-fk` | **Date**: 2026-05-05 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/002-fix-tx-update-fk/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Resolve an internal bug where rolling back a transaction fails to release row claims in the `MvccEngine` cache, leading to persistent "uncommitted changes" errors on subsequent queries. The fix involves adding a `rollback_all_tables` method to `TransactionEngineOperations` and implementing it in `MvccEngine` to properly cascade rollbacks and release MVCC row locks, bringing rollback cache handling inline with the existing commit logic.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `dashmap`
**Testing**: `cargo nextest` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, internal bugfix)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, explicitly fixes an MVCC leak)
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, strictly operates on cache references)
- [x] **Safe Rust**: Are errors properly propagated? (No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or unjustified `unsafe`) (Yes)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, integration tests covering transactions with FK constraints)

## Project Structure

### Documentation (this feature)

```text
specs/002-fix-tx-update-fk/
├── plan.md              # This file
├── research.md          # Research into the rollback cache leak
├── data-model.md        # State transitions of TransactionVersionStore
├── quickstart.md        # Steps to verify the fix
└── tasks.md             # Phase 2 output (future command)
```

### Source Code (repository root)

```text
src/
├── storage/
│   ├── mvcc/
│   │   ├── engine.rs      # Implementing `rollback_all_tables`
│   │   ├── transaction.rs # Updating `MvccTransaction::rollback` and `TransactionEngineOperations` trait
│   │   └── version_store.rs # Implementing `Drop` for safety net
tests/                 # Integration tests for transaction rollback & constraints
```

**Structure Decision**: This feature exclusively impacts `src/storage/mvcc/` because it addresses an internal logic flaw in how MVCC state caching handles rollbacks.

## Complexity Tracking

*No Constitution Check violations expected.*
