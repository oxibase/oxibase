# Specification: Elaborate Existing Logs

## Clarifications

### Session 2026-05-24
- Q: The user explicitly requested: "it also lacks the metrics and traces tables, can we add them so we finish the storage side of the opentelemetry, but NOT implement metrics or traces?". Should I update the spec to include the creation of `system.traces` and `system.metrics` tables? → A: Yes, update spec.

## 1. Feature Overview
The `system.logs` system table currently lacks crucial OpenTelemetry standard fields, specifically trace context (`trace_id` and `span_id`). By adding trace correlation fields, we can link individual log entries directly to the spans (e.g., query execution, parsing) that generated them.

Additionally, to prepare the storage engine for full OpenTelemetry observability, we will preemptively create the `system.traces` and `system.metrics` tables. This feature is strictly bounded to the **storage schema creation** for these tables; the actual collection and insertion of traces and metrics will be implemented in a future iteration. This aligns the database's internal logging and storage mechanism with OpenTelemetry standards.

## 2. User Scenarios & Testing

### Scenario 1: Traced Query Execution Logs
**Given** a query is executing within an active trace span,
**When** the database engine emits an internal log (e.g., via `tracing::info!`),
**Then** the background log flusher captures the log and persists it to `system.logs` with the active `trace_id` and `span_id` attached.

### Scenario 2: Trace Context Querying
**Given** a user is investigating a failed or slow query,
**When** they query the `system.logs` table and filter by a specific `trace_id`,
**Then** they can see all logs emitted exclusively during that query's lifecycle.

### Scenario 3: Initialization of Telemetry Tables
**Given** a fresh or existing database instance starts,
**When** the executor ensures the system schema,
**Then** the `system.traces` and `system.metrics` tables are created alongside the updated `system.logs` table, but remain empty until future telemetry producers are implemented.

## 3. Functional Requirements

### FR-001: Logs Schema Enhancement
The `system.logs` table MUST be updated (or migrated) to include `trace_id` (TEXT) and `span_id` (TEXT) columns. 

### FR-002: Trace Context Extraction
The internal logging mechanism MUST extract the current `trace_id` and `span_id` from the active OpenTelemetry span context whenever a log event occurs.

### FR-003: Log Persistence
The log persistence layer MUST write the extracted `trace_id` and `span_id` values to the `system.logs` table alongside the existing log fields. If a log is emitted outside of an active trace context, these fields SHOULD be stored as NULL.

### FR-004: Traces Table Schema Creation
The system MUST create a `system.traces` table on startup to store OpenTelemetry span data. No data ingestion is required for this table.

### FR-005: Metrics Table Schema Creation
The system MUST create a `system.metrics` table on startup to store OpenTelemetry metric data. No data ingestion is required for this table.

## 4. Non-Functional Requirements

### NFR-001: Performance Impact
Extracting and storing trace context for logs MUST NOT introduce measurable latency or block the primary query execution threads.

### NFR-002: Backward Compatibility
Existing log queries MUST continue to work. The new columns should be appended without breaking existing tooling that relies on the previous schema.

## 5. Success Criteria

- **SC-001**: A user can query `system.logs` and see non-null `trace_id` and `span_id` values for logs emitted during query execution.
- **SC-002**: All logs emitted within a single query execution share the same `trace_id`.
- **SC-003**: The schema update for `system.logs` is applied automatically during database initialization.
- **SC-004**: The `system.traces` and `system.metrics` tables are created in the `system` namespace on startup and can be queried (returning empty results).

## 6. Dependencies & Assumptions

### Assumptions
- The system uses the `tracing` ecosystem, which can natively expose span and trace IDs when configured with `tracing-opentelemetry`.
- The `system.logs` table schema can be safely altered or recreated during initialization if it's an internal-only table.
- OTel Span and Metric data models dictate the required fields for the new telemetry tables.

## 7. Key Entities (Data Model)
**Entity**: `system.logs` Table
- `id`: INTEGER PRIMARY KEY AUTO_INCREMENT
- `timestamp`: TIMESTAMP DEFAULT CURRENT_TIMESTAMP
- `level`: TEXT NOT NULL
- `target`: TEXT NOT NULL
- `message`: TEXT NOT NULL
- `json_fields`: TEXT
- `trace_id`: TEXT (New, Nullable)
- `span_id`: TEXT (New, Nullable)

**Entity**: `system.traces` Table (New)
- `id`: INTEGER PRIMARY KEY AUTO_INCREMENT
- `trace_id`: TEXT NOT NULL
- `span_id`: TEXT NOT NULL
- `parent_span_id`: TEXT
- `name`: TEXT NOT NULL
- `span_kind`: TEXT NOT NULL
- `start_time`: TIMESTAMP NOT NULL
- `end_time`: TIMESTAMP NOT NULL
- `duration_ms`: FLOAT NOT NULL
- `status_code`: TEXT NOT NULL
- `status_message`: TEXT
- `attributes`: TEXT
- `events`: TEXT

**Entity**: `system.metrics` Table (New)
- `id`: INTEGER PRIMARY KEY AUTO_INCREMENT
- `name`: TEXT NOT NULL
- `description`: TEXT
- `unit`: TEXT
- `metric_type`: TEXT NOT NULL
- `value`: FLOAT NOT NULL
- `attributes`: TEXT
- `timestamp`: TIMESTAMP DEFAULT CURRENT_TIMESTAMP
