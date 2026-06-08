# Phase 0: Research & Clarifications

## Background

The database suffers from severe performance bottlenecks due to how internal telemetry (OpenTelemetry spans, logs, metrics) was integrated. By channeling telemetry data into standard, ACID-compliant MVCC tables (`system.logs`, `system.traces`, `system.metrics`), the system incurred massive overhead: lock contention, heavy memory allocations (JSON serialization), and write-amplification via the Write-Ahead Log (WAL).

## Resolved Clarifications

### 1. How are telemetry records reconstructed into standard `Row` format for SQL querying?

**Decision**: The background flusher thread (`start_trace_flusher`, `start_log_flusher`, `start_metrics_flusher`) will receive raw telemetry data structs. Inside the flusher thread, these structs will be converted into standard `Row` objects. The `SystemRingBufferTable` will store these `Row` objects in a `VecDeque<Row>`.

**Rationale**: This keeps the SQL executor completely oblivious to the underlying table implementation. A `SELECT * FROM system.logs` just iterates over the `VecDeque<Row>` without having to do any on-the-fly parsing or JSON deserialization. The heavy lifting (JSON stringification) is shifted entirely to the background flusher threads.

### 2. How does the system handle schema changes to system tables?

**Decision**: Schema modifications (e.g., `ALTER TABLE`, `DROP TABLE`) against `system.*` tables must be explicitly blocked at the engine or executor level.

**Rationale**: Telemetry queries depend on a fixed schema to reliably convert data types into `Row` values. Permitting schema changes would break the hardcoded `Row` construction logic within the flusher threads.

### 3. Should we disable WAL entirely for `system.*` tables?

**Decision**: Yes, telemetry tables will strictly bypass the WAL.

**Rationale**: Telemetry is highly ephemeral. Synchronously writing it to the WAL alongside user data fundamentally destroys the unikernel efficiency. If a crash occurs, losing the last few seconds of telemetry is acceptable and heavily outweighs the performance penalty of disk I/O. The `SystemRingBufferTable` will return an empty vector for `get_pending_versions()`.

### 4. How to defer formatting in the hot path?

**Decision**: The `AttributeVisitor` will no longer serialize values to JSON on the hot path. Instead, it will store a lightweight representation (e.g., raw string clones, primitives) and pass those via the channel.

**Rationale**: `serde_json::to_string()` is too expensive for synchronous query execution paths. The background thread will reconstruct and format the JSON.