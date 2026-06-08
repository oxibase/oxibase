# Phase 1: Data Model

## Entities

### `SystemRingBufferTable`

Implements the `Table` trait.

**Fields**:
- `name: String`
- `schema: Schema`
- `capacity: usize` (default e.g., 100,000)
- `buffer: RwLock<VecDeque<Row>>`
- `stats: RwLock<TableStats>` (for row counts and metrics)

**Key Behaviors**:
- `insert()`: Obtains a write lock on `buffer`. Pushes the `Row` to the back. If `buffer.len() > capacity`, it calls `pop_front()`. Returns Ok.
- `scan()`: Obtains a read lock on `buffer` and clones the current state, returning a `Scanner` implementation over a cloned `Vec<Row>` (or a custom iterator holding a read lock, though cloning might be safer to prevent long-held locks during large queries).
- `get_pending_versions()`: Returns an empty `Vec`.
- `commit()`, `rollback()`: No-ops. MVCC and transaction scoping are ignored.

### Hot Path Telemetry Events

**`SpanEvent`**:
- Remains largely the same but `attributes` will be a `Vec<(String, String)>` or a raw map instead of a serialized JSON string.
- The `start_trace_flusher` loop will iterate over this structure, create the JSON string, and then build the `Row`.

**`LogEntry`**:
- `json_fields` changed to `Vec<(String, String)>` instead of `Option<String>` to defer serialization.

**`MetricEvent`**:
- `attributes` changed to `Vec<(String, String)>` instead of serialized JSON.

## Interface Contracts

### Internal Flusher to Storage

The core architectural change occurs at `engine.rs` (`MVCCEngine::create_table` and table retrieval logic). 
When initializing system tables during database startup, the engine will explicitly instantiate a `SystemRingBufferTable` rather than `MVCCTable`.

```rust
// Contract Concept
if table_name.starts_with("system.") {
    Ok(Box::new(SystemRingBufferTable::new(table_name, schema, 100_000)))
} else {
    // Normal MVCC Table
}
```