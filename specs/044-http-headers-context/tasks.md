# Tasks: HTTP Headers Context for Python and PL/SQL

**Input**: Design documents from `specs/044-http-headers-context/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

**Tests**: Must run `cargo nextest run --features python` to verify Python feature tests and standard test suites.

## Phase 1: Python HTTP Header Context (Priority: P1)

**Goal**: Expose standard `get_http_header` inside Python Virtual Machine so Python procedures and UDFs can read HTTP headers.

- [X] T001 [US1] Register native `oxibase` module using `Interpreter::builder` in `execute` function of `src/functions/backends/python.rs`.
- [X] T002 [US1] Create integration test in `tests/server_rpc_tests.rs` defining Python procedure calling `oxibase.get_http_header('Authorization')` and verifying successful invocation.

---

## Phase 2: PL/SQL HTTP Header Context (Priority: P1)

**Goal**: Support evaluating `get_http_header` within the PL/SQL expression evaluation engine.

- [X] T010 [US2] Implement `Expression::FunctionCall` pattern matching inside `eval_expr` in `src/functions/plsql/interpreter.rs`.
- [X] T011 [US2] If function name is `get_http_header`, validate arguments length, recursively evaluate argument 0 to extract header name as string, perform case-insensitive search in `crate::functions::context::HTTP_HEADERS`, and return matched string as `Value::Text(...)` or `Value::Null(crate::core::DataType::Null)` if missing.
- [X] T012 [US2] Create integration test in `tests/server_rpc_tests.rs` defining PL/SQL procedure calling `get_http_header('Authorization')` and verifying successful invocation.

---

## Phase 3: Edge Cases & Validation (Priority: P2)

**Goal**: Verify lookups are case-insensitive and handle empty/non-existent headers cleanly.

- [X] T020 [US1] Add test verifying case-insensitive header retrieval in both Python and PL/SQL backends (e.g., retrieving `authorization` when passed `Authorization`).
- [X] T021 [US2] Add test verifying standard execution of both Python and PL/SQL `get_http_header` outside HTTP RPC context (returns `None` and `NULL` cleanly).
