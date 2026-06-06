# Phase 0: Research & Architecture Decisions

## Unknowns & Clarifications

The spec did not explicitly define any unknowns requiring external clarification (Section 7 is "None"). However, some technical details needed resolution during the planning phase:

### 1. Trace and Span ID Generation inside `tracing.rs`
- **Decision**: Update the `SystemTraceLayer::on_new_span` to generate or extract a `trace_id` and `span_id`. If `tracing-opentelemetry` is active, it would provide these; otherwise, we generate a trace ID (e.g. `format!("trace-{}", id.into_u64())` or `uuid::Uuid::new_v4().to_string()`) and use the tracing `Id` for `span_id`.
- **Rationale**: To correlate logs and metrics to spans, the span extensions must contain the `trace_id` and `span_id` as soon as the span is created.
- **Alternatives**: Relying solely on `tracing-opentelemetry` was rejected because the system must function correctly standalone.

### 2. Context Propagation to Logs and Metrics
- **Decision**: Require `S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>` in `InternalLogLayer` and `SystemMetricsLayer`. In `on_event`, call `ctx.lookup_current()` to get the active span, then access its extensions to retrieve `trace_id` and `span_id`.
- **Rationale**: This is the idiomatic way to propagate context within the `tracing` ecosystem.

### 3. Structured Fields in Logs
- **Decision**: Implement a comprehensive `LogVisitor` in `src/common/logging.rs` similar to `AttributeVisitor` in `tracing.rs`. Extract `message` explicitly, but collect all other fields into a `serde_json::Map` and serialize it for the `json_fields` column.
- **Rationale**: Ensures no data loss (e.g. `tracing::error!(code = 500)` will capture `code=500` in `json_fields`).
