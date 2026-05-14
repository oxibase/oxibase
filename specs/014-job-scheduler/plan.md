# Implementation Plan: Built-in Job Scheduler for Procedures

**Branch**: `014-job-scheduler` | **Date**: 2026-05-14 | **Spec**: `/specs/014-job-scheduler/spec.md`
**Input**: Feature specification from `/specs/014-job-scheduler/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Built-in Job Scheduler for Procedures and internal system table migration. Implements `CREATE/ALTER/DROP SCHEDULE` syntax, evaluates schedules using the `cron` crate, and stores configurations in `system.cron` and logs in `system.cron_runs`. A background thread handles execution autonomously. Additionally, migrates all existing `_sys_*` metadata tables (functions, procedures, statistics) to the `system` schema.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: `cron = "0.12"` (new dependency), thiserror, anyhow
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, jobs run inside the database process).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, jobs execute within standard transactions. Migrations happen inside transactions).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, by sleeping the thread efficiently).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, execution errors are safely caught and logged).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes).

## Project Structure

### Documentation (this feature)

```text
specs/014-job-scheduler/
├── plan.md              
├── research.md          
├── data-model.md        
├── quickstart.md        
├── contracts/           
│   └── ddl.md
└── tasks.md             
```

### Source Code (repository root)

```text
src/
├── api/
│   └── database.rs      # Modify DatabaseInner to spawn the scheduler thread
├── executor/
│   ├── ddl.rs           # Handle CREATE/ALTER/DROP SCHEDULE commands
│   ├── information_schema.rs # Update table references
│   └── mod.rs           # Scheduler execution bindings & table migrations
├── parser/
│   ├── ast.rs           # Add Schedule AST nodes
│   ├── statements.rs    # Parse CREATE/ALTER/DROP SCHEDULE
│   └── token.rs         # Add SCHEDULE, CRON, ACTIVE keywords
├── storage/
│   ├── jobs.rs          # (New) System table schemas and structures
│   ├── procedures.rs    # Update SYS_PROCEDURES to system.procedures
│   ├── functions.rs     # Update SYS_FUNCTIONS to system.functions
│   ├── statistics.rs    # Update SYS_TABLE_STATS / SYS_COLUMN_STATS
│   ├── triggers.rs      # Update SYS_TRIGGERS to system.triggers
│   └── mvcc/
│       └── engine.rs    # Initialization of system schema & tables
Cargo.toml               # Add `cron` crate dependency
```

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
