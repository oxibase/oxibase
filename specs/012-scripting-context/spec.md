# Feature Specification: Scripting Backend Context Refactor (oxibase.ctx)

**Feature Branch**: `012-scripting-context`  
**Created**: May 13 2026  
**Status**: Draft  
**Input**: "in javascript, exposing OLD, and NEW looks off, same as in JS. can you think of a more 'natural' way ? maybe somethign like import oxibase; oxibase.ctx.new; oxibase.ctx.old; similar with JS ?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Python Context Refactor (Priority: P1)

Developers writing triggers in Python need an idiomatic way to access trigger context rather than relying on magically injected global variables, which pollute the namespace and feel unnatural.

**Why this priority**: Python is a core scripting backend and heavily utilized for triggers.

**Independent Test**: Can be fully tested by updating the Python trigger integration tests in `tests/triggers_test.rs` to use `oxibase.ctx.old` and `oxibase.ctx.new`, ensuring data validation and transformation still work correctly.

**Acceptance Scenarios**:

1. **Given** a Python `BEFORE UPDATE` trigger using `oxibase.ctx.new`, **When** the trigger mutates the new row via `oxibase.ctx.new["balance"] = 100`, **Then** the updated value is successfully extracted and saved to the database.
2. **Given** a Python `AFTER UPDATE` trigger, **When** the script accesses `oxibase.ctx.old["id"]`, **Then** it correctly resolves the integer ID of the record before the update.

---

### User Story 2 - JavaScript Context Refactor (Priority: P1)

Developers writing triggers in JavaScript need an idiomatic way to access trigger context rather than relying on magically injected global variables.

**Why this priority**: JavaScript is a core scripting backend alongside Python.

**Independent Test**: Can be fully tested by updating the JS trigger integration tests in `tests/triggers_test.rs`.

**Acceptance Scenarios**:

1. **Given** a JavaScript `BEFORE UPDATE` trigger, **When** the script mutates `oxibase.ctx.new.balance = 100`, **Then** the updated value is correctly saved to the database.
2. **Given** a JavaScript `AFTER UPDATE` trigger, **When** the script accesses `oxibase.ctx.old.balance`, **Then** it successfully retrieves the balance value from the previous state.

---

### User Story 3 - Rhai Context Refactor (Priority: P2)

Developers writing triggers in Rhai should use the same idiomatic `oxibase.ctx` API as Python and JS for consistency across the engine, removing the `OLD` and `NEW` global variables from the evaluation scope.

**Why this priority**: Rhai is the default, embedded scripting engine. It must remain feature-parity consistent with JS and Python.

**Independent Test**: Can be fully tested by updating the Rhai trigger tests in `tests/triggers_test.rs`.

**Acceptance Scenarios**:

1. **Given** a Rhai `BEFORE INSERT` trigger, **When** it modifies `oxibase.ctx.new.name`, **Then** the Rhai dynamic proxy mutates the underlying memory correctly and persists to storage.

### Edge Cases

- What happens if the trigger explicitly attempts to reassign the entire `ctx` or `new` object instead of its properties (e.g., `oxibase.ctx.new = {"foo": 1}`)? 
  - *Mitigation*: The engine extraction phase must safely ignore this or fail gracefully, as it expects to extract fields by iterating the known table schema against the root `new` dictionary.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Python scripting backend MUST inject an `oxibase` module containing a `ctx` object, which in turn houses `old` and `new` row dictionaries.
- **FR-002**: The Boa scripting backend MUST inject a global `oxibase` object containing a `ctx` object, which houses the `old` and `new` JS objects.
- **FR-003**: The Rhai scripting backend MUST inject an `oxibase` Map containing a `ctx` Map, which exposes the `OldRowProxy` and `NewRowProxy`.
- **FR-004**: System MUST NOT inject `OLD` and `NEW` directly into the global execution scope for any of the backends.
- **FR-005**: Trigger `new` row extraction logic MUST read from the deeply nested `oxibase.ctx.new` path instead of the global namespace.
- **FR-006**: All embedded tutorial markdown examples and Rust integration tests MUST be updated to utilize the new syntax.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Passes all modified triggers tests via `cargo nextest run --features js,python --test triggers_test`.
- **SC-002**: Passes the tutorial compilation test via `cargo nextest run --features js,python --test tutorial_triggers_test`.
- **SC-003**: Passes `make lint` without warnings, ensuring no dead code remains from the legacy `OLD`/`NEW` extraction paths.

## Assumptions

- **Assumption**: Breaking existing trigger code is acceptable and intended for this release.
- **Assumption**: The underlying proxy memory implementations (`NewRowProxy`, PyDicts, JS Objects) do not require architectural changes; only their scoping/injection paths require modification.
