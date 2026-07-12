# Feature Specification: router-refinement-dap-tcp

**Feature Branch**: `053-router-refinement-dap-tcp`  
**Created**: Mon Jun 22 2026  
**Status**: Draft  
**Input**: User description: "Refine Generic Router to fix web server leaks, percent decoding, SQL query routing bugs, and add standalone TCP listener support for external DAP debuggers."

## Clarifications

### Session 2026-06-22
- Q: How should we transition complex telemetry dashboard formatting and calculation out of Rust handlers while keeping the router generic? → A: Instead of hardcoded Rust telemetry processing or complex inline SQL page-rendering query blocks, the frontend workspace templates should leverage existing HTTP RPC endpoints (`/api/rpc/{procedure_name}`) asynchronously via Fetch/AJAX. Complex telemetry aggregations and bounds calculations will be implemented as stored procedures/functions inside the database, which is heavily encouraged for maximum separation of concerns.
- Q: How should the standalone DAP TCP listener be activated in the Oxibase CLI / binary execution? → A: It will be exposed as a global CLI argument option (`--dap-tcp` and `--dap-port <port>`) on the main `oxibase` binary, allowing developers to activate external debugging in any serve, REPL, or file-execution mode.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Multi-Byte URL Filter Querying (Priority: P1)

Users execute dynamic log and trace filtering queries from the workstation using localized search strings and UTF-8 parameters (e.g. searching for logs containing "á" or "connection_failed"). The generic router correctly decodes query-string percent characters into multi-byte UTF-8 bytes without corrupting them.

**Why this priority**: Core query capability. Without proper UTF-8 percent decoding, any search filters or inputs with non-ASCII text will fail matching or corrupt data.

**Independent Test**: Can be verified by issuing a GET request to `/workspace/observe/logs?search=error_%C3%A1` and validating that the router correctly decodes and binds `"error_á"` into the query parameters.

**Acceptance Scenarios**:
1. **Given** a request query parameter with percent-encoded multi-byte UTF-8 characters (`%C3%A1`), **When** `url_decode` is invoked on the input, **Then** it must correctly decode it into its actual UTF-8 representation (`á`) instead of separate ASCII representation (`Ã¡`).
2. **Given** a query parameter with standard ASCII spaces (`+` or `%20`), **When** decoded, **Then** it must resolve to a standard space character.

---

### User Story 2 - SQL Query Classification with Comments & Formatting (Priority: P1)

Users type custom SQL queries inside the workstation editor that include leading formatting, indentation, or SQL comments (e.g. `-- Get trace count\nSELECT COUNT(*) FROM system.traces`). The generic router correctly classifies the query as a SELECT (read) statement, runs it via `query_named` instead of `execute_named`, and renders result rows in the workspace grid.

**Why this priority**: Editor robustness. Developers frequently document query code with inline comments. Executing read queries as write blocks results in empty/blank workstations or failed SQL evaluations.

**Independent Test**: Verified by posting a SQL query with leading spaces, newlines, and comments to `/workspace/sql` and confirming that the engine returns the expected row output.

**Acceptance Scenarios**:
1. **Given** a SQL query containing leading comments (`-- comment` or `/* comment */`) or whitespace characters, **When** evaluated by the query router, **Then** the router must ignore the comments/whitespace and correctly classify it as a row-returning SELECT/SHOW query.
2. **Given** a multi-statement or non-SELECT query with leading comments, **When** evaluated, **Then** it must classify it as a write execution block and return rows affected.

---

### User Story 3 - Clean Workspace Domain Separation (Priority: P2)

All specialized telemetry dashboard calculations—such as log level histograms, Gantt timeline durations, and trace details—must be cleanly separated from the Rust server code. The workstation templates utilize standard frontend AJAX requests to trigger back-end database stored procedures and retrieve results asynchronously via the `/api/rpc/{procedure_name}` RPC endpoints.

**Why this priority**: Engine purity and performance. Offloading UI-specific telemetry aggregations to database-defined procedural languages (such as PL/SQL or Rhai) dramatically simplifies the routing layer, matching the dynamic, database-driven monolith architecture.

