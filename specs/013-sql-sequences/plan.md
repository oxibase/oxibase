# Implementation Plan: SQL Sequences

**Branch**: `main`
**Input**: Feature specification from `specs/013-sql-sequences/spec.md`

## Summary

This feature adds native support for standard SQL Sequences (`CREATE SEQUENCE`, `NEXTVAL`, `CURRVAL`, etc.) to oxibase. Sequences require strict concurrency guarantees to bypass standard MVCC write-conflicts, allowing non-blocking unique identifier generation across threads.

## Technical Context

**Language/Version**: Rust 1.85+
**Dependencies**: `parking_lot`, `dashmap`
**Testing**: SQL logic tests (AST parsing) + Concurrent integration tests for `NEXTVAL` thread-safety (`cargo nextest run`)
**Integrations**: Information schema (`information_schema.sequences`), execution context (session state)

## Constitution Check

- [x] **Mainframe Monolith**: Adheres to the embedded architecture.
- [x] **ACID & MVCC**: Safely bypasses MVCC Write-Write conflicts for sequence increments to allow non-blocking, autonomous transaction IDs, honoring the need for monotonic data integrity.
- [x] **Memory Efficiency**: Utilizing `AtomicI64` and `DashMap` provides zero-allocation atomic increments on sequence access.
- [x] **Safe Rust**: No `unwrap()`; standard error propagation via `Result` when limits are hit or sequences don't exist.
- [x] **Tests First**: Concurrency and standard SQL functionality will be explicitly tested.

## Project Structure

### Documentation

```text
specs/013-sql-sequences/
├── plan.md               # This file
├── research.md           # SQL semantics and concurrency architecture research
├── data-model.md         # Data structures and catalog modeling
├── contracts/
│   └── parser-ast.md     # Parser and trait interface definitions
└── quickstart.md         # Usage guide
```

### Source Code Impacts

```text
src/
├── parser/
│   ├── ast.rs            # Add CreateSequence, AlterSequence, DropSequence nodes
│   └── parser.rs         # Implement parsing grammar for SEQUENCE
├── catalog/
│   └── schema.rs         # Add sequence mappings and storage maps
├── executor/
│   ├── ddl.rs            # Handle CREATE/ALTER/DROP Sequence execution
│   ├── session.rs        # Add SessionState to track currval isolated per session
│   └── info_schema.rs    # Wire `information_schema.sequences` view to catalog
└── functions/
    └── scalar/
        └── sequence.rs   # Implement nextval(), currval(), setval()
```
