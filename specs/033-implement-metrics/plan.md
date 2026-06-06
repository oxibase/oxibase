# Implementation Plan: Implement System Metrics

**Feature**: `033-implement-metrics`
**Spec Reference**: `specs/033-implement-metrics/spec.md`

## 1. Technical Context

*Context gathered from spec and existing codebase.*

- **Language/Framework**: Rust, OpenTelemetry, tracing.
- **Key Files/Directories**: `src/common/tracing.rs`, `src/common/logging.rs`, `src/bin/oxibase.rs`, `src/common/metrics.rs` (new), `src/storage/metrics.rs`, `src/executor/mod.rs`
- **Knowns**:
  - `system.metrics` schema is already defined in `src/storage/metrics.rs`.
  - The table is created on startup in `src/executor/mod.rs`.
  - The existing tracing/logging mechanism uses a layer and a background crossbeam thread to flush to MVCC, bypassing tracing on its own thread via a thread-local flag.
- **Unknowns**: None

## 2. Constitution Check

*Assessment against the project's Core Principles defined in `.specify/memory/constitution.md`.*

- **Strict ACID Integrity**: Modifying the metrics collection must not interfere with the ACID guarantees of the transactional data operations. We'll use the existing `MVCCEngine` interface, creating a transaction just like logging and tracing.
- **Zero-Copy Unikernel Efficiency**: The metrics flushing thread batches metric events to avoid excessive allocations and frequent disk writes, aligning with the zero-copy principle.
- **Safe and Idiomatic Rust**: No `unwrap()`/`expect()` in library code. Use standard `Result` propagation.
- **Embedded Business Logic**: N/A for this task.

### Gate Evaluation
- [x] Does the plan respect the monolith structure? (Yes)
- [x] Does it maintain ACID integrity? (Yes)
- [x] Does it avoid unnecessary allocations (`Vec` clones, etc.)? (Yes)
- [x] Is the error handling idiomatic (no `unwrap` in lib)? (Yes)

## 3. Phase 0: Research Findings

We will follow the existing tracing/logging architecture and create a new `src/common/metrics.rs` file.

**Decision**: Use `tracing` metrics extensions or a custom OpenTelemetry setup. Since the project uses `tracing` heavily, we will hook into it. However, the requirement specifically mentions `MetricEvent`, counters, gauges, etc., which aligns more closely with standard metrics libraries. Looking at `Cargo.toml`, we have `opentelemetry = "0.32.0"`.

Wait, the current logging/tracing infrastructure intercepts `tracing::info!` and `tracing::span`. If we want to capture *metrics* (like query counts), we could either emit specific tracing events (`tracing::info!(metric_type="counter", metric_name="queries", value=1.0)`) and intercept them via a layer, OR we can use OpenTelemetry metrics directly. Let's use the explicit `tracing` event interception method, as it reduces dependencies and integrates perfectly with the existing crossbeam-channel batching architecture. We can create a `SystemMetricsLayer` that listens to `tracing::info!` calls that have a `metric_name` field.

Actually, let's create a dedicated module `src/common/metrics.rs` with explicit functions `increment_counter`, `record_histogram`, etc., that push directly to a channel, bypassing the `tracing` layer overhead for metrics. This provides a cleaner API: `metrics::increment_counter("queries_total", 1, "query count");`

Wait, `SystemTraceLayer` uses `tracing` to capture span start/end.
Let's build `src/common/metrics.rs` with:
1. `MetricEvent` struct.
2. Global `crossbeam_channel::Sender<MetricEvent>` (initialized via `lazy_static` or passed around, but passing around in a database is hard. Let's use a `std::sync::OnceLock` or similar, or just pass it where needed. The `executor` handles query execution).
Actually, to avoid `lazy_static`, we can pass the sender to the `Database` or `MVCCEngine`.

Let's look at how logging and tracing are wired. In `src/bin/oxibase.rs`, a `crossbeam_channel` is created, the Receiver is passed to `start_trace_flusher`, and the Sender is passed to `SystemTraceLayer` which is registered globally via `tracing_subscriber::registry().with(...)`.

We can do exactly this: create `SystemMetricsLayer` in `src/common/metrics.rs`.
If someone emits `tracing::info!(metric_type = "counter", metric_name = "queries_total", value = 1.0)`, the `SystemMetricsLayer` catches it, creates a `MetricEvent`, and sends it to the flusher.

## 4. Phase 1: Data Model & Contracts

### Data Model (`data-model.md`)

```markdown
# Data Model: System Metrics

## Entity: MetricEvent

Represents an internal database metric to be persisted in `system.metrics`.

- `name` (String): The name of the metric (e.g., `queries_total`).
- `description` (Option<String>): Optional description.
- `unit` (Option<String>): Optional unit (e.g., `ms`, `count`).
- `metric_type` (String): `counter`, `gauge`, or `histogram`.
- `value` (f64): The metric value.
- `attributes` (String): JSON encoded string of key-value attributes.
- `timestamp` (DateTime<Utc>): When the metric was recorded.
```

### Contracts (`contracts/metrics.md`)

```markdown
# Contracts: System Metrics

No external API contracts. Internally, developers emit metrics using the `tracing` macros:

\`\`\`rust
tracing::info!(
    target: "oxibase::metrics",
    metric_type = "counter",
    metric_name = "queries_total",
    value = 1.0,
    unit = "count",
    description = "Total number of executed queries"
);
\`\`\`
```

## 5. Post-Design Constitution Check

- The design maintains the existing architectural patterns established by logging/tracing.
- No new external services are introduced.
- Performance impact is minimal due to asynchronous batching via crossbeam channels.

## 6. Next Steps

1. Run the agent to update `AGENTS.md`.
2. Move to implementation phase.
