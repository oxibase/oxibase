# Feature Specification: Scripting Backend Parity

**Feature Branch**: `049-scripting-backend-parity`  
**Created**: 2026-06-20  
**Status**: Draft  
**Input**: User description: "the action items" (Based on the previous audit report identifying missing features in PL/SQL and Python backends compared to Rhai).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - JSON and Timestamp in PL/SQL (Priority: P1)

As a database user writing PL/SQL procedures and triggers, I need to be able to declare, assign, and manipulate variables of type `JSON` and `TIMESTAMP` natively, so that my PL/SQL code can interact with modern data types just like Rhai and Python do.

**Why this priority**: PL/SQL is a core backend, and missing support for these fundamental data types breaks parity and prevents users from fully utilizing the database's capabilities within PL/SQL.

**Independent Test**: Can be fully tested by adding new tests in `tests/procedure_plsql_tests.rs` that declare `JSON` and `TIMESTAMP` variables, assign values to them, and verify their outputs.

**Acceptance Scenarios**:

1. **Given** a PL/SQL block with a `DECLARE v_json JSON;` statement, **When** assigning a valid JSON value and retrieving it, **Then** the value is correctly stored and retrieved as a JSON type.
2. **Given** a PL/SQL block with a `DECLARE v_ts TIMESTAMP;` statement, **When** assigning a valid timestamp value, **Then** the value is correctly stored and retrieved as a Timestamp type.

---

### User Story 2 - Random Number Generation in PL/SQL (Priority: P1)

As a database user writing PL/SQL procedures, I need to generate random numbers using a `random()` function, so that I can implement randomized logic, matching the capabilities already present in Rhai and Python backends.

**Why this priority**: Feature parity. The function exists in other backends and is commonly needed.

**Independent Test**: Can be fully tested by adding a new test in `tests/procedure_plsql_tests.rs` that calls `random()` and verifies it returns a valid Float value.

**Acceptance Scenarios**:

1. **Given** a PL/SQL procedure, **When** calling `random()`, **Then** the execution succeeds and returns a Float between 0.0 and 1.0.

---

### User Story 3 - Python Native Type Marshalling for JSON and Timestamp (Priority: P2)

As a database user writing Python functions, I need JSON payloads passed as arguments to be translated into native Python `dict` (or `list`) objects, and `TIMESTAMP` values translated into native Python `datetime` objects, so that I don't have to manually parse strings within my Python code.

**Why this priority**: While Python currently works via string fallbacks, native marshalling significantly improves developer experience and aligns with how Rhai handles these types (converting JSON to maps).

**Independent Test**: Can be fully tested by adding tests in `tests/python_scripting_test.rs` that pass `JSON` and `TIMESTAMP` arguments to Python functions and verify they are received as `dict`/`datetime` types.

**Acceptance Scenarios**:

1. **Given** a Python function accepting a JSON argument, **When** the function is called with `{"key": "value"}`, **Then** inside the Python function, `type(arguments[0])` is `dict` and `arguments[0]["key"]` equals `"value"`.
2. **Given** a Python function accepting a TIMESTAMP argument, **When** called with a valid timestamp, **Then** inside the Python function, the argument is a valid Python `datetime` object.

---

### Edge Cases

- What happens when invalid JSON strings are passed from Python back to Oxibase? (Should fallback to string or error).
- What happens when a Python function returns an unsupported object type?
- How does PL/SQL handle coercion between incompatible types (e.g., assigning a JSON string to an Integer variable)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The PL/SQL Interpreter (`src/functions/plsql/interpreter.rs`) MUST correctly parse and assign literal values for `JSON` and `TIMESTAMP` types.
- **FR-002**: The PL/SQL Interpreter MUST support evaluating the `random()` function call, returning a random `Float`.
- **FR-003**: The Python Backend (`src/functions/backends/python.rs`) MUST serialize Oxibase `Value::Json` objects into Python dictionaries or lists when passing them as arguments to Python functions.
- **FR-004**: The Python Backend MUST serialize Oxibase `Value::Timestamp` objects into Python `datetime` objects when passing them as arguments to Python functions.
- **FR-005**: The Python Backend MUST be able to deserialize Python dictionaries/lists back into Oxibase `Value::Json` objects.
- **FR-006**: Test coverage for these new capabilities MUST be added to `tests/procedure_plsql_tests.rs` and `tests/python_scripting_test.rs`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All new and existing tests in `make test` pass successfully.
- **SC-002**: Feature parity is achieved: `JSON`, `TIMESTAMP`, and `random()` functionalities work identically (from the user's perspective) across Rhai, Python, and PL/SQL backends.
- **SC-003**: Test parity is achieved: The PL/SQL and Python test suites contain equivalents for the related tests found in `tests/rhai_scripting_test.rs`.
- **SC-004**: Passes `make lint` without warnings.

## Assumptions

- We assume `rustpython_vm` provides adequate utilities for constructing dictionaries and `datetime` objects from Rust.
- We assume the existing parser infrastructure for PL/SQL (`src/functions/plsql/parser.rs`) already handles `JSON` and `TIMESTAMP` keywords sufficiently, and only the interpreter needs updates.
