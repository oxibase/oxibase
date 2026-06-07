# Phase 1: Quickstart

To understand the architecture and test the implementation of the Telemetry Ring Buffer:

1. **Investigate the hot paths:**
   Review `src/common/tracing.rs`, `src/common/logging.rs`, and `src/common/metrics.rs`. Notice how `AttributeVisitor` is currently performing JSON serialization synchronously.

2. **Understand the Table Trait:**
   Look at `src/storage/traits/table.rs`. The new `SystemRingBufferTable` must implement this trait. Most transactional methods (`commit`, `rollback`, `get_pending_versions`) can be stubbed or no-ops.

3. **Engine Integration:**
   Check `src/storage/mvcc/engine.rs`, specifically `get_table_for_transaction` and where system tables are created during startup.

## Development Workflow

1. Start by creating `SystemRingBufferTable` in `src/storage/mvcc/ring_buffer_table.rs`.
2. Implement the `Table` trait for it, using `RwLock<VecDeque<Row>>`.
3. Modify `engine.rs` to route "system.logs", "system.traces", and "system.metrics" to this new table type.
4. Refactor the `common/` modules to pass raw data over the channels instead of JSON strings.
5. Update the flusher threads (`start_trace_flusher`, etc.) to perform the JSON stringification before calling `table.insert()`.
6. Run `make test` to verify telemetry ingestion still works.