# Research: Stdout Interception for Rhai and PL/SQL

## 1. Rhai `print` Hook Integration
- **Decision**: Register the `Engine::on_print` hook in `RhaiBackend::new()` to forward output to `crate::functions::context::append_stdout`.
- **Rationale**: The Rhai engine provides a native mechanism to intercept `print` statements via its `on_print` hook. Since oxibase already maintains an execution context with an output buffer (`append_stdout`), passing the string directly from the hook perfectly matches the requirement without any overhead or complex modifications.
- **Alternatives considered**: None, this is the canonical and idiomatic way to capture output from a Rhai engine.

## 2. PL/SQL Parser Extensions
- **Decision**: Add a new `Print(Token, Expression)` variant to the `PlSqlStatement` enum in the PL/SQL AST. The parser will be updated to handle `PRINT` and `RAISE NOTICE` keywords.
- **Rationale**: Both `PRINT` and `RAISE NOTICE` are intended to evaluate a single expression and output its value. Mapping both statements to a single `Print` AST node simplifies the interpreter logic. We will parse the expression and expect a terminating semicolon.
- **Alternatives considered**: Creating separate AST nodes for `PRINT` and `RAISE NOTICE`. This was rejected because they serve the exact same semantic purpose in this context, so a single shared AST node is cleaner.

## 3. PL/SQL Interpreter Support
- **Decision**: Implement handling for `PlSqlStatement::Print` in `PlSqlInterpreter::execute_statement`. It will evaluate the expression using `eval_expr` and pass the resulting string to `crate::functions::context::append_stdout`.
- **Rationale**: By evaluating the expression at runtime, we can correctly print variables and evaluated literals. This aligns with standard PL/SQL implementations.
- **Alternatives considered**: Passing the raw expression string instead of evaluating it. Rejected because the requirement implies evaluating expressions (e.g., `PRINT 1 + 2;` should output `3`).