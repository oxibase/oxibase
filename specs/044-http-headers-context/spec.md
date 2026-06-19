# Feature Specification: HTTP Headers Context for Python and PL/SQL

**Feature Branch**: `044-http-headers-context`  
**Created**: June 19, 2026  
**Status**: Draft  
**Input**: User description: "Implement HTTP Headers Context for Python and PL/SQL backends matching the existing Rhai behavior."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Python HTTP Header Retrieval (Priority: P1)

As a developer building web-aware applications, I want to retrieve HTTP request headers from within my Python stored procedures and user-defined functions so that I can handle request-specific context (like Authorization tokens) in Python.

**Why this priority**: Python is a major scripting engine in the database; full service parity requires accessing HTTP context just like Rhai.

**Independent Test**: Create a Python procedure/function that fetches an HTTP header and returns/assigns it, then execute it within an active HTTP context to verify.

**Acceptance Scenarios**:

1. **Given** a Python procedure `check_auth_py(OUT token TEXT)` defined as `import oxibase; token = oxibase.get_http_header("Authorization")`, **When** the procedure is invoked via HTTP with header `Authorization: Bearer test123`, **Then** the output parameter `token` contains `"Bearer test123"`.
2. **Given** a Python function/procedure, **When** calling `oxibase.get_http_header` with a non-existent header name, **Then** it returns `None`.

---

### User Story 2 - PL/SQL HTTP Header Retrieval (Priority: P1)

As a developer writing database business logic, I want to retrieve HTTP request headers from within my PL/SQL stored procedures and functions so that I can handle request-specific context directly in native PL/SQL.

**Why this priority**: PL/SQL is the native language clone of PL/pgSQL; exposing HTTP request metadata allows it to function as a backend web router or validation layer.

**Independent Test**: Create a PL/SQL procedure that fetches an HTTP header and assigns it to an OUT parameter, then execute it within an active HTTP context to verify.

**Acceptance Scenarios**:

1. **Given** a PL/SQL procedure `check_auth_plsql(OUT token TEXT)` defined as `BEGIN token := get_http_header('Authorization'); END;`, **When** the procedure is invoked via HTTP with header `Authorization: Bearer test123`, **Then** the output parameter `token` contains `"Bearer test123"`.
2. **Given** a PL/SQL procedure, **When** calling `get_http_header` with a non-existent header name, **Then** it returns `NULL`.

---

### Edge Cases

- **Non-HTTP Execution Context**: When UDFs or procedures are invoked outside of an HTTP RPC call (such as standard CLI or test SQL queries), calling `get_http_header` MUST return `None`/`NULL` rather than throwing an error or panicking.
- **Header Case-Sensitivity**: HTTP headers are case-insensitive. Lookups like `get_http_header('authorization')`, `get_http_header('Authorization')`, and `get_http_header('AUTHORIZATION')` MUST return the same header value if present.
- **Null and Empty Values**: Empty headers or missing headers must be translated to the natural null representation of each scripting language (`None` in Python, `NULL` in PL/SQL).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Python scripting environment (both standard scalar function `execute` and procedural `execute_procedure`) MUST expose the native module `oxibase` containing `get_http_header(name)`.
- **FR-002**: The Python `oxibase.get_http_header` implementation MUST lookup the header value case-insensitively in `crate::functions::context::HTTP_HEADERS`.
- **FR-003**: The PL/SQL expression evaluation (`eval_expr` in `src/functions/plsql/interpreter.rs`) MUST support evaluating `Expression::FunctionCall` for functions named `get_http_header`.
- **FR-004**: The PL/SQL `get_http_header` function call MUST accept exactly 1 argument which evaluates to a string.
- **FR-005**: The PL/SQL `get_http_header` implementation MUST lookup the header value case-insensitively in `crate::functions::context::HTTP_HEADERS`.
- **FR-006**: Both Python and PL/SQL implementations MUST safely return their language-native null representation if the header does not exist, or if the thread-local context is empty.

### Key Entities

- **Python Native Module**: The `oxibase` Rust-defined Python module which registers `get_http_header` under RustPython.
- **PL/SQL Interpreter (`PlSqlInterpreter`)**: The interpreter that evaluates PL/SQL expressions and executes AST nodes.
- **Thread-Local Context (`HTTP_HEADERS`)**: The static `RefCell<Option<HashMap<String, String>>>` in `crate::functions::context` that holds HTTP headers during execution.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Verification via integration tests confirming that a procedure written in Python can retrieve the "Authorization" HTTP header via RPC context.
- **SC-002**: Verification via integration tests confirming that a procedure written in PL/SQL can retrieve the "Authorization" HTTP header via RPC context.
- **SC-003**: Testing confirms that case-insensitive searches (e.g. `get_http_header('authorization')`) work correctly in both backends.
- **SC-004**: Non-HTTP execution environments cleanly return null (`None` / `NULL`) without raising execution or runtime errors.

## Assumptions

- We assume that `crate::functions::context::HTTP_HEADERS` is correctly populated by the server routing middleware for all HTTP incoming requests, which is already verified by Rhai integration tests.
