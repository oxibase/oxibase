# Implementation Plan: HTTP Headers Context for Python and PL/SQL

**Branch**: `044-http-headers-context` | **Date**: 2026-06-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/044-http-headers-context/spec.md`

## Summary

This plan extends the existing `get_http_header` context functionality (currently supported in Rhai and standard SQL) to both the Python and PL/SQL scripting engines. It registers the built-in `oxibase` native module in Python UDF standard executions, and adds support for evaluating `Expression::FunctionCall` within the PL/SQL AST interpreter's expression evaluation logic.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: RustPython VM, PL/SQL AST, thread-local context
**Testing**: `cargo nextest` (via `make test` or `cargo nextest run --features python`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Constraints**: No `unwrap()`, must pass `make lint` and `make license`

## Constitution Check

*GATE: Passed*

- [x] **Mainframe Monolith**: Yes. Exposes native header functions directly in the database runtime VM environment.
- [x] **ACID & MVCC**: Yes. Non-intrusive metadata context reading has zero impact on transactions or storage engine.
- [x] **Memory Efficiency**: Yes. Leverages thread-local variables with zero-copy where possible.
- [x] **Safe Rust**: Yes. Avoids panics, cleanly handles missing keys or missing context by returning native language representation of null.
- [x] **Tests First**: Yes. Integration tests will verify header parsing and retrieval for both Python and PL/SQL.

## Project Structure

### Documentation

```text
specs/044-http-headers-context/
├── plan.md              # This file
├── research.md          # Architectural decisions & findings
├── data-model.md        # Scripting and context state mapping
├── spec.md              # Requirements and user stories
├── contracts/
│   └── api.md           # API signatures for Python and PL/SQL
└── checklists/
    └── requirements.md  # Quality checklists
```

### Source Code

```text
src/
├── functions/backends/python.rs      # Initialize builder with native module def in execute()
├── functions/plsql/interpreter.rs   # Add Expression::FunctionCall in eval_expr
└── tests/server_rpc_tests.rs         # Integration tests for both backends
```

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
