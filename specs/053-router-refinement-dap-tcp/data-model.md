# Data Model: Telemetry & Log Explorer UDFs

This document details the database-side models, system routines, and schemas required to support the decoupled AJAX telemetry explorer dashboards.

## 1. System Telemetry Functions

Instead of hardcoded formatting inside the Rust server, the workspace dashboards query these table-valued User-Defined Functions (UDFs) asynchronously.

### `system.get_log_histogram()`

*   **Type**: Table-Valued Function (TVF)
*   **Returns**: `TABLE(level TEXT, count INT)`
*   **Purpose**: Groups and aggregates the total occurrences of log entries by their severity level from the `system.logs` system table.
*   **SQL Schema**:
    ```sql
    CREATE FUNCTION system.get_log_histogram()
    RETURNS TABLE(level TEXT, count INT)
    LANGUAGE SQL AS '
      SELECT level, COUNT(*) AS count
      FROM system.logs
      GROUP BY level;
    ';
    ```

### `system.get_trace_timeline(p_trace_id TEXT)`

*   **Type**: Table-Valued Function (TVF)
*   **Input**: `p_trace_id TEXT`
*   **Returns**:
    ```text
    TABLE(
      span_id TEXT,
      parent_span_id TEXT,
      name TEXT,
      start_time TIMESTAMP,
      end_time TIMESTAMP,
      duration_ms INT,
      status_code TEXT,
      trace_start_time TIMESTAMP,
      trace_end_time TIMESTAMP
    )
    ```
*   **Purpose**: Retrieves all trace spans for a given UUID and calculates Gantt coordinate boundaries to let the frontend chart nested timelines.
*   **SQL Schema**:
    ```sql
    CREATE FUNCTION system.get_trace_timeline(p_trace_id TEXT)
    RETURNS TABLE(
      span_id TEXT,
      parent_span_id TEXT,
      name TEXT,
      start_time TIMESTAMP,
      end_time TIMESTAMP,
      duration_ms INT,
      status_code TEXT,
      trace_start_time TIMESTAMP,
      trace_end_time TIMESTAMP
    )
    LANGUAGE SQL AS '
      SELECT 
        span_id,
        parent_span_id,
        name,
        start_time,
        end_time,
        duration_ms,
        status_code,
        (SELECT MIN(start_time) FROM system.traces WHERE trace_id = p_trace_id) AS trace_start_time,
        (SELECT MAX(end_time) FROM system.traces WHERE trace_id = p_trace_id) AS trace_end_time
      FROM system.traces
      WHERE trace_id = p_trace_id
      ORDER BY start_time ASC;
    ';
    ```
