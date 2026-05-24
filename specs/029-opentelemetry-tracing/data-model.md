# Data Model: OpenTelemetry Tracing System

## Key Entities

### SpanEvent
An internal struct representing a completed tracing span, sent over the crossbeam channel to the background flusher.

**Fields**:
- `trace_id` (String): The unique OpenTelemetry trace ID.
- `span_id` (String): The unique OpenTelemetry span ID.
- `parent_span_id` (Option<String>): The ID of the parent span, if any.
- `name` (String): The name of the span (e.g., `parse_program`, `create_plan`).
- `target` (String): The Rust module path where the span originated.
- `start_time` (DateTime): The time the span began.
- `end_time` (DateTime): The time the span completed.
- `duration_ms` (u64): The duration of the span in milliseconds.
- `attributes` (JSON): Metadata captured during the span (e.g., query string, transaction ID).

### System Table: `system.traces`
The relational table where background spans are inserted. This schema is defined during initialization.

**Schema (Assumed based on Issue 34)**:
- `id` (INTEGER PRIMARY KEY AUTOINCREMENT)
- `trace_id` (TEXT)
- `span_id` (TEXT)
- `parent_span_id` (TEXT NULL)
- `name` (TEXT)
- `target` (TEXT)
- `start_time` (TIMESTAMP)
- `duration_ms` (INTEGER)
- `attributes` (TEXT) -- JSON representation of span attributes

## State Transitions
1. **Query Execution**: `tracing::instrument` macro begins a span.
2. **Span Closure**: When the function exits, `tracing` calls `on_close` on the `SystemTraceLayer`.
3. **Queueing**: The layer builds a `SpanEvent` and pushes it to `crossbeam-channel`.
4. **Flushing**: The background thread receives `SpanEvent`s in batches.
5. **Ingestion**: The background thread executes an `INSERT INTO system.traces` query using an internal, non-traced connection.
