# Implementation Tasks: Rhai JSON Support

**Feature**: Rhai JSON Support
**Branch**: `044-rhai-json-support`
**Status**: Pending

## Phase 1: Setup

Goal: Update `Cargo.toml` dependencies and features to allow Rhai JSON serialization and metadata exposure.

- [ ] T001 Update `Cargo.toml` to add `metadata` and `serde` features to the `rhai` dependency.

## Phase 2: User Story 1 - Parse JSON string in Rhai script (Priority: P1) & User Story 2 - Handle Invalid JSON Gracefully (Priority: P2)

Goal: Enable the `parse_json` native function in the Rhai engine so scripts can directly invoke it to parse strings into dynamic objects.

**Independent Test**:
- Create a test `test_rhai_parse_json` in `tests/rhai_scripting_test.rs` which executes a Rhai script calling `parse_json('{"key": 42}')` and extracts the value `42`.
- Create a test `test_rhai_parse_invalid_json` ensuring an error is returned when passing invalid JSON to `parse_json`.

- [ ] T002 [P] [US1] Write test `test_rhai_parse_json` in `tests/rhai_scripting_test.rs`.
- [ ] T003 [P] [US2] Write test `test_rhai_parse_invalid_json` in `tests/rhai_scripting_test.rs`.
- [ ] T004 [US1] Modify `src/functions/backends/rhai.rs` to register the `parse_json` functionality if it requires manual exposure, or confirm `Engine::new()` via `metadata` feature automatically brings it into the `oxibase` namespace or global scope. Actually, per research, since we use `Engine::new()`, `metadata` feature will auto-include it in `LanguageCorePackage` globally. We just need to ensure the engine exposes it properly.

## Phase 3: User Story 3 - Pass JSON to Rhai functions natively (Priority: P1)

Goal: When a SQL function signature accepts JSON, the executor passes `Value::Json(String)`. `src/functions/backends/rhai.rs` must seamlessly convert this into a `rhai::Dynamic` map/array using `serde_json` and `rhai::serde::to_dynamic`.

**Independent Test**:
- Create a test `test_rhai_json_arguments` in `tests/rhai_scripting_test.rs` defining a function that takes a `JSON` argument and returns an extracted property to prove it was received as a Map.

- [ ] T005 [P] [US3] Write test `test_rhai_json_arguments` in `tests/rhai_scripting_test.rs`.
- [ ] T006 [US3] Update `value_to_dynamic` in `src/functions/backends/rhai.rs` to map `crate::core::Value::Json(s)` to a `rhai::Dynamic` using `serde_json::from_str` and `rhai::serde::to_dynamic`. Add handling inside `execute` and `execute_procedure` where arguments are bound.

## Phase 4: User Story 4 - Return Rhai objects as JSON (Priority: P1)

Goal: When a Rhai function returns a `rhai::Dynamic` that is a Map or Array, and the SQL return type is `JSON`, seamlessly serialize it back to `Value::Json`.

**Independent Test**:
- Create a test `test_rhai_json_returns` in `tests/rhai_scripting_test.rs` verifying returning a Rhai Map (e.g. `#{a: 1}`) produces a correct SQL JSON value.

- [ ] T007 [P] [US4] Write test `test_rhai_json_returns` in `tests/rhai_scripting_test.rs`.
- [ ] T008 [US4] Update `dynamic_to_value` in `src/functions/backends/rhai.rs` to handle returning `DataType::Json`. Convert the `rhai::Dynamic` back to a `serde_json::Value` using `rhai::serde::from_dynamic`, then serialize it to `String` and wrap in `Value::Json`.

## Phase 5: Polish & Cross-Cutting Concerns

Goal: Ensure everything is clean, idiomatic, and documented.

- [ ] T009 Ensure all error scenarios inside the conversion logic return valid `Error::internal` instead of panicking.
- [ ] T010 Run `make lint` and `cargo nextest run` to ensure tests pass and code styling conforms to Oxibase standards.

## Dependencies

- Phase 1 (Setup) is a prerequisite for all other phases since it changes `Cargo.toml`.
- Phase 2 (Global `parse_json`) is independent of Phase 3 and Phase 4.
- Phase 3 (Arguments) and Phase 4 (Returns) can be developed in parallel after Setup.
- Tests (T002, T003, T005, T007) can be written concurrently before their respective implementations.

## Implementation Strategy

We will follow TDD (Tests First). We will first write the integration tests for US1, US2, US3, and US4 in `tests/rhai_scripting_test.rs`. Then, we will add the `serde` and `metadata` features to `Cargo.toml`. Finally, we will implement the conversion logic inside `src/functions/backends/rhai.rs`'s `value_to_dynamic` and `dynamic_to_value` functions to make all tests pass.