# Phase 1: Data Model & Contracts

## Entities

### `SpanEvent` (in `src/common/tracing.rs`)
- Already has `trace_id` and `span_id`.
- Requires moving the generation of these IDs from `on_close` to `on_new_span`, storing them in the span's extensions so other layers can access them while the span is active.

### Span Extensions Tuple
- Currently: `(Instant, DateTime<Utc>, String, String, serde_json::Map<String, serde_json::Value>)`
- Will become: `(Instant, DateTime<Utc>, String, String, serde_json::Map<String, serde_json::Value>, String, String)`
- The last two strings represent `trace_id` and `span_id`.

### `LogEntry` (in `src/common/logging.rs`)
- Update `trace_id: Option<String>` (populated from span context).
- Update `span_id: Option<String>` (populated from span context).
- Add `json_fields: Option<String>` to hold the serialized extra attributes from the `LogVisitor`.
- Update `insert_log_batch` to write `json_fields` as `Value::Text(json)` instead of `Value::null_unknown()`.

### `MetricEvent` (in `src/common/metrics.rs`)
- Add `trace_id` and `span_id` lookup in `SystemMetricsLayer::on_event`.
- Append `trace_id` and `span_id` into the `attributes` map before it is serialized to JSON, or add them as explicit fields if the DB schema supports it. (According to FR-004, include them in the metric's attributes JSON payload).

## Contracts
No external API contracts. Internal changes strictly involve `tracing` layers and internal flusher queues (`crossbeam_channel::Sender`).
