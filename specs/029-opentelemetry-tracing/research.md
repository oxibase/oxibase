# Research & Decisions: OpenTelemetry Tracing System

## 1. OpenTelemetry Setup & Configuration

**Task**: Research best practices for setting up `tracing_subscriber` with `opentelemetry-otlp` in Rust.
**Decision**: Use `opentelemetry-otlp` with `tonic` (gRPC) as the default export mechanism, falling back to HTTP if specified by `OTEL_EXPORTER_OTLP_PROTOCOL`. Configure it conditionally based on the presence of `OTEL_EXPORTER_OTLP_ENDPOINT`. Use `tracing_opentelemetry::layer()` to bridge `tracing` spans to OTel.
**Rationale**: gRPC is the standard, most performant protocol for OTLP. `tracing_subscriber::registry().with(...)` allows composing the standard logger, the OTel exporter, and our custom internal ingestion layer simultaneously. 
**Alternatives considered**: Using `reqwest` instead of `tonic` for HTTP-only. Rejected because gRPC is the industry standard default for OTLP.

## 2. Background Trace Ingestion (Custom Layer)

**Task**: Research how to build a `tracing_subscriber::Layer` to capture completed spans and send them via `crossbeam-channel`.
**Decision**: Implement `struct SystemTraceLayer` that implements `tracing_subscriber::Layer`. In the `on_close` or `on_record` method, extract span attributes, start time, and duration, and push a `SpanEvent` struct into a `crossbeam_channel::Sender`. 
**Rationale**: `crossbeam-channel` is lock-free and extremely fast, ideal for not blocking the critical path of query execution. The `on_close` method is the right place to capture total duration and final metadata.
**Alternatives considered**: `std::sync::mpsc`. Rejected because `crossbeam-channel` provides better performance and a unified cross-thread communication API.

## 3. Preventing Recursive Tracing Loops

**Task**: Research how to prevent the internal trace ingestion queries from generating their own traces, causing an infinite loop.
**Decision**: Use a `std::thread_local!` boolean flag (e.g., `IS_TELEMETRY_THREAD`). The trace flusher thread sets this flag to `true`. The custom `SystemTraceLayer` checks this thread-local variable in its `enabled()` or `on_new_span()` method and returns `false` if it is set. 
**Rationale**: This guarantees that any query executed by the flusher thread (which inserts into `system.traces`) will be completely ignored by the tracing subscriber.
**Alternatives considered**: Span filtering by name or target. Rejected because relying on string matching is brittle and prone to errors if an internal query doesn't perfectly match the exclusion string. Thread-local flags are robust and zero-cost.
