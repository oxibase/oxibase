# Feature Specification: Service Invocation via HTTP

**Feature Branch**: `009-service-invocation`  
**Created**: May 11, 2026  
**Status**: Draft  
**Input**: User description: "Service invocation feature (e.g., exposing stored procedures through the HTTP server)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Invoke a Stored Procedure via HTTP POST (Priority: P1)

As a developer building a web application, I want to invoke a stored procedure by sending an HTTP POST request to a specific endpoint so that I can trigger database logic directly from my application without writing custom middleware.

**Why this priority**: Exposing stored procedures via HTTP is the core of "Service Invocation," turning the database into an application backend.

**Independent Test**: Can be tested by creating a simple procedure, starting the HTTP server, and using a local HTTP client to `POST` to the invocation endpoint and verify the JSON response contains the procedure's output.

**Acceptance Scenarios**:

1. **Given** a stored procedure named `calculate_tax` that takes `amount` (INTEGER) and returns INTEGER, **When** I send a `POST` request to `/api/rpc/calculate_tax` with JSON body `{"amount": 100}`, **Then** the server responds with HTTP 200 and a JSON body containing the result (e.g., `{"result": 108}`).
2. **Given** a stored procedure that takes no arguments, **When** I send a `POST` request to its endpoint with an empty body, **Then** the procedure executes successfully and returns HTTP 200.

---

### User Story 2 - Handle Procedure Errors Gracefully (Priority: P1)

As an API consumer, I want clear HTTP error responses when a procedure invocation fails so that my application can handle errors (like invalid arguments or business logic exceptions) appropriately.

**Why this priority**: Robust error handling is critical for any HTTP API, mapping database/scripting errors to standard web semantics.

**Independent Test**: Can be tested by sending requests with missing arguments, wrong data types, or to non-existent procedures, verifying the HTTP status codes and error payloads.

**Acceptance Scenarios**:

1. **Given** an active HTTP server, **When** I send a `POST` request to `/api/rpc/non_existent_proc`, **Then** the server responds with HTTP 404 Not Found and a JSON error message.
2. **Given** a procedure requiring an `amount` parameter, **When** I send a request missing this parameter or with the wrong type (e.g., `"amount": "abc"`), **Then** the server responds with HTTP 400 Bad Request and details about the parameter mismatch.
3. **Given** a procedure that throws a runtime error (e.g., division by zero), **When** I invoke it, **Then** the server responds with HTTP 500 Internal Server Error (or appropriate code) and the exception message.

---

### User Story 3 - Pass Context Variables via HTTP Headers (Priority: P2)

As a developer, I want HTTP request metadata (like the Authorization header or client IP) to be accessible inside the stored procedure so that I can implement custom authorization or auditing logic.

**Why this priority**: True "services" often need to authenticate requests or perform logic based on the user's token or session.

**Independent Test**: Create a procedure that returns the value of an HTTP header. Send a request with that header and assert the response matches.

**Acceptance Scenarios**:

1. **Given** a procedure that reads the `Authorization` header, **When** I invoke the procedure via HTTP passing `Authorization: Bearer xyz`, **Then** the procedure successfully reads `Bearer xyz` and returns it or uses it for logic.

### Edge Cases

- What happens if two procedures exist with the same name but different parameter signatures (if overloading is supported)?
- How does the system handle concurrent HTTP requests invoking the same procedure (MVCC context)?
- How are complex JSON payloads mapped to procedure arguments (e.g., passing a JSON object to a procedure expecting a `JSON` type)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The HTTP server MUST expose a new routing pattern (e.g., `/api/rpc/:procedure_name`) that accepts `POST` requests.
- **FR-002**: The route handler MUST extract the `procedure_name` from the URL path, look it up in the function registry, and map the JSON request body payload to the procedure's expected parameters.
- **FR-003**: The handler MUST invoke the procedure using the existing execution engine logic.
- **FR-004**: The system MUST return the procedure's output as a JSON-formatted HTTP response with a `200 OK` status on success.
- **FR-005**: The system MUST return appropriate HTTP error codes: `404` for missing procedures, `400` for parameter mismatches or invalid JSON, and `500` for runtime execution errors.
- **FR-006**: The system MUST provide a mechanism to pass HTTP request context (headers) into the execution environment via a special built-in SQL function (e.g., `get_http_header('Header-Name')`) that returns the header value if the procedure was invoked via HTTP, or NULL otherwise.

### Key Entities 

- **HTTP Route Handler**: The entry point that receives the web request and parses the JSON body.
- **Function/Procedure Registry**: Used to look up the procedure and validate the expected parameters against the incoming JSON keys.
- **Execution Engine**: The context required to actually execute the procedure and bridge to the database engine.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Integration tests demonstrate successful HTTP POST invocations of procedures across all supported scripting languages, returning the correct JSON output.
- **SC-002**: The server handles parameter validation failures by returning HTTP 400 without crashing the database process.
- **SC-003**: The implementation introduces zero new `unwrap()` or `expect()` calls in the request path, adhering strictly to the `Result`-based error handling standard.
- **SC-004**: Execution latency for a simple procedural HTTP call adds less than 5ms overhead compared to native SQL `CALL` execution.

## Assumptions

- We assume the JSON payload keys will exactly match the named parameters of the stored procedure.
- We assume procedures invoked via HTTP will automatically run in an implicit transaction (following the standard `CALL` behavior).
- We assume the HTTP server feature flag (`--features server`) remains the gate for this functionality.