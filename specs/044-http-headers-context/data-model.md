# Design & Context: HTTP Headers Context for Python and PL/SQL

## 1. Context Interface

This feature integrates existing HTTP headers context with Python and PL/SQL backends without modifying the database schema.

### Thread-Local State (Existing)
The thread-local variable defined in `src/functions/context.rs` is:
```rust
thread_local! {
    pub static HTTP_HEADERS: RefCell<Option<HashMap<String, String>>> = const { RefCell::new(None) };
}
```

---

## 2. Scripting Integration

### Python VM Integration
- **Module Name**: `oxibase`
- **Function**: `get_http_header`
- **Access**: In `src/functions/backends/python.rs`, register the `oxibase` module inside the Python interpreter for standard function executions (`execute`) as well as procedural executions (`execute_procedure`).
- **Internal Mapping**:
  ```rust
  crate::functions::context::HTTP_HEADERS.with(|headers| {
      // Perform case-insensitive search on keys and return the value
  })
  ```

### PL/SQL Interpreter Integration
- **Component**: `PlSqlInterpreter` in `src/functions/plsql/interpreter.rs`
- **Expression Evaluation**: Extend `eval_expr` to match `Expression::FunctionCall(fc)`.
- **Lookup Routing**: If function is `get_http_header`, evaluate its first argument, check if it's a string, and search case-insensitively in `crate::functions::context::HTTP_HEADERS`.
