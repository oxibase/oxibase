# Implementation Plan: plsql-table-iteration

**Branch**: `050-plsql-table-iteration` | **Date**: 2026-06-20 | **Spec**: [specs/050-plsql-table-iteration/spec.md](./spec.md)
**Input**: Feature specification from `/specs/050-plsql-table-iteration/spec.md`

## Summary

This plan outlines the architecture, parser modifications, and interpreter additions to support:
1. A new PL/SQL variable type `TABLE` representing an array of rows (backed by `JSON` internally).
2. A streamlined iteration construct `FOR variable IN table_expression LOOP ... END LOOP;` in PL/SQL.
3. Native procedural query APIs `oxibase::query(sql)` (Rhai) and `oxibase.query(sql)` (Python).
4. Centralized SQL query scalar functions `QUERY_VALUE(sql)` and `QUERY_ROWS(sql)`.

---

## Technical Context

**Language/Version**: Rust 1.85+  
**Primary Dependencies**: rhai (scripting), rustpython-vm (python vm), serde/serde_json (JSON serialization)  
**Testing**: `cargo nextest` / `make test`  
**Target Platform**: Embedded Monolithic DB (Linux, macOS, Windows)

---

## Constitution Check

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, queries execute through thread-local SQL runner inside existing transactions).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations? (Yes, reusing `Value::Json` Arc-shared strings).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, propagating `Result` with `anyhow` / `Error`).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes).

---

## Project Structure

### Source Code Files Affected
```text
src/
├── parser/
│   ├── ast.rs               # Define parser models (if needed)
│   └── statements.rs        # Main SQL parser (no changes needed)
├── functions/
│   ├── plsql/
│   │   ├── ast.rs           # Add PlSqlStatement::ForLoop AST node
│   │   ├── parser.rs        # Parse variable type "TABLE" and "FOR...IN"
│   │   ├── interpreter.rs   # Interpret loop iteration, bind loop variables, dot-notations
│   │   └── backend.rs       # Setup PlSqlBackend dependencies
│   ├── backends/
│   │   ├── rhai.rs          # Expose oxibase::query in Rhai module
│   │   └── python.rs        # Expose oxibase.query in Python module
│   ├── scalar/
│   │   └── utility.rs       # Implement QUERY_VALUE & QUERY_ROWS scalar functions
│   └── registry.rs          # Register QUERY_VALUE and QUERY_ROWS functions
```

---

## Phased Implementation Details

### Phase 0: Research & Verification
- Verify thread-local SQL execution via `execute_sql_query` inside functions.
- Determine optimal mapping from query results to dynamic/JSON objects.
- Completed and documented in [research.md](./research.md).

### Phase 1: Design & Contracts
- Design AST representations and Environment assignments for dot-access and loops.
- Defined in [data-model.md](./data-model.md) and [quickstart.md](./quickstart.md).

### Phase 2: Implementation & Code Integration
- **Step 1**: Register `oxibase::query(sql)` in `src/functions/backends/rhai.rs`.
- **Step 2**: Register `oxibase.query(sql)` in `src/functions/backends/python.rs`.
- **Step 3**: Define `QUERY_VALUE` and `QUERY_ROWS` scalar functions in `src/functions/scalar/utility.rs` and register them in `src/functions/registry.rs`.
- **Step 4**: Extend `src/functions/plsql/ast.rs` with `PlSqlStatement::ForLoop` and modify `PlSqlParser::parse_variable_declaration` to parse `TABLE` as a sugar alias for `JSON` type.
- **Step 5**: Parse the `FOR variable IN table_expr LOOP ... END LOOP;` statement inside `src/functions/plsql/parser.rs`.
- **Step 6**: Implement loop execution, environment binding, and qualified dot-assignments inside `src/functions/plsql/interpreter.rs`.

### Phase 3: Verification & Integration Testing
- Create comprehensive integration tests:
  - `test_rhai_scripting_query`
  - `test_python_scripting_query`
  - `test_plsql_sugar_table_iteration`
- Verify with `cargo nextest run`.
- Confirm with `make lint`.
