<!--
Sync Impact Report:
- Version change: 1.0.0 -> 1.1.0
- Added sections: Core Principles, Development Workflow, Governance
- Removed sections: N/A
- Templates requiring updates:
  - ⚠ .specify/templates/plan-template.md
  - ⚠ .specify/templates/spec-template.md
  - ⚠ .specify/templates/tasks-template.md
- Follow-up TODOs: Determine original RATIFICATION_DATE if earlier than today.
-->

# Oxibase Constitution

## Core Principles

### I. The Modern Mainframe Monolith
Oxibase is an autonomous SQL monolith where business logic executes directly within the database engine. N-tier architectural drift is strictly eliminated—no external configuration drift, redundant API layers, or decoupled microservices unless absolutely unavoidable. Infrastructure is Data.

### II. Strict ACID Integrity
Data correctness is non-negotiable. Every operation must adhere strictly to ACID properties backed by MVCC (Multi-Version Concurrency Control) and time-travel queries.

### III. Zero-Copy Unikernel Efficiency
Performance relies on minimal overhead. Code MUST avoid unnecessary memory allocations (e.g., `Vec` clones) and expensive context switching. The engine optimizes for direct data access and efficient edge computing scenarios.

### IV. Safe and Idiomatic Rust
Code MUST NOT use `unwrap()` or `expect()` in library routines; all errors must be propagated using `Result` with `thiserror`/`anyhow`. The use of `unsafe` is forbidden unless strictly necessary and extensively documented. `todo!()` or `unimplemented!()` macros are not permitted in committed code.

### V. Embedded Business Logic
Scripting backends (JavaScript/Boa, Python/RustPython, Rhai) are first-class citizens. Business logic runs alongside the data. When testing backend-specific logic, ensure appropriate features (e.g., `--features js`) are active.

## Development Workflow

- **Testing Discipline**: Use `cargo nextest run` (or `make test`). Tests MUST pass before merging.
- **Code Quality**: All code MUST pass `make lint` (`cargo fmt` and `cargo clippy -D warnings`).
- **Licensing**: Every `.rs` file MUST include the required Apache-2.0 copyright header (verify with `make license`).

## Governance

This Constitution supersedes local preferences. Amendments to these core rules require an increment to the constitution version:
- **MAJOR**: Fundamental shifts in architecture (e.g., abandoning the monolith).
- **MINOR**: Addition of new core languages, engines, or major workflow mandates.
- **PATCH**: Clarifications, wording improvements, and typo fixes.

All agents, contributors, and reviewers must verify compliance against these rules.

**Version**: 1.1.0 | **Ratified**: 2026-05-05 | **Last Amended**: 2026-05-05