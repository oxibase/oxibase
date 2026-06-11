# Research: DAP Support for PL/SQL Procedures

## 1. Line Number and Source Mapping in AST
**Decision**: Add a `span` or `line_number` to `PlSqlStatement` variants, or wrap it in a struct `SpannedStatement`. We will add a `line_number: usize` or `token: Token` field to statements that don't have it (like `Assignment`, `If`, `While`) to ensure the interpreter knows the current line before execution.
**Rationale**: DAP needs to know exactly which line is about to be executed to determine if a breakpoint is hit. The parser must inject this metadata.
**Alternatives considered**: Wrapping the entire `PlSqlStatement` enum in a `struct Spanned<T> { inner: T, line: usize }` which is cleaner but requires more refactoring. We will go with adding a `line_number: usize` directly to the structures for simplicity, or storing `Token` which has line info.

## 2. Interpreter Hooking for Breakpoints
**Decision**: Inject a `DebugContext` or hook closure into `PlSqlInterpreter::new()`. Within `PlSqlInterpreter::evaluate_statement()`, call `debug_context.on_statement(line_number, current_environment)` before executing the statement logic.
**Rationale**: The interpreter is a loop/recursive evaluator. We need a way to block the thread if a breakpoint is hit. The `DebugController` (from task #33) will provide this blocking channel mechanism.
**Alternatives considered**: Having the interpreter yield back to the caller (state machine). This is too complex. Blocking the current execution thread is standard for synchronous evaluators during debugging.

## 3. Mapping Environment to DAP Variables
**Decision**: Implement a mapping from PL/SQL `Environment` (which holds scopes and variables) to DAP `Variable` objects. The `DebugContext` hook will pass the `Environment` reference.
**Rationale**: DAP requires converting internal representation (e.g., Oxibase `Value`s) into stringified standard formats. We need a utility function `env_to_dap_scopes(env: &Environment) -> Vec<DapScope>`.

## 4. DebugController Integration (Dependency on #33)
**Decision**: We will define a trait `PlSqlDebugHook` or depend directly on the upcoming `DebugController` API. Since `#33` introduces the DAP server, we will design our interpreter hook to be generic enough (e.g., passing `Box<dyn DebugAdapterHook>`) or integrate directly if the controller is already in the codebase.
**Rationale**: Keeps PL/SQL decoupled from the raw TCP DAP implementation.
