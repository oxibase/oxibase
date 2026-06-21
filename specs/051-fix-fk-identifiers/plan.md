# Implementation Plan: Schema-Qualified Foreign Keys

**Branch**: `051-fix-fk-identifiers` | **Date**: June 20, 2026 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/051-fix-fk-identifiers/spec.md`

## Summary

This feature resolves issue 140 by allowing table-level and column-level `FOREIGN KEY` constraints to reference tables in non-default schemas (e.g. `REFERENCES crm.customers(id)`). The parser will be updated to use the `TableName` AST type for referenced tables, and the validation/referential-action execution logic in the executor will dynamically resolve unqualified referenced table names relative to the referencing table's schema.

## Technical Context

* **Language/Version**: Rust 1.85+ (expected for modern backend)
* **Primary Dependencies**: `thiserror`, `anyhow`, `parking_lot`, `dashmap`
* **Testing**: `cargo nextest` (via `make test` / `make test-all`)
* **Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
* **Performance Goals**: Zero-Copy Unikernel memory efficiency
* **Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (No new external microservices/APIs)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity?
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)?
- [x] **Safe Rust**: Are errors properly propagated? (No `unwrap()`, `expect()`, `todo!()`, `unimplemented!()`, or unjustified `unsafe`)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`?

## Project Structure

### Documentation (this feature)

```text
specs/051-fix-fk-identifiers/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/           # Phase 1 output
    └── sql_grammar.md   # SQL Grammar interface contract
```

### Source Code (repository root)

```text
src/
├── core/
│   └── schema.rs        # Schema and ForeignKeyMetadata core types
├── parser/
│   ├── ast.rs           # Token definitions, TableConstraint, ColumnConstraint
│   └── statements.rs    # Parsing logic for constraints and statements
└── executor/
    ├── ddl.rs           # Table creation and constraint verification logic
    └── dml.rs           # Referential integrity validation during inserts and deletes
```

### Structure Decision
We will modify the core AST and parser in `src/parser/` to support parsing `TableName` instead of raw `Identifier` tokens for both table-level and column-level `REFERENCES` constraints. We will also update the validation and execution hooks in `src/executor/ddl.rs` and `src/executor/dml.rs` to dynamically resolve unqualified referenced names relative to the schema of the referencing table.

## Complexity Tracking

*No violations of the Constitution are planned or required. The implementation is clean, safe, and leverages existing idiomatic patterns.*
