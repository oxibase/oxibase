# Data Model: Observability Overhaul

## 1. Logs Data Contracts

### Log Explorer Context Payload (JSON)
Served to the `workspace_observe_logs.html` template:
```json
{
  "logs": [
    {
      "id": 412,
      "timestamp": "2026-06-20T14:02:03Z",
      "level": "ERROR",
      "target": "oxibase::executor::planner",
      "message": "Table 'users' does not exist",
      "json_fields": "{\"query\": \"SELECT * FROM users\", \"code\": \"42P01\"}",
      "trace_id": "9fbc38d10b7a421b",
      "span_id": "b617f1a0e9c3"
    }
  ],
  "filters": {
    "level": "ERROR",
    "search": "users",
    "trace_id": "",
    "limit": 100
  },
  "histogram": {
    "ERROR": 5,
    "WARN": 12,
    "INFO": 154,
    "DEBUG": 310
  }
}
```

---

## 2. Traces Summary Data Contracts

### Trace Summary List Payload (JSON)
Served to the `workspace_observe_traces.html` template:
```json
{
  "traces": [
    {
      "trace_id": "9fbc38d10b7a421b",
      "root_span_name": "procedure.execute",
      "start_time": "2026-06-20T14:02:01Z",
      "end_time": "2026-06-20T14:02:02Z",
      "total_duration_ms": 12.4,
      "span_count": 3,
      "has_error": true
    }
  ],
  "filters": {
    "search": "",
    "status": "errors_only",
    "limit": 100
  }
}
```

---

## 3. Span Timeline Tree Structures

### Hierarchical Tree Node (JavaScript Object)
Generated in-browser from flat span records:
```json
{
  "span_id": "b617f1a0e9c3",
  "parent_span_id": "none",
  "name": "procedure.execute",
  "span_kind": "INTERNAL",
  "start_time": "2026-06-20T14:02:01.000Z",
  "end_time": "2026-06-20T14:02:02.240Z",
  "duration_ms": 12.4,
  "status_code": "ERROR",
  "status_message": "Table 'users' does not exist",
  "attributes": "{\"db.statement\": \"CALL register_user()\"}",
  "events": "[]",
  "children": [
    {
      "span_id": "c19aef3d02b8",
      "parent_span_id": "b617f1a0e9c3",
      "name": "query.execute",
      "duration_ms": 4.2,
      "status_code": "OK",
      "children": []
    }
  ]
}
```
