# Research: Procedural Random Support

This document details the architectural decisions and research conducted for supporting native random number generation across all three procedural backend languages in Oxibase.

## Decisions & Designs

### Decision 1: Rhai Native Function Registration
- **Decision**: Register `oxibase::random()` as a native function under the `oxibase` module in `src/functions/backends/rhai.rs`.
- **Rationale**: Keeps execution logic within the established module structure. Zero-copy and zero-overhead.
- **Alternatives Considered**: Direct inclusion in the Rhai global environment. Rejected to keep the `oxibase` namespace clean and isolated.

### Decision 2: Python Native Function Registration
- **Decision**: Register `oxibase.random()` as a `#[pyfunction]` within the existing `oxibase_py_module` in `src/functions/backends/python.rs`.
- **Rationale**: Parity with the Rhai implementation. Straightforward registration using the rustpython macros.
- **Alternatives Considered**: Standard library `random` module usage in Python code. Rejected because standard library imports inside the lightweight rustpython-vm context can be slow, heavy, and lack proper control over sandboxing and state. Exposing a database-provided RNG is more consistent and secure.

### Decision 3: PL/SQL Function Evaluation via General Function Registry
- **Decision**: Implement generic support for `Expression::FunctionCall` inside PL/SQL `PlSqlInterpreter::eval_expr` by querying the database's central `FunctionRegistry`.
- **Rationale**: Completely elegant and generic. Instead of just adding a hardcoded `random()` handler, this enables PL/SQL scripts to call **any** registered scalar function (including `RANDOM()`, `ABS()`, `ROUND()`, `SIN()`, `UPPER()`, etc.) directly, significantly expanding PL/SQL capabilities.
- **Alternatives Considered**: Adding custom keywords and parser support specifically for `random()`. Rejected because it would introduce complex parser-level rules for each built-in function, leading to over-engineering.

## Performance and Thread-Safety
- **RNG Engine**: Using `rand::rng()` provides a fast, thread-local, and cryptographically secure random number generator. This avoids lock contention across concurrent query-executing threads, matching our memory efficiency principles.
