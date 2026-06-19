# Feature Specification: Rhai JSON Support

**Feature Branch**: `043-rhai-json-support`  
**Created**: 2026-06-18  
**Status**: Draft  
**Input**: User description: "add json support to rhai"

## Clarifications

### Session 2026-06-18
- Q: Clarify user intent regarding JSON arguments and returns → A: Functions and procedures should natively accept JSON arguments (mapped to Rhai objects) and be able to return Rhai objects as JSON.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Parse JSON string in Rhai script (Priority: P1)

As a database user writing Rhai scripts for functions or triggers, I want to be able to parse a JSON string into a manipulable object map or array so that I can easily extract data from JSON payloads stored in the database.

**Why this priority**: Parsing JSON is the core requirement of adding JSON support. It allows users to work with structured text data natively within their scripts.

**Independent Test**: Can be fully tested by a new integration test in `tests/rhai_scripting_test.rs` that creates a function which parses a JSON string and returns an extracted value.

**Acceptance Scenarios**:

1. **Given** a Rhai script function that calls `parse_json('{"key": 42}')`, **When** the function is executed, **Then** it successfully parses the string and allows accessing the `key` property, returning `42`.
2. **Given** a Rhai script function that calls `parse_json('[1, 2, 3]')`, **When** the function is executed, **Then** it parses it as an array and allows indexing into it.

---

### User Story 2 - Handle Invalid JSON Gracefully (Priority: P2)

As a database user writing Rhai scripts, I want the system to handle invalid JSON strings gracefully by returning an error rather than crashing the database or script engine.

**Why this priority**: Robustness is key for database operations. Bad data shouldn't bring down the system.

**Independent Test**: Can be fully tested by an integration test that attempts to parse malformed JSON and ensures a clean error is returned.

**Acceptance Scenarios**:

1. **Given** a Rhai script function that calls `parse_json('{invalid json}')`, **When** the function is executed, **Then** it returns a descriptive error and does not panic or crash.

### User Story 3 - Pass JSON to Rhai functions natively (Priority: P1)

As a database user, I want to define a Rhai function that takes a JSON string/object as an argument, so that when I call the SQL function with JSON data, the script natively receives it as a manipulable Rhai dynamic object without manual parsing.

**Why this priority**: Required by the user to support transparent JSON argument passing.

**Independent Test**: Can be tested by creating a SQL function taking a JSON parameter and returning a specific field from it.

**Acceptance Scenarios**:

1. **Given** a Rhai script function that takes an argument `doc`, **When** called via SQL `SELECT my_func('{"key": 42}'::JSON)`, **Then** the Rhai variable `doc` is a map where `doc.key == 42`.

---

### User Story 4 - Return Rhai objects as JSON (Priority: P1)

As a database user, I want a Rhai function to be able to return a map or array, and have it automatically converted into a SQL JSON value, so I don't have to manually format JSON strings in my scripts.

**Why this priority**: Required by the user to support transparent JSON returning.

**Independent Test**: Can be tested by returning `#{a: 1}` from Rhai and asserting the SQL result is a JSON value `{"a": 1}`.

**Acceptance Scenarios**:

1. **Given** a Rhai script that returns `#{key: 42}`, **When** executed, **Then** the SQL engine receives a JSON type containing `{"key": 42}`.

### Edge Cases

- What happens when a null value is parsed (`parse_json('null')`)? It should translate to Rhai's unit type `()`.
- What happens when deeply nested JSON is parsed? It should be fully traversable in the Rhai script.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Engine MUST provide a `parse_json` function accessible to Rhai scripts that takes a string argument and returns a dynamic Rhai value (Map, Array, or primitive).
- **FR-002**: Engine MUST handle standard JSON types correctly, mapping JSON objects to Rhai maps and JSON arrays to Rhai arrays.
- **FR-003**: System MUST return an evaluable error (not a panic) when `parse_json` is provided with an invalid JSON string.

- **FR-004**: The backend executor MUST convert database `JSON` type arguments into native Rhai `Dynamic` maps/arrays/primitives before executing the script.
- **FR-005**: The backend executor MUST convert returned Rhai `Dynamic` maps/arrays back into the database `JSON` type when the function signature specifies a JSON return type.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully parse a valid JSON string containing nested objects/arrays in a Rhai script and extract a leaf value.
- **SC-002**: Scripts attempting to parse invalid JSON return a catchable script error, preventing any database panics.
- **SC-003**: Passes all new and existing test suites.

## Assumptions

- No custom JSON serialization (Rhai object to JSON string) is strictly required for this specific feature request, as the focus is on "adding support" generally interpreted as parsing incoming JSON payloads. Note: the user clarified they DO want seamless passing/returning of JSON, which implies conversion logic must be added to the parameter binding and return value extraction phases of `src/functions/backends/rhai.rs`.