# Telemetry Correlation

## 1. Feature Description
Currently, the system's logging, tracing, and metrics modules exist independently, resulting in isolated observability data. Specifically, logs and metrics lack trace/span correlation, making it difficult to trace the lifecycle of a request or operation across the system. 

This feature will implement context propagation and capture structured fields so that logs, metrics, and traces are properly correlated. By attaching trace IDs and span IDs to logs and metrics, and capturing all structured fields into JSON representations, we aim to unify observability data, enabling robust analysis and root-cause identification.

## 2. User Scenarios & Testing

### Scenario 1: Correlating Logs with a Trace
- **Given** an active operation is executing within a trace span (e.g., a database query execution),
- **When** an error or informational log is emitted during that operation,
- **Then** the log entry in `system.logs` contains the `trace_id` and `span_id` of the active span, and all structured attributes passed to the logging macro are captured in `json_fields`.

### Scenario 2: Correlating Metrics with a Trace
- **Given** an active operation is executing within a trace span,
- **When** a metric (e.g., buffer pool miss or query duration) is recorded,
- **Then** the metric entry in `system.metrics` carries the `trace_id` and `span_id` as part of its attributes, linking the metric back to the specific operation that generated it.

### Scenario 3: Missing Fields Fallback
- **Given** a log is emitted using `tracing::error!(code = 500)` without an explicit string message,
- **When** the log is processed,
- **Then** the `message` field is empty, but the `code` field and any other attributes are properly serialized and stored in the `json_fields` column.

## 3. Functional Requirements

- **FR-001**: **Trace ID Generation**: The `trace_id` and `span_id` must be generated (or extracted from OpenTelemetry context) at the moment a span is created, and stored within the span's extensions so they are available while the span is active.
- **FR-002**: **Log Context Propagation**: The logging layer must retrieve the `trace_id` and `span_id` from the currently active span's extensions and attach them to the `LogEntry`.
- **FR-003**: **Structured Log Fields**: The logging layer must capture all fields passed to the logging macro. The explicit `message` field must map to the `message` property, while all other fields must be serialized into a JSON string and stored in the `json_fields` property.
- **FR-004**: **Metric Context Propagation**: The metrics layer must retrieve the `trace_id` and `span_id` from the active span and include them in the metric's attributes JSON payload.

## 4. Success Criteria

- 100% of internal logs emitted within an active span have non-null `trace_id` and `span_id` values in the `system.logs` table.
- 100% of structured attributes attached to a log macro are stored in the `json_fields` column, rather than being discarded.
- Metrics emitted within an active span include the associated `trace_id` and `span_id` in their `attributes` JSON payload.
- Telemetry correlation processing introduces no noticeable performance degradation (latency overhead per log/metric emission must remain minimal).

## 5. Key Entities / Data Models

- **LogEntry**: Expanded to correctly populate `trace_id`, `span_id`, and `json_fields` (derived from captured attributes).
- **MetricEvent**: The `attributes` JSON payload is enriched to include `trace_id` and `span_id` when the event occurs within an active span.
- **Span Event/Extensions**: Updated to hold the initialized `trace_id` and `span_id` throughout the lifetime of the span.

## 6. Assumptions & Dependencies

- The database system already has functional `system.logs`, `system.traces`, and `system.metrics` tables.
- The `tracing` and `tracing-subscriber` crates are used for capturing and processing telemetry events.
- Any OpenTelemetry integration (if present) uses standard context propagation headers/mechanisms.

## 7. Open Questions / Clarifications Needed

None.
