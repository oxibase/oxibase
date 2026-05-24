# Feature Specification: OpenTelemetry Tracing System

**Feature Branch**: `029-opentelemetry-tracing`  
**Created**: 2026-05-24  
**Status**: Draft  
**Input**: User description: "implement the remainder of the Query and Procedure Tracing System (Issue #34) to fully enable OpenTelemetry in the database engine..."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Trace Export Configuration (Priority: P1)

As a database operator, I want to configure the database to export query traces to a standard external endpoint (like Jaeger or Zipkin) so that I can monitor database performance and query lifecycles using my existing observability stack.

**Why this priority**: Essential for integrating the database into a broader infrastructure observability ecosystem. Without export, tracing data is locked in the system.

**Independent Test**: Can be tested by starting the database with the appropriate export environment variable and verifying that traces are successfully delivered to a mock server.

**Acceptance Scenarios**:

1. **Given** the database is started with the external exporter configured, **When** a query is executed, **Then** span data is exported to the configured endpoint.
2. **Given** the database is started without the exporter configured, **When** a query is executed, **Then** the database operates normally without attempting to export traces externally.

---

### User Story 2 - Query Lifecycle Instrumentation (Priority: P1)

As a database administrator, I want the core query lifecycle phases (Parsing, Planning, Execution) to be automatically instrumented with tracing spans, including relevant metadata, so that I can identify performance bottlenecks in query processing.

**Why this priority**: Instrumentation is the source of all tracing data. Without it, there is nothing to export or ingest.

**Independent Test**: Can be tested via unit tests that capture emitted tracing spans during a query execution and verify the presence of expected spans (Parser, Planner, Executor) and metadata (query string, transaction ID).

**Acceptance Scenarios**:

1. **Given** tracing is enabled, **When** a `SELECT` statement is executed, **Then** spans for parsing, planning, and execution are generated with the query string and duration.
2. **Given** tracing is enabled, **When** a DML statement (`INSERT`/`UPDATE`) is executed, **Then** critical execution paths emit spans capturing transaction IDs and relevant contextual data.

---

### User Story 3 - Background Trace Ingestion into System Tables (Priority: P2)

As a database user, I want the database to automatically capture its own tracing spans and ingest them into the internal `system.traces` table in the background, so that I can query my own query performance data using standard SQL without needing an external observability stack.

**Why this priority**: Provides out-of-the-box observability for users who don't have an external setup, utilizing the already implemented `system.traces` table.

**Independent Test**: Can be tested by executing a query and then querying `SELECT * FROM system.traces`, verifying that records matching the executed query's spans appear within a short timeframe.

**Acceptance Scenarios**:

1. **Given** a query is executed, **When** I query the `system.traces` table a moment later, **Then** the table contains records corresponding to the query's execution spans.
2. **Given** a high volume of queries, **When** tracing spans are generated rapidly, **Then** the background ingestion batches inserts efficiently without blocking query execution.
3. **Given** the background process is inserting spans into `system.traces`, **When** the insertion SQL executes, **Then** the insertion itself does NOT generate recursive traces (prevented by internal logic).

### Edge Cases

- What happens if the configured external endpoint is unreachable or times out?
- How does the background trace ingestion handle a massive influx of spans to prevent memory exhaustion?
- How does the system guarantee that internal system queries (like the trace flushers) are strictly filtered to prevent infinite recursive tracing loops?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST configure a global tracing registry supporting external telemetry export.
- **FR-002**: The export MUST be conditionally enabled based on the presence of specific environment variables (e.g., `OTEL_EXPORTER_OTLP_ENDPOINT`).
- **FR-003**: The core query phases (parsing, planning, executing) MUST be instrumented to capture start, end, and duration.
- **FR-004**: Traces MUST capture metadata including, but not limited to, the query string, transaction ID, and phase context.
- **FR-005**: The system MUST implement an internal mechanism to capture completed span events.
- **FR-006**: The capture mechanism MUST send completed spans over a lock-free queue to a dedicated background ingestion thread.
- **FR-007**: The background ingestion thread MUST batch-insert completed spans into the `system.traces` table.
- **FR-008**: The background ingestion thread MUST utilize a thread-local flag to absolutely prevent recursive trace emission when executing its own internal SQL inserts.

### Key Entities

- **Tracing Registry**: The central component that collects and dispatches tracing events to exporters.
- **Span Event**: An instance of captured data representing a distinct phase in query processing.
- **Trace Flusher**: A background process responsible for dequeuing captured spans and persisting them as SQL rows.
- **System Table (`system.traces`)**: The relational representation of the span data.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Traces are successfully visible in an external backend when export is configured.
- **SC-002**: A query against `system.traces` successfully returns the spans for recently executed user queries.
- **SC-003**: System overhead for trace collection and internal ingestion does not exceed a 10% performance degradation on standard benchmark queries.
- **SC-004**: No infinite recursive loops occur; internal `system.traces` and `system.logs` inserts do not appear in the trace logs.

## Assumptions

- The `system.traces` table schema exactly matches the data structure of the captured tracing spans.
- The lock-free queue mechanism provides sufficient throughput for the expected trace volume.
- The existing logging infrastructure provides a valid structural template for the trace flusher.
