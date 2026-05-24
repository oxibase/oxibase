# Implementation Plan: Elaborate Existing Logs & Create Telemetry Tables

1.  **Update `src/storage/logs.rs`**:
    *   Modify `CREATE_LOGS_SQL` to include `trace_id TEXT` and `span_id TEXT`.
    *   *Self-correction: Existing databases will need migration. We should add an `ALTER TABLE` fallback if the table exists but lacks these columns, or just drop and recreate if it's strictly internal and ephemeral.*

2.  **Create `src/storage/traces.rs`**:
    *   Define `SYS_TRACES` constant.
    *   Define `CREATE_TRACES_SQL` using the schema from the spec.
    *   Add `is_traces_table(schema, name)` helper.

3.  **Create `src/storage/metrics.rs`**:
    *   Define `SYS_METRICS` constant.
    *   Define `CREATE_METRICS_SQL` using the schema from the spec.
    *   Add `is_metrics_table(schema, name)` helper.

4.  **Update `src/executor/mod.rs`**:
    *   Add `ensure_traces_table_exists()` to run `CREATE_TRACES_SQL`.
    *   Add `ensure_metrics_table_exists()` to run `CREATE_METRICS_SQL`.
    *   Update `ensure_system_schema_and_migrations()` to call these new functions.
    *   Update `ensure_logs_table_exists()` to handle the schema migration (adding `trace_id` and `span_id` columns if they are missing).

5.  **Review existing log insertion logic**:
    *   Wait, does `oxibase` currently have a background log flusher that inserts into `system.logs`? I need to search for where `SYS_LOGS` is written to. If it exists, I must update it to extract trace contexts. If it *doesn't* exist yet (the table might just be a placeholder), I need to clarify or implement the extraction mechanism as per FR-002 and FR-003. Let's check this first.