**Independent Test**: Search the `src/server/` directory and confirm that zero code blocks reference workspace-specific dashboard paths (`/workspace/observe/logs`, `/workspace/observe/traces`, etc.) or custom telemetry fields.

**Acceptance Scenarios**:
1. **Given** a GET request to `/workspace/observe/logs`, **When** rendered, **Then** the template javascript must call `/api/rpc/` to fetch log counts by level asynchronously via a stored procedure.
2. **Given** a GET request to `/workspace/traces/123`, **When** rendered, **Then** the timeline bounds and spans must be fetched asynchronously via a stored procedure RPC call.

---

### User Story 4 - External DAP Debugger TCP Support (Priority: P1)

Database developers debug stored procedures and functions using external DAP-compatible editors (e.g., VS Code, Neovim, or Eclipse) rather than the web workstation UI. The server hosts a configurable standalone TCP listener port that communicates standard DAP payloads, enabling debugger clients to connect directly.

**Why this priority**: Flexibility and standard compatibility. Developers prefer using their native coding environments for debugging tasks.

**Independent Test**: Can be tested by starting up a local database instance with the DAP TCP listener enabled and successfully initializing a debug connection using a standard VS Code debug profile.

**Acceptance Scenarios**:
1. **Given** the database starts with the standalone DAP TCP listener active on a designated port, **When** a standard external DAP client connects to the port, **Then** they must successfully perform standard DAP handshakes (`initialize`, `setBreakpoints`, etc.).
2. **Given** a debugging session is active over the TCP port, **When** a stored procedure hits a registered breakpoint, **Then** the database execution thread must block, and the TCP client must receive a `"stopped"` event.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `url_decode` helper MUST collect percent-encoded byte values in a byte-buffer (`Vec<u8>`) during decoding, converting the final array into a UTF-8 lossy string at completion to guarantee multi-byte character safety.
- **FR-002**: The query classification logic (`is_query` or equivalent) MUST strip leading whitespace, single-line SQL comments (`--...`), and multi-line SQL comments (`/*...*/`) before checking the SQL verb to ensure correct read-vs-write categorization.
- **FR-003**: The generic router in `src/server/` MUST NOT contain hardcoded references or conditional blocks targeting specific workspace UI paths (such as `/workspace/observe/logs`, `/workspace/observe/traces`, or `/workspace/run_modal`). 
- **FR-004**: All specialized workspace telemetry aggregations (e.g., logs level histogram, traces Gantt timeline calculation, and procedural arguments extraction) MUST be implemented inside the database as stored procedures or functions, and accessed asynchronously by front-end templates via standard HTTP RPC (`/api/rpc/{procedure_name}`) calls.
- **FR-005**: All direct `.unwrap()` operations on procedural results or maps inside routing and execution handlers MUST be replaced with safe, propagation-ready error handling to prevent thread panics.
- **FR-006**: The system MUST implement a standalone TCP listener that implements the standard DAP framing protocol (`Content-Length: <len>\r\n\r\n<payload>`) over raw TCP, configurable via global CLI flags `--dap-tcp` and `--dap-port <port>`. This listener MUST be capable of running concurrently inside any server, REPL, or command-execution modes.
- **FR-007**: The TCP listener MUST bridge inbound and outbound messages directly to the shared `DebugController`, allowing standard DAP commands to pause, step, and resume execution threads identically to the WebSocket implementation.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of workspace-specific hardcoded logic is removed from `src/server/handlers.rs`, and the files compile with zero clippy or formatting violations under `make lint`.
- **SC-002**: A new integration test executes and passes proving that multi-byte percent-encoded query filters (such as `error_á`) are correctly decoded and matched.
- **SC-003**: An integration test executes and passes proving that SQL statements starting with whitespace, indentation, or leading comments (`--`) are correctly evaluated and run as row-returning queries.
- **SC-004**: An integration test executes and passes proving that an external DAP client can connect to the TCP listener port, initialize, set a breakpoint, receive a `"stopped"` event, and resume execution successfully.

---

## Assumptions

- Storing the logs histogram computation as a database view (e.g., `system.log_histogram`) provides sufficient performance for the logs dashboard.
- The default standalone DAP TCP port is `4711` but can be customized or disabled completely to prevent unauthorized debugging connections.
