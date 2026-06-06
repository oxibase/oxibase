# Implementation Plan: Telemetry Correlation

**Branch**: `034-telemetry-correlation` | **Date**: 2026-06-06 | **Spec**: [specs/034-telemetry-correlation/spec.md](spec.md)
**Input**: Feature specification from `specs/034-telemetry-correlation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Currently, the system's logging, tracing, and metrics modules exist independently, resulting in isolated observability data. This feature will implement context propagation and capture structured fields so that logs, metrics, and traces are properly correlated by attaching trace IDs and span IDs to logs and metrics, and capturing all structured fields into JSON representations.

## Technical Context

**Language/Version**: Rust 1.85+ (expected for modern backend)
**Primary Dependencies**: `tracing`, `tracing-subscriber`, `serde_json`, `chrono`, `crossbeam_channel`
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency. The `tracing` logic must not introduce heavy allocations inside the hot path. JSON serialization is deferred to the flusher where possible, or done carefully without unbounded allocations.
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? Yes, modifies internal telemetry processing.
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? Yes, does not alter storage engine logic, only telemetry ingestion into internal tables.
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? Yes, context propagation uses `tracing` extensions which are pre-allocated/managed efficiently.
- [x] **Safe Rust**: Are errors properly propagated? (No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or unjustified `unsafe`) Yes.
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? Yes, integration tests will be updated/added for telemetry correlation.

## Project Structure

### Documentation (this feature)

```text
specs/034-telemetry-correlation/
├── plan.md              # This file
├── research.md          # Implementation details and findings
├── data-model.md        # Telemetry data models
├── quickstart.md        # Quickstart for the telemetry setup
└── tasks.md             # Tasks for implementation
```

### Source Code (repository root)

```text
src/common/
├── logging.rs           # Log layer and log flusher
├── metrics.rs           # Metrics layer and metrics flusher
├── tracing.rs           # Tracing layer and trace flusher
tests/
├── tracing_ingestion.rs # Existing trace test to update
├── logging_ingestion.rs # Existing log test to update/add
└── metrics_ingestion_test.rs # Existing metrics test to update
```

**Structure Decision**: This feature directly impacts `src/common/logging.rs`, `src/common/tracing.rs`, and `src/common/metrics.rs`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
