# Feature Specification: Procedural Random Support

**Feature Branch**: `047-procedural-random`  
**Created**: June 19, 2026  
**Status**: Draft  
**Input**: User description: "create the spec for the random"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Rhai Procedural Random (Priority: P1)

Users writing database scripts and stored procedures in Rhai need to generate random floats natively without executing custom SQL queries via `oxibase::execute()`.

**Why this priority**: High priority as it resolves the core issue raised in GitHub #88 and establishes the core functionality of native random number generation in the default scripting language.

**Independent Test**: Can be fully tested by an integration test in `tests/procedure_tests.rs` creating a Rhai procedure that calls `oxibase::random()` and verifying that the output is a float within the correct range.

**Acceptance Scenarios**:

1. **Given** a database connection, **When** calling a Rhai procedure that retrieves `oxibase::random()`, **Then** the result is a float between `0.0` (inclusive) and `1.0` (exclusive).

---

### User Story 2 - Python Procedural Random (Priority: P2)

Users writing database scripts and stored procedures in Python need to generate random floats natively without having to invoke SQL query workarounds.

**Why this priority**: Medium priority to ensure parity across procedural engines (maintaining the "three procedural languages" principle).

**Independent Test**: Can be fully tested by an integration test in `tests/procedure_multilang_tests.rs` creating a Python procedure that calls `oxibase.random()` and verifying the output range.

**Acceptance Scenarios**:

1. **Given** a database connection, **When** calling a Python procedure that retrieves `oxibase.random()`, **Then** the result is a float between `0.0` (inclusive) and `1.0` (exclusive).

---

### User Story 3 - PL/SQL Procedural Random & Built-in Functions (Priority: P2)

Users writing database functions and procedures in PL/SQL need to call `random()` and other standard database functions natively within procedural expressions and assignments (e.g., `v_rand := random();`).

**Why this priority**: Medium priority to ensure complete functional parity across all scripting/procedural engines, enabling native function evaluation inside PL/SQL expressions.

**Independent Test**: Can be fully tested by an integration test in `tests/plsql_functions.rs` executing a PL/SQL function containing math/random function calls.

**Acceptance Scenarios**:

1. **Given** a database connection, **When** calling a PL/SQL function that assigns `random()` to a variable and returns it, **Then** the returned result is a float between `0.0` (inclusive) and `1.0` (exclusive).

---

### Edge Cases

- **Concurrent Execution**: Calling random functions simultaneously across parallel query threads/transactions must be safe and produce independent random values without blocking on a shared global lock.
- **Null & Type Coercion**: Handling cases where the random output is coerced to other database types (e.g., assigning a random float to an integer column/variable should correctly round/coerce according to standard conversion rules).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Rhai backend MUST register and expose an `oxibase::random()` function returning a random float.
- **FR-002**: The Python backend MUST register and expose an `oxibase.random()` function returning a random float.
- **FR-003**: The PL/SQL interpreter MUST support evaluating `Expression::FunctionCall` AST nodes in `eval_expr` by looking them up in the database's `FunctionRegistry`.
- **FR-004**: Random float generation MUST return values in the uniform range `[0.0, 1.0)`.
- **FR-005**: All procedural backends MUST maintain thread-safety and avoid shared state bottlenecks during concurrent query execution.

### Key Entities

- **[FunctionRegistry]**: Central registry of all SQL functions in Oxibase, used by PL/SQL interpreter to resolve function names to scalar function instances.
- **[RhaiBackend]**: Scripting engine for Rhai, exposing native `oxibase::random()`.
- **[PythonBackend]**: Scripting engine for Python, exposing native `oxibase.random()`.
- **[PlSqlInterpreter]**: Interpreter for PL/SQL statements and expressions, extended to evaluate scalar functions in expressions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Passes all new and existing integration test suites verified via `cargo nextest run`.
- **SC-002**: Generates random numbers with uniform distribution and zero blocking/lock contention under concurrent execution.
- **SC-003**: Passes `make lint` without warnings and introduces no new `unwrap()` or `expect()` calls in library code.

## Assumptions

- We assume standard thread-local RNG (`rand::rng()`) meets the performance and isolation requirements of concurrent transactions.
- We assume that resolving all registered scalar functions dynamically in PL/SQL is preferable to special-casing a hardcoded parser rule for random numbers.
