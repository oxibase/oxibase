# Feature Specification: Formal Stored Procedure Logging API

**Feature Branch**: `046-stored-procedure-logging`  
**Created**: June 19, 2026  
**Status**: Draft  
**Input**: User description: "can you create a spercification for the logs ? include pl/SQL with a new keyword LOG. Do not mix it with PLSQL PRINT or RAISE NOTICE, the goals are diff."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scripting Engine Logging (Rhai/Python) (Priority: P1)

As a database stored procedure developer writing in Rhai or Python, I want to use a safe built-in function `oxibase::log(level, msg)` or `oxibase.log(level, msg)` respectively, to write diagnostic messages into `system.logs` without executing direct, unsafe SQL `INSERT` statements on system tables.

**Why this priority**: Core of issue #89. Allows stored procedures to easily record runtime telemetry.

**Independent Test**:
1. Start the database.
2. Create a Rhai procedure calling `oxibase::log("WARN", "Disk space is running tight")`.
3. Create a Python procedure calling `oxibase.log("INFO", "Python initialization complete")`.
4. Run both procedures and query `SELECT level, target, message FROM system.logs` to verify the logs were ingested asynchronously and target is set to the executing procedure names.

**Acceptance Scenarios**:
1. **Given** a Rhai procedure, **When** invoking `oxibase::log("info", "Rhai execution started")`, **Then** a log row is appended to `system.logs` with level 'INFO', target set to the procedure name, and the correct message.
2. **Given** a Python procedure, **When** invoking `oxibase.log("warn", "Python warning message")`, **Then** a log row is appended to `system.logs` with level 'WARN', target set to the procedure name, and the correct message.
3. **Given** an active query trace span, **When** the scripting log is called, **Then** the logged entry automatically inherits the active query's `trace_id` and `span_id`.

---

### User Story 2 - PL/SQL native `LOG` statement (Priority: P1)

As a PL/SQL developer, I want to use a native statement keyword `LOG <level>, <expression>;` to write diagnostic events directly to the database logging tables, distinct from user-facing diagnostic channels (PRINT/RAISE NOTICE).

**Why this priority**: Required for parity across PL/SQL and procedural engines, keeping logging clean and isolated from stdout.

**Independent Test**:
1. Create a PL/SQL stored procedure containing the statement `LOG INFO, 'PL/SQL event triggered';`.
2. Call the procedure.
3. Query `system.logs` to verify a log entry exists with level 'INFO' and message 'PL/SQL event triggered'.

**Acceptance Scenarios**:
1. **Given** a PL/SQL stored procedure with `LOG ERROR, 'Failed transaction attempt';`, **When** the procedure is run, **Then** a log is recorded with level 'ERROR' and target set to the procedure name.
2. **Given** a `LOG` statement in PL/SQL, **When** it executes, **Then** it does not output to the standard procedure console capture/stdout (unlike PRINT and RAISE NOTICE), but instead is channeled straight to the background log flusher.

---

### User Story 3 - System Table Namespace Protection (Priority: P2)

As a database administrator, I want direct DML inserts into `system.logs` from standard user query sessions and transactions to be fully blocked, preventing arbitrary/malicious manipulation of system logs.

**Why this priority**: Lock down security boundaries of internal logs.

**Independent Test**:
1. Attempt to run `INSERT INTO system.logs (level, message) VALUES ('INFO', 'unauthorized insert')` from a user session.
2. Verify it throws a `cannot modify reserved namespace` error.

**Acceptance Scenarios**:
1. **Given** a standard query context, **When** trying to run an INSERT, UPDATE, or DELETE on `system.logs`, **Then** the database returns a `ReservedNamespaceModification` error.

---

### Edge Cases

- **Invalid Log Level**: If an unrecognized level string or identifier is passed (e.g., `oxibase::log("CHICKEN", "test")` or `LOG CHICKEN, 'test';`), the engine MUST default the level to `INFO`.
- **No Procedure Name**: If logging occurs outside of a stored procedure (e.g., a trigger or a function executed from a query where the procedure name is not set), the target MUST default to `'user_procedure'`.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Rhai scripting engine MUST expose `oxibase::log(level, msg)` where `level` and `msg` are string parameters.
- **FR-002**: Python scripting engine MUST expose `oxibase.log(level, msg)` under the native `oxibase` module.
- **FR-003**: PL/SQL parser MUST support the `LOG` statement with syntax: `LOG <level_identifier>, <expression>;` where level_identifier is an unquoted level keyword (`INFO`, `WARN`, `ERROR`, `DEBUG`, `TRACE`), and expression is a string/text-yielding expression.
- **FR-004**: PL/SQL interpreter MUST evaluate the expression and route the resulting log through the engine logging helper.
- **FR-005**: Procedural logging MUST call Rust's native `tracing` macro ecosystem under the hood to leverage the asynchronous background flusher, OpenTelemetry spans, and trace context propagation.
- **FR-006**: Direct user inserts into the `system.logs` table MUST be forbidden in `src/executor/dml.rs` by removing the exception whitelist.

### Key Entities

- **`log_message` helper**: A central function in `src/common/logging.rs` or backend handlers that maps a level string and a message into a `tracing` event.
- **`PLSQL Log AST Node`**: Representing a `LOG` statement in the PL/SQL AST.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Direct inserts to `system.logs` throw `ReservedNamespaceModification` error.
- **SC-002**: Rhai stored procedures successfully write to `system.logs` via `oxibase::log`.
- **SC-003**: Python stored procedures successfully write to `system.logs` via `oxibase.log`.
- **SC-004**: PL/SQL stored procedures successfully write to `system.logs` via `LOG` statement.
- **SC-005**: All 3068 existing tests continue to pass.

---

## Assumptions

- Procedural target name is automatically retrieved from thread-local `CURRENT_PROCEDURE_NAME` context.
- High-severity events (INFO, WARN, ERROR) are recorded in `system.logs`, while DEBUG and TRACE levels are handled by standard log streaming/tracing filters.
