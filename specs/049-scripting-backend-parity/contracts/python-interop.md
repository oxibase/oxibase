# Python Interop Contracts

## JSON Mapping

- **Rust to Python**: `oxibase::core::Value::Json(Arc<str>)` -> parsed to `serde_json::Value` -> translated recursively to `rustpython_vm` `PyDict`, `PyList`, `PyInt`, `PyFloat`, `PyString`, `PyBool`, `None`.
- **Python to Rust**: `PyDict` or `PyList` -> serialized via Python's `json.dumps` (using `rustpython_vm` standard library `json` module) -> stored as `oxibase::core::Value::Json(Arc<str>)`.

## Timestamp Mapping

- **Rust to Python**: `oxibase::core::Value::Timestamp(chrono::DateTime<Utc>)` -> formatted to ISO 8601 string -> parsed in Python using `datetime.fromisoformat`.
- **Python to Rust**: `datetime` object -> formatted to ISO string using `.isoformat()` -> parsed in Rust using `chrono::DateTime::parse_from_rfc3339` -> stored as `oxibase::core::Value::Timestamp`.
