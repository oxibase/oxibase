---
layout: default
title: "Internal Observability System"
parent: Architecture
grand_parent: Explanations
nav_order: 6
---

# Internal Observability System

Oxibase provides a comprehensive internal observability system designed to capture high-severity operational events, query performance traces, and database telemetry metrics without impacting the performance of the hot query path. This system allows database administrators to monitor health and operational events using standard SQL queries, while also remaining compatible with external aggregation tools (e.g., Datadog, Logstash, Prometheus) via OpenTelemetry configuration.

## Core Objectives

1. **System Table Ingestion**: Persist application logs (`INFO`, `WARN`, `ERROR`), query traces, and performance metrics into dedicated internal system tables (`system.logs`, `system.traces`, `system.metrics`).
2. **Zero-Overhead Hot Path**: Ensure that observability gathering does not block or slow down execution. Metrics and traces must be collected with minimal overhead (< 5% performance impact).
3. **Infinite Loop Prevention**: Prevent the act of recording observability data from recursively triggering more observability events.
4. **Actionable SQL Analysis**: Provide all telemetry data immediately via standard `SELECT` queries for built-in database administration.

## Architecture

The internal observability system relies on the [`tracing`](https://crates.io/crates/tracing) and [`opentelemetry`](https://crates.io/crates/opentelemetry) ecosystems, utilizing custom subscriber layers to intercept events.

### 1. Interception Layers (`InternalLogLayer`, `SystemTraceLayer`, `SystemMetricsLayer`)

Custom `tracing_subscriber::Layer`s are initialized in the Oxibase CLI (`src/bin/oxibase.rs`). They are composed alongside the standard output formatters.

When an event or span occurs:
- The layers verify the `std::thread_local!` flags (e.g., `IS_LOG_FLUSHER`, `IS_METRICS_THREAD`) to ensure the event isn't originating from a background flusher.
- Events are classified into Log, Trace, or Metric records.
- The formatted record is pushed to a non-blocking, bounded in-memory `crossbeam_channel`.

### 2. Lock-free Asynchronous Channels

To guarantee that the hot query path is not blocked by slow disk I/O, the observability system decouples data generation from storage persistence using bounded `crossbeam_channel`s. 

If the storage engine experiences high latency or an enormous flood of data is generated, the bounded channel prevents memory exhaustion. Once the channel is full, new telemetry is dropped via a `try_send` operation rather than blocking the execution threads.

### 3. Background Flusher Threads

During database initialization (`Database::open`), dedicated background threads are spawned (`oxibase-log-flusher`, `oxibase-trace-flusher`, `oxibase-metrics-flusher`). 

These threads operate entirely asynchronously:
- They wait for new data using `recv_timeout`.
- They batch up to 100 entries at a time (or flush immediately after a 1-second timeout).
- They use the standard `MVCCEngine` API to open a transaction and batch-insert the entries into their respective `system.*` tables.

### 4. Loop Prevention Mechanism

Because the flusher threads utilize the standard database transaction API to insert rows into `system.*` tables, they could inadvertently trigger new `tracing` events (e.g., a "transaction committed" or "disk flush" event). This would create a catastrophic infinite loop where inserting a metric generates a new metric, which generates a new metric, and so on.

To solve this, Oxibase uses `std::thread_local!` flags (`IS_LOG_FLUSHER`, `IS_TELEMETRY_THREAD`, `IS_METRICS_THREAD`).
- When a background thread starts, it sets its specific flag to `true` for itself.
- When an interception layer receives a new `tracing` event, it first checks this flag.
- If the flag is `true`, the event is completely ignored. 

This simple and effective mechanism guarantees safety while allowing the rest of the database engine to remain completely unaware of the observability layer.

## The System Tables

The internal tables are automatically migrated and verified during the `Executor` initialization. Database operators can query them directly using standard SQL.

### `system.logs`

| Column        | Type      | Description                               |
|---------------|-----------|-------------------------------------------|
| `id`          | INTEGER   | Primary Key (Auto Increment)              |
| `timestamp`   | TIMESTAMP | When the event occurred                   |
| `level`       | TEXT      | Severity (`INFO`, `WARN`, `ERROR`)        |
| `target`      | TEXT      | Module path (e.g., `oxibase::storage`)    |
| `message`     | TEXT      | Formatted string of the log event         |
| `trace_id`    | TEXT      | Associated trace ID (if any)              |
| `span_id`     | TEXT      | Associated span ID (if any)               |
| `json_fields` | TEXT      | (Reserved) Additional structured data     |

### `system.traces`

| Column           | Type      | Description                               |
|------------------|-----------|-------------------------------------------|
| `id`             | INTEGER   | Primary Key (Auto Increment)              |
| `trace_id`       | TEXT      | The OpenTelemetry trace ID                |
| `span_id`        | TEXT      | The unique ID of the span                 |
| `parent_span_id` | TEXT      | The parent span ID (if nested)            |
| `name`           | TEXT      | The name of the traced operation          |
| `span_kind`      | TEXT      | The type of span (INTERNAL, SERVER, etc)  |
| `start_time`     | TIMESTAMP | Span start timestamp                      |
| `end_time`       | TIMESTAMP | Span end timestamp                        |
| `duration_ms`    | FLOAT     | Total execution time in milliseconds      |
| `status_code`    | TEXT      | Status code (OK, ERROR)                   |
| `status_message` | TEXT      | Error details (if any)                    |
| `attributes`     | TEXT      | JSON string of key-value pair attributes  |
| `events`         | TEXT      | JSON string of span events                |

### `system.metrics`

| Column        | Type      | Description                               |
|---------------|-----------|-------------------------------------------|
| `id`          | INTEGER   | Primary Key (Auto Increment)              |
| `name`        | TEXT      | The metric name (e.g., `queries_total`)   |
| `description` | TEXT      | Optional description of the metric        |
| `unit`        | TEXT      | Optional unit (e.g., `ms`, `count`, `bytes`) |
| `metric_type` | TEXT      | The type (`counter`, `gauge`, `histogram`)|
| `value`       | FLOAT     | The metric value                          |
| `attributes`  | TEXT      | JSON string of key-value pair attributes  |
| `timestamp`   | TIMESTAMP | When the metric was recorded              |

#### Key Metrics Collected

- **`queries_total`** (Counter): Total number of SQL queries executed.
- **`query_duration_ms`** (Histogram): Execution time for executed queries.
- **`errors_total`** (Counter): Number of queries resulting in an execution error. Includes the error message in the attributes.
- **`transactions_active`** (Gauge): Number of active (in-flight) transactions managed by the MVCC engine.
- **`storage_bytes_written`** (Counter): Number of bytes written to disk/WAL during persistence.
- **`buffer_pool_hits`** & **`buffer_pool_misses`** (Counters): Cache hit and miss events for the in-memory shared buffer pools (`small`, `medium`, `large`). 

*Note: Buffer pool metrics are highly optimized; the underlying atomic counters are periodically sampled by the flusher thread once per second to prevent tracing overhead.*

## Example Queries

Retrieve the most recent error logs:
```sql
SELECT timestamp, message, target 
FROM system.logs 
WHERE level = 'ERROR' 
ORDER BY timestamp DESC;
```

Find the average query execution time:
```sql
SELECT AVG(value) as avg_duration_ms 
FROM system.metrics 
WHERE name = 'query_duration_ms';
```

Track the 5 most expensive recorded operations:
```sql
SELECT name, duration_ms, start_time 
FROM system.traces 
ORDER BY duration_ms DESC 
LIMIT 5;
```