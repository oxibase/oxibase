# Quickstart: Trace Hierarchy implementation

1. First, address `src/api/database.rs`:
   - For `Database::execute`, `query`, `execute_with_timeout`, `query_with_timeout`, `execute_named`, `query_named`: 
   - Define a helper macro or function to truncate SQL (`let trunc_sql = if sql.len() > 1024 { &sql[..1024] } else { sql };`)
   - Create and enter the span: `let _span = tracing::info_span!("db.execute", sql = %trunc_sql).entered();`

2. Address Background Jobs (`src/executor/scheduler.rs`):
   - In `JobScheduler::execute_job`, add `let _span = tracing::info_span!("job.execute", job_id = job_id, job_name = name).entered();`

3. Address Network Context (`src/server/handlers.rs`):
   - For relevant Axum handlers (`get_table`, `insert_row`, `update_row`, `delete_row`, `invoke_procedure`, `execute_sql`, `workspace_execute_sql`), extract HTTP headers via `axum::http::HeaderMap` injection.
   - Use `tracing_opentelemetry::OpenTelemetrySpanExt` or manual extraction to set the parent span context from incoming OpenTelemetry headers.

4. Write an integration test in `tests/` that intercepts spans and verifies `parent_span_id` is populated for inner executor logic.
