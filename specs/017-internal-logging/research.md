# Research: Internal Logging System

## 1. Tracing Subscriber Initialization in `src/bin/oxibase.rs`
- **Decision**: Use `tracing_subscriber::registry().with(fmt_layer).with(internal_log_layer).init()` where `fmt_layer` is a JSON-formatted stdout layer from `tracing_subscriber::fmt`.
- **Rationale**: The `tracing` ecosystem is built around composition via `Registry`. We need standard JSON output and our custom interceptor to work together.
- **Alternatives**: Replacing the global logger entirely, but we want to retain the ability to easily output JSON to the console.

## 2. Location/Mechanics for `system.logs` Creation
- **Decision**: Add `src/storage/logs.rs` (following the pattern of `jobs.rs`, `statistics.rs`, etc.) to define the DDL constant for `system.logs`. Update `src/executor/mod.rs` inside `ensure_system_schema_and_migrations` to execute the table creation.
- **Rationale**: Keeps system table definitions collocated and uses the existing initialization lifecycle in `Executor::new()`.
- **Alternatives**: Creating a true virtual table in `system_schema.rs` like `system.tables`. However, since we actually want to persist logs (up to a limit or rotation policy in the future) and index them, a physical table in the system schema is better.

## 3. Lock-free Channel
- **Decision**: Add `crossbeam-channel` dependency to `Cargo.toml`.
- **Rationale**: It provides a highly optimized bounded channel. We will set a reasonable bound (e.g., 10,000 items) and use `try_send` to drop messages if the channel is full, preventing any backpressure from blocking execution threads.

## 4. Loop Prevention Flag
- **Decision**: Use `std::thread_local!` to define a `RefCell<bool>` named `IS_LOG_FLUSHER`.
- **Rationale**: When the background thread wakes up to insert logs, it sets this to `true`. The custom `tracing` Layer checks this variable; if `true`, it ignores the event. Simple and effective.
