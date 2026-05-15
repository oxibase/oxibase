---
layout: default
title: "Internal Logging System"
parent: Architecture
grand_parent: Explanations
nav_order: 6
---

# Internal Logging System

Oxibase provides a comprehensive internal logging system designed to capture high-severity operational events without impacting the performance of the hot query path. This system allows database administrators to monitor health and operational events using standard SQL queries, while also emitting structured JSON logs for external aggregation tools (e.g., Datadog, Logstash).

## Core Objectives

1. **System Table Ingestion**: Persist high-severity application logs (`INFO`, `WARN`, `ERROR`) into a dedicated internal system table (`system.logs`).
2. **Structured Console Output**: Output logs as structured JSON to stdout for easy ingestion by external observability agents.
3. **Zero-Overhead Hot Path**: Ensure that logging does not block or slow down query execution.
4. **Infinite Loop Prevention**: Prevent the logging of database insertions from recursively triggering more logs.

## Architecture

The internal logging system relies on the [`tracing`](https://crates.io/crates/tracing) ecosystem and uses a custom layer to intercept events.

### 1. The `InternalLogLayer`

A custom `tracing_subscriber::Layer` is initialized in the Oxibase CLI (`src/bin/oxibase.rs`). It is composed alongside the standard `fmt::layer().json()`.

When an event occurs:
- The layer checks the `IS_LOG_FLUSHER` thread-local flag (see below).
- It verifies the severity level. Only `INFO`, `WARN`, and `ERROR` logs are captured.
- The event is formatted into a `LogEntry` structure containing the timestamp, severity level, target module, and the formatted message.
- The `LogEntry` is pushed to a non-blocking, bounded in-memory channel.

### 2. Lock-free Asynchronous Channel

To guarantee that the hot query path is not blocked by slow disk I/O, the logging system decouples log generation from storage persistence using a bounded `crossbeam_channel`. 

If the storage engine experiences high latency or an enormous flood of logs is generated, the bounded channel prevents memory exhaustion. Once the channel is full, new logs are dropped via a `try_send` operation rather than blocking the execution threads.

### 3. Background Flusher Thread

During database initialization (`Database::open`), a dedicated background thread named `oxibase-log-flusher` is spawned. 

This thread operates entirely asynchronously:
- It waits for new logs using `recv_timeout`.
- It batches up to 100 entries at a time (or flushes immediately after a 1-second timeout).
- It uses the standard `MVCCEngine` API to open a transaction and batch-insert the log entries into the `system.logs` table.

### 4. Loop Prevention Mechanism

Because the flusher thread utilizes the standard database transaction API to insert rows into `system.logs`, it could inadvertently trigger new `tracing` events (e.g., a "transaction committed" or "disk flush" log). This would create a catastrophic infinite loop where inserting a log generates a new log, which generates a new log, and so on.

To solve this, Oxibase uses a `std::thread_local!` flag: `IS_LOG_FLUSHER`.
- When the background thread starts, it sets this flag to `true` for itself.
- When `InternalLogLayer` intercepts a new `tracing` event, it first checks this flag.
- If `IS_LOG_FLUSHER` is `true`, the event is completely ignored. 

This simple and effective mechanism guarantees safety while allowing the rest of the database engine to remain completely unaware of the logging layer.

## The `system.logs` Table

The internal `system.logs` table is automatically migrated and verified during the `Executor` initialization.

| Column        | Type      | Description                               |
|---------------|-----------|-------------------------------------------|
| `id`          | INTEGER   | Primary Key (Auto Increment)              |
| `timestamp`   | TIMESTAMP | When the event occurred                   |
| `level`       | TEXT      | Severity (`INFO`, `WARN`, `ERROR`)        |
| `target`      | TEXT      | Module path (e.g., `oxibase::storage`)    |
| `message`     | TEXT      | Formatted string of the log event         |
| `json_fields` | TEXT      | (Reserved) Additional structured data     |

Database operators can query it directly using standard SQL:

```sql
SELECT timestamp, level, message 
FROM system.logs 
ORDER BY timestamp DESC 
LIMIT 10;
```
