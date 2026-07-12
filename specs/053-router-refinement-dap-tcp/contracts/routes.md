# Contracts: Telemetry RPC & Standalone TCP DAP Framing

This document details the interface schemas, parameters, and stream framing protocols for the telemetry RPC calls and the standalone TCP DAP listener.

## 1. Telemetry AJAX RPC Contracts

To populate dashboards dynamically, the frontend workspace javascript issues POST requests to `/api/rpc/{procedure_name}`:

### POST `/api/rpc/GET_LOG_HISTOGRAM`

* **Request Headers**:
  `Content-Type: application/json`
* **Request Payload**:
  `{}`
* **Response Payload (JSON)**:
  ```json
  {
    "result": [
      { "level": "ERROR", "count": 12 },
      { "level": "WARN", "count": 5 },
      { "level": "INFO", "count": 250 },
      { "level": "DEBUG", "count": 0 }
    ]
  }
  ```

### POST `/api/rpc/GET_TRACE_TIMELINE`

* **Request Headers**:
  `Content-Type: application/json`
* **Request Payload**:
  ```json
  {
    "p_trace_id": "trace-uuid-1234"
  }
  ```
* **Response Payload (JSON)**:
  ```json
  {
    "result": [
      {
        "span_id": "span-1",
        "parent_span_id": null,
        "name": "root-op",
        "start_time": "2026-06-22T10:00:00.000Z",
        "end_time": "2026-06-22T10:00:01.000Z",
        "duration_ms": 1000,
        "status_code": "OK",
        "trace_start_time": "2026-06-22T10:00:00.000Z",
        "trace_end_time": "2026-06-22T10:00:01.000Z"
      }
    ]
  }
  ```

---

## 2. Standalone TCP DAP Framing Protocol

Incoming connection on port `4711` streams raw bytes representing standard DAP (Debug Adapter Protocol) request/response structures.

```text
+----------------------------------------+
| Content-Length: <length>\r\n\r\n       | <-- Header (ASCII)
+----------------------------------------+
| { "seq": 1, "type": "request", ... }   | <-- JSON Body (<length> bytes)
+----------------------------------------+
```

### Request Flow
1. **Client Sends Frame**: The external debugger sends the standard header and JSON body.
2. **Buffer Accumulation**: The server reads bytes, extracts `Content-Length`, validates completion of payload bytes, and processes the command.
3. **Server Responds**: The server responds directly over the same TCP socket, writing standard DAP responses framed in the exact same format.
