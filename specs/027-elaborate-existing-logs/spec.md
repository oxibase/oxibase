# Specification: Elaborate Existing Logs

## 1. Feature Overview
The `system.logs` system table currently lacks crucial OpenTelemetry standard fields, specifically trace context (`trace_id` and `span_id`). By adding trace correlation fields, we can link individual log entries directly to the spans (e.g., query execution, parsing) that generated them. This aligns the database's internal logging mechanism with OpenTelemetry standards and provides a foundation for the upcoming Query and Procedure Tracing System.

## 2. User Scenarios & Testing

### Scenario 1: Traced Query Execution Logs
**Given** a query is executing within an active trace span,
**When** the database engine emits an internal log (e.g., via `tracing::info!`),
**Then** the background log flusher captures the log and persists it to `system.logs` with the active `trace_id` and `span_id` attached.

### Scenario 2: Trace Context Querying
**Given** a user is investigating a failed or slow query,
**When** they query the `system.logs` table and filter by a specific `trace_id`,
**Then** they can see all logs emitted exclusively during that query's lifecycle.

## 3. Functional Requirements

### FR-001: Schema Enhancement
The `system.logs` table MUST be updated (or migrated) to include `trace_id` (TEXT) and `span_id` (TEXT) columns. 

### FR-002: Trace Context Extraction
The internal logging mechanism MUST extract the current `trace_id` and `span_id` from the active OpenTelemetry span context whenever a log event occurs.

### FR-003: Log Persistence
The log persistence layer MUST write the extracted `trace_id` and `span_id` values to the `system.logs` table alongside the existing log fields (timestamp, level, target, message, json_fields). If a log is emitted outside of an active trace context, these fields SHOULD be stored as NULL.

## 4. Non-Functional Requirements

### NFR-001: Performance Impact
Extracting and storing trace context for logs MUST NOT introduce measurable latency or block the primary query execution threads.

### NFR-002: Backward Compatibility
Existing log queries MUST continue to work. The new columns should be appended without breaking existing tooling that relies on the previous schema.

## 5. Success Criteria

- **SC-001**: A user can query `system.logs` and see non-null `trace_id` and `span_id` values for logs emitted during query execution.
- **SC-002**: All logs emitted within a single query execution share the same `trace_id`.
- **SC-003**: The schema update is applied automatically during database initialization.

## 6. Dependencies & Assumptions

### Assumptions
- The system uses the `tracing` ecosystem, which can natively expose span and trace IDs when configured with `tracing-opentelemetry`.
- The `system.logs` table schema can be safely altered or recreated during initialization if it's an internal-only table.

## 7. Key Entities (Data Model)
**Entity**: `system.logs` Table
- `id`: INTEGER PRIMARY KEY AUTO_INCREMENT
- `timestamp`: TIMESTAMP DEFAULT CURRENT_TIMESTAMP
- `level`: TEXT NOT NULL
- `target`: TEXT NOT NULL
- `message`: TEXT NOT NULL
- `json_fields`: TEXT
- `trace_id`: TEXT (New)
- `span_id`: TEXT (New)
