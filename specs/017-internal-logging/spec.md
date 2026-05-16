# Feature Specification: Comprehensive Internal Logging System

**Feature Branch**: `017-internal-logging`  
**Created**: May 15, 2026  
**Status**: Draft  
**Input**: User description: "Issue #35: Comprehensive Internal Logging System"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - System Table Ingestion (Priority: P1)

As a database administrator, I want high-severity internal logs to be written to a system table (`system.logs`), so I can query and monitor the database health using standard SQL tools.

**Why this priority**: Core objective of the issue.

**Independent Test**: Can be tested by starting the database, triggering a logged event (e.g. creating a table, which logs an info event), and querying `SELECT * FROM system.logs`.

**Acceptance Scenarios**:

1. **Given** the database is running, **When** I execute `SELECT * FROM system.logs`, **Then** it returns the recent high-severity operational logs (INFO, WARN, ERROR).
2. **Given** a new instance of the database is created, **When** the executor initializes, **Then** the `system.logs` table is automatically created if it doesn't exist.

---

### User Story 2 - Structured Console Output (Priority: P2)

As a system operator, I want the server console output to be in structured JSON format and configurable via log levels, so it can be easily ingested by external logging services (like Datadog).

**Why this priority**: Required for external observability tools, but less critical than internal queryability for DB admins.

**Independent Test**: Can be tested by running the oxibase binary with `RUST_LOG=debug` and inspecting standard output for JSON lines.

**Acceptance Scenarios**:

1. **Given** the server is started with `RUST_LOG=info`, **When** the server logs a startup message, **Then** the message is printed to stdout as a single-line JSON object.

---

### Edge Cases

- What happens if the background logging thread attempts to log an event? The system MUST detect this (via a thread-local flag) and drop the log to prevent an infinite recursive loop.
- What happens if logs are generated faster than the storage engine can write them? The system MUST use a bounded channel and drop older/newer logs rather than blocking query execution threads.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST create a `system.logs` virtual table or physical table on startup.
- **FR-002**: System MUST intercept `tracing` events of level INFO, WARN, and ERROR.
- **FR-003**: System MUST push intercepted logs into a non-blocking, bounded in-memory channel (e.g. `crossbeam-channel`).
- **FR-004**: System MUST have a dedicated background thread that consumes from the channel and inserts records into `system.logs`.
- **FR-005**: System MUST prevent infinite loops by using a thread-local flag (e.g., `IS_LOG_FLUSHER`) within the flusher thread; if set, any new tracing events emitted by that thread are ignored.
- **FR-006**: The CLI binary (`src/bin/oxibase.rs`) MUST configure a JSON formatting `tracing_subscriber` layered with the internal logging interceptor.

### Key Entities

- **`LogEntry`**: Structure representing a captured log event (timestamp, level, target, message).
- **`InternalLogLayer`**: A custom `tracing_subscriber::Layer` that captures events and pushes them to the channel.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `SELECT * FROM system.logs` successfully executes and returns results.
- **SC-002**: Server starts without hanging (verifying loop prevention works).
- **SC-003**: Console output from the binary is valid JSON.
- **SC-004**: Adding the feature causes negligible performance degradation to query throughput.

## Assumptions

- We will use `crossbeam-channel` for the bounded queue.
- The `system.logs` table will be a physical table stored via the standard Executor DDL mechanism on startup (like `system.cron` and `system.procedures`).