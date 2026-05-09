# Implementation Plan: Transaction Management in Procedures

**Branch**: `008-procedure-transactions` | **Date**: 2026-05-09 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `specs/008-procedure-transactions/spec.md`

## Summary

Implement transaction management (`COMMIT`, `ROLLBACK`, `BEGIN`) within stored procedures. `COMMIT` and `ROLLBACK` will interact directly with the executor's transaction context via an extended `SqlRunner` trait. These capabilities will be exposed natively in PL/SQL and via global/module functions in Javascript (Boa), Python (RustPython), and Rhai. The implementation strictly mirrors PostgreSQL behavior: transaction control is permitted only if the `CALL` is not inside an explicit transaction block; otherwise, it throws an error.

## Technical Context

**Language/Version**: Rust 1.85+
**Primary Dependencies**: thiserror, anyhow, rhai, boa_engine, rustpython-vm
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)
**Performance Goals**: Zero-Copy Unikernel memory efficiency
**Constraints**: No `unwrap()`, strict ACID compliance, MVCC logic required, must pass `make lint` and `make license`

### Architecture Decisions (Research Phase Outcomes)

- **Execution Context**: `Executor::execute_call` will wrap procedure execution in an implicit transaction if no explicit transaction is active. A flag will track if the transaction is explicit (started by `BEGIN; CALL...`).
- **SqlRunner API**: `SqlRunner` trait will gain `commit()`, `rollback()`, and `begin()` methods. The `Executor` implementation of this trait will check the explicit transaction flag; if explicit, it throws an "invalid transaction termination" error. Otherwise, it commits/rolls back the active transaction and *immediately starts a new one*.
- **Scripting Contexts**: 
  - JS and Rhai will receive `commit()`, `rollback()`, and `begin()` as globally registered functions.
  - Python will receive them under the existing `oxibase` native module.
- **PL/SQL AST**: `PlSqlStatement` will get `Commit`, `Rollback`, and `BeginTransaction` variants, parsed from the respective tokens. `BeginTransaction` will execute as a no-op via the `SqlRunner::begin()` wrapper.

## Constitution Check

*GATE: Passed*

- [x] **Mainframe Monolith**: Yes. Transaction management stays tightly coupled to the embedded MVCC engine.
- [x] **ACID & MVCC**: Yes. `COMMIT` will finalize and publish the MVCC write set; `ROLLBACK` will discard it. Partial success is handled securely.
- [x] **Memory Efficiency**: Yes. Simple state swaps (`Option::take`) and reference passing to `SqlRunner`.
- [x] **Safe Rust**: Yes. Error handling propagates from the scripting engine via `Result`.
- [x] **Tests First**: Yes. `tests/procedure_tests.rs` will be expanded to cover transaction boundaries.

## Project Structure

### Documentation

```text
specs/008-procedure-transactions/
├── plan.md              # This file
├── research.md          # Implementation details and architectural decisions
├── data-model.md        # AST additions and contract definitions
└── spec.md              # Requirements and scenarios
```

### Source Code Impacts

```text
src/
├── functions/backends.rs           # Add commit/rollback/begin to SqlRunner trait
├── functions/backends/rhai.rs      # Inject functions to Rhai engine
├── functions/backends/boa.rs       # Inject functions to Boa context
├── functions/backends/python.rs    # Add functions to oxibase_py_module
├── functions/plsql/ast.rs          # Add Commit, Rollback, BeginTransaction AST nodes
├── functions/plsql/parser.rs       # Parse COMMIT, ROLLBACK, BEGIN statements
├── functions/plsql/interpreter.rs  # Execute transaction AST nodes via SqlRunner
├── executor/mod.rs                 # Implement SqlRunner methods for Executor
└── executor/query.rs               # Update execute_call to handle implicit tx state
```

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |