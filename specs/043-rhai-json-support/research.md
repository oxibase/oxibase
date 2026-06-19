# Research & Decisions: Rhai JSON Support

## Topic: Enabling Rhai JSON parsing

**Decision**: We will enable the `"metadata"` and `"serde"` features for the `rhai` dependency in `Cargo.toml`.

**Rationale**:
- The `"metadata"` feature automatically includes the `parse_json` built-in function in `Engine::new()`, removing the need to write and maintain a custom JSON parser binding.
- The `"serde"` feature provides `rhai::serde::to_dynamic` and `rhai::serde::from_dynamic`, which are crucial for seamlessly converting between our internal `Value::Json` (which uses `serde_json`) and native `rhai::Dynamic` maps and arrays.

**Alternatives considered**:
- Writing a manual recursive converter between `serde_json::Value` and `rhai::Dynamic`. This was rejected because it introduces unnecessary boilerplate and maintenance burden when upstream Rhai already provides robust serialization support via `serde`.

## Topic: Seamless JSON Argument & Return Handling

**Decision**: Update `src/functions/backends/rhai.rs` to intercept `Value::Json` parameters and `FunctionDataType::Json` return expectations.

**Rationale**:
- **Arguments**: Inside `value_to_dynamic` (and argument binding loops), if the incoming value is `Value::Json`, we parse its underlying string using `serde_json::from_str` into a `serde_json::Value` and use `rhai::serde::to_dynamic` to convert it into a Rhai object.
- **Returns**: Inside `dynamic_to_value` (and return extraction), if the expected data type is `DataType::Json` (or the Rhai value is a map/array), we convert the `rhai::Dynamic` to `serde_json::Value` via `rhai::serde::from_dynamic`, serialize it to string, and return it as a `Value::Json`.

**Alternatives considered**:
- Forcing users to always call `parse_json()` on string arguments and `to_json()` on objects before returning. Rejected because it contradicts User Story 3 and 4, which explicitly request native handling for ergonomics.
