# Phase 0: Research

## Research Topics

1. **PL/SQL JSON and TIMESTAMP Declarations**
2. **PL/SQL random() Function**
3. **Python Native Type Marshalling**

## Findings & Decisions

### 1. PL/SQL JSON and TIMESTAMP Declarations

- **Decision**: Update `PlSqlInterpreter::execute_statement` to correctly initialize variables declared as `JSON` or `TIMESTAMP` to `Value::Null(DataType::Json)` and `Value::Null(DataType::Timestamp)`.
- **Rationale**: Currently, `interpreter.rs` has a simple default initialization block that assigns `Value::Null(DataType::Null)` for unknown types, and only explicitly checks `INT`, `BOOL`, `TEXT`, `FLOAT`. We must add checks for `JSON` and `TIMESTAMP` strings in the data type.
- **Alternatives considered**: None. This is standard PL/SQL declaration behavior.

### 2. PL/SQL random() Function

- **Decision**: Update `PlSqlInterpreter::eval_expr` in the `FunctionCall` arm. Add an explicit check for `func_name == "random"`. Use `rand::rng().random::<f64>()` and return `Value::Float`.
- **Rationale**: This is how `get_http_header` is currently implemented. Since the PL/SQL parser handles generic function calls, the interpreter just needs to intercept it and generate the value, matching the Rhai and Python native intercepts.
- **Alternatives considered**: Implementing `random()` as a globally registered scalar function in `FunctionRegistry`. However, because `random()` is non-deterministic and often needs to bypass constant folding, injecting it at the interpreter level is safer for this embedded monolith pattern, just as Rhai/Python do.

### 3. Python Native Type Marshalling

- **Decision**: 
  - **Rust -> Python (`convert_oxibase_to_python`)**: 
    - For `Value::Json`: Parse the string using `serde_json::from_str`. Convert the resulting `serde_json::Value` into a `PyDict` or `PyList` using `rustpython_vm`.
    - For `Value::Timestamp`: Import the `datetime` module using `vm.import("datetime", 0)`, extract the `datetime` class, and call `fromisoformat()` with the RFC3339 string.
  - **Python -> Rust (`convert_python_to_oxibase`)**: 
    - For Dict/List: Check if `py_obj.class().name() == "dict"` or `"list"`. If so, use Python's `json.dumps()` (by importing the `json` module) to convert it to a JSON string, then store as `Value::Json(Arc::from(string))`.
    - For Datetime: Check if it's a datetime object. If so, call `.isoformat()`, parse the string back into `chrono::DateTime`, and store as `Value::Timestamp`.
- **Rationale**: Users expect native Python dictionaries and `datetime` objects. RustPython provides interop to call Python builtins (`json.dumps` and `datetime.fromisoformat`) which avoids manually traversing and building complex `PyDict` structures from Rust if we can leverage the VM. For `JSON` specifically, we can use a helper Python script executed in the VM to do the conversion quickly if needed, or just traverse the `serde_json::Value`. The most robust method for JSON in RustPython without heavy traversal is evaluating a Python string or traversing it. We will write a small converter in Rust using `rustpython_vm`'s `PyDict` API.
- **Alternatives considered**: Passing JSON as a raw string and requiring the user to `import json; json.loads()`. Rejected because it violates the P2 requirement for native marshalling.
