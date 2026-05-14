# Research: Job Scheduler Implementation & Schema Migration

## Decision: Use `cron` crate for schedule parsing and calculation

**Rationale**: 
Implementing a robust cron expression parser and evaluator from scratch is tedious, error-prone, and adds unnecessary maintenance burden. The `cron` crate (`0.12`) is a well-established, standard Rust library for this exact purpose. It handles complex cron logic (ranges, intervals, specific days of week, etc.) out-of-the-box and provides a simple iterator to find the next execution timestamp.

**Alternatives Considered**:
- *Custom parser*: Rejected due to high complexity and edge-case handling (e.g., leap years, day-of-week vs. day-of-month logic).
- *`tokio-cron-scheduler`*: Rejected because our execution engine needs to run within our specific MVCC transaction and thread boundaries; we only need parsing and time calculation, not a full-blown async task scheduler framework.

## Decision: Background Thread vs. Async Task

**Rationale**:
The scheduler will be implemented as a standard `std::thread::spawn` background worker attached to `DatabaseInner`. Since `oxibase` core is primarily synchronous and thread-based for its execution engine, using a simple blocking thread that `sleep`s until the next job execution time avoids dragging heavy async dependencies into the core storage logic unnecessarily.

**Alternatives Considered**:
- *Tokio tasks*: Overkill if `tokio` is only an optional feature in the crate (`server` or `pg-server`). The core `oxibase` library should be able to run jobs without requiring the async runtime.

## Decision: Migration of `_sys_*` to `system.*`

**Rationale**:
To organize internal tables under a dedicated `system` schema instead of using the `_sys_` prefix in the public schema, we will:
1. Ensure the `system` schema is created at initialization.
2. Update all internal table constants (e.g., `SYS_PROCEDURES`, `SYS_FUNCTIONS`, `SYS_TABLE_STATS`) to point to `system.procedures`, `system.functions`, etc.
3. Update their `CREATE TABLE` initialization statements.
4. For backward compatibility/migration during engine startup, we'll check if the old `_sys_` tables exist and move/copy their contents to the new `system.*` tables.

**Alternatives Considered**:
- *Drop and recreate*: We cannot afford to lose existing user-defined functions and procedures, so a migration path is necessary during engine startup.
