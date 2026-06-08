# Quickstart: Telemetry Correlation

Logs, traces, and metrics are automatically correlated in Oxibase.

To see this in action:
1. Wrap a database operation in a tracing span: `tracing::info_span!("execute_query")`.
2. Inside that operation, emit a log: `tracing::error!(code = 500, user = "admin", "Query failed")`.
3. The log will automatically be linked to the `execute_query` span.
4. When queried from `system.logs`, the row will contain:
   - `trace_id`: The ID of the overarching trace.
   - `span_id`: The ID of the active span.
   - `json_fields`: `{"code": 500, "user": "admin"}`.
