# Data Model: Rhai JSON Support

## Entity Alterations

There are no new database schema entities introduced with this feature. Instead, this feature introduces new type mapping rules between existing Data Models:

### 1. SQL `JSON` to Rhai `Dynamic` (Inbound)
When the Executor evaluates a query and binds arguments to a Rhai function, values of type `Value::Json` (which wrap a JSON-formatted `String`) must be converted.
- **Source**: `crate::core::Value::Json(String)`
- **Transformation**: `serde_json::from_str` -> `serde_json::Value` -> `rhai::serde::to_dynamic`
- **Destination**: `rhai::Dynamic` (typically representing a `rhai::Map` or `rhai::Array`)

### 2. Rhai `Dynamic` to SQL `JSON` (Outbound)
When a Rhai function completes and returns a value to the Executor, the return value must be converted back.
- **Source**: `rhai::Dynamic` (e.g., `rhai::Map`, `rhai::Array`)
- **Transformation**: `rhai::serde::from_dynamic` -> `serde_json::Value` -> `serde_json::to_string`
- **Destination**: `crate::core::Value::Json(String)`
- **Constraint**: This mapping is triggered if the SQL function signature declared its return type as `JSON`, or optionally, if the Rhai value dynamically reports as a Map or Array and dynamic type inference is active.

### 3. Built-in Function
- **Entity**: `parse_json(String)`
- **Namespace**: Available natively in Rhai, but typically we ensure it's available via standard language core packages (`rhai::packages::LanguageCorePackage`).
