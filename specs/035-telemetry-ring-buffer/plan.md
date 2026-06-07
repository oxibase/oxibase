# Implementation Plan: Telemetry Ring Buffer Table

**Branch**: `035-telemetry-ring-buffer` | **Date**: 2026-06-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/035-telemetry-ring-buffer/spec.md`

## Summary

This feature replaces the standard MVCC tables used for internal telemetry (`system.traces`, `system.logs`, `system.metrics`) with a new `SystemRingBufferTable`. This new table bypasses the Write-Ahead Log (WAL) and MVCC locks, acting as a fixed-capacity, thread-safe, memory-bounded ring buffer. Furthermore, hot-path telemetry macros will be optimized to defer formatting and serialization to the background flusher threads, resolving severe bottlenecks and restoring zero-copy unikernel efficiency to user queries.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `crossbeam-channel` or `crossbeam-queue`, `parking_lot::RwLock`, `std::collections::VecDeque`
**Testing**: `cargo nextest run` (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency (0 locks in the hot path, 0 heap allocations for strings in the hot path, 0 WAL writes for telemetry)
**Constraints**: No `unwrap()`, strict memory boundaries, `make lint` and `make license` compliance.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, integrates deeply into the existing database architecture as a new Table implementation)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, by explicitly bypassing MVCC for telemetry to preserve ACID performance for user data)
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, this feature is fundamentally about removing massive `String`, JSON, and `Vec` allocations from the telemetry hot path)
- [x] **Safe Rust**: Are errors properly propagated? (Yes, no `unwrap()` or `expect()`)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, by ensuring existing telemetry tests still pass and adding capacity tests)

## Project Structure

### Documentation (this feature)

```text
specs/035-telemetry-ring-buffer/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── common/        # Tracing, Logging, and Metrics layers (Hot Path optimizations)
├── storage/mvcc/  # New `SystemRingBufferTable` implementation
└── storage/       # Integration into `MVCCEngine` logic (bypassing WAL)
tests/             # Telemetry integration tests
```

**Structure Decision**: This feature directly impacts `src/common/` (telemetry layers), `src/storage/mvcc/` (the new Table), and `src/storage/mvcc/engine.rs` (bypassing normal table instantiation for system tables).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (None) | N/A | N/A |