# Architecture & Research Findings: HTTP Headers Context for Python and PL/SQL

## 1. Python VM native module exposure for standard function execution

**Need**: Expose `get_http_header` in standard user-defined functions executed via `execute()` in `src/functions/backends/python.rs`.
**Decision**: Use `Interpreter::builder` instead of `Interpreter::with_init` inside `ScriptingBackend::execute` implementation.
**Rationale**: By using the custom `builder` and registering the `oxibase` native module definition, the `oxibase` module becomes importable and accessible within the Python UDF sandbox, maintaining identical behavior to procedures.

---

## 2. PL/SQL function evaluation mapping

**Need**: Evaluate `get_http_header` function inside the PL/SQL AST engine.
**Decision**:
1. Support `Expression::FunctionCall` within the `eval_expr` pattern match in `src/functions/plsql/interpreter.rs`.
2. Inspect `fc.function.to_lowercase() == "get_http_header"`.
3. Evaluate its single parameter argument recursively and extract the string value.
4. Access `crate::functions::context::HTTP_HEADERS` to perform a case-insensitive search and return the matched value, returning standard SQL NULL representation if missing or executed outside an HTTP context.
**Rationale**: This keeps expression evaluation robust and lightweight, allowing variables or expressions (such as variables containing header names) to be evaluated as the parameter argument of `get_http_header`.
