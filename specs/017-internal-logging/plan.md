# Implementation Plan: Comprehensive Internal Logging System

**Branch**: `017-internal-logging` | **Date**: May 15, 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/017-internal-logging/spec.md`

## Summary

This feature intercepts application logs generated via the `tracing` ecosystem, routing standard console logs to a structured JSON format and high-severity events (INFO, WARN, ERROR) into a dedicated internal system table (`system.logs`). This requires a custom `tracing_subscriber::Layer`, a non-blocking `crossbeam-channel` for asynchronous delivery, a thread-local flag to prevent infinite logging loops within the flusher thread, and executor/storage updates to manage the new virtual/system table.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `tracing`, `tracing-subscriber`, `crossbeam-channel`
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Lock-free channel (`crossbeam-channel`) ensures logging emission (near-zero overhead) doesn't block the hot query path.
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`
**Key Unknowns**:
- `tracing_subscriber` layered initialization syntax in `src/bin/oxibase.rs`.
- The exact location/mechanics for creating `system.logs` vs `system.cron` in `src/executor/mod.rs` and `src/storage/`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, logs are stored internally in `system.logs`).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, internal background flusher will use standard database connection API which handles MVCC/transactions).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, leveraging `crossbeam-channel` for bounded memory lock-free delivery).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, standard `Result` propagation).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, via unit/integration tests that check the system table creation and log capture).

## Project Structure

### Documentation (this feature)

```text
specs/017-internal-logging/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
└── quickstart.md        # Phase 1 output
```

### Source Code (repository root)

```text
src/
├── bin/oxibase.rs       # Tracing setup for CLI
├── executor/
│   ├── mod.rs           # Background thread startup & system schema migration
│   └── system_schema.rs # system.logs virtual table interceptor/logic
├── storage/
│   └── logs.rs          # (New) Table definitions and constants for system.logs
└── common/
    └── logging.rs       # (New) Custom Layer and flusher thread logic
```

**Structure Decision**: A new `common/logging.rs` module will hold the custom `tracing_subscriber` Layer, the `crossbeam-channel` setup, and the background flusher task. `src/executor/mod.rs` and `src/storage/logs.rs` will handle the system table persistence. `src/bin/oxibase.rs` manages the final subscriber setup.
