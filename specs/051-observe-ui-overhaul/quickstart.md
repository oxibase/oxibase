# Quickstart: Observability Overhaul Dashboard

This guide describes how to access, use, and verify the newly overhauled Loki-style Logs Explorer and Tempo-style Trace timelines.

## 1. Accessing the Dashboards

Once installed and running, open the Workspace interface in your browser (typically `http://localhost:8080/workspace`) and head to the **Observe** tab:

1. **Logs Dashboard:** Click "Logs" in the sidebar or navigate to `/workspace/observe/logs`.
2. **Traces Summary:** Click "Traces" in the sidebar or navigate to `/workspace/observe/traces`.
3. **Trace Timelines:** Click any individual `trace_id` badge inside logs or the traces table to load `/workspace/traces/{trace_id}`.

---

## 2. Interactive Controls Guide

### Logs Explorer (`/workspace/observe/logs`)
* **Level Filtering:** Click any of the coloured severity tabs (`ERROR`, `WARN`, `INFO`, `DEBUG`) to isolate logs instantly.
* **Correlated Spans:** Look for the green `🔍 View Trace` button on any log containing database transaction tracing context.
* **Inspect Parameters:** Click a log line to expand its details panel showing full `json_fields` parameters in a clean JSON viewer.

### Gantt Tree Timeline (`/workspace/traces/{trace_id}`)
* **Expand / Collapse:** Parent spans have toggle handles to fold nested sub-queries and triggers.
* **Gantt Bars:** Hover over duration bars to view exact timing tooltips.
* **Side Drawer:** Click a span block to open the attributes side drawer displaying JSON logs and parameters.

---

## 3. Running Verification Tests

To verify that server routes and parameterized SQL filtering behave correctly, run:
```bash
cargo nextest run --test procedure_plsql_tests
```
And check that Workspace formatting passes our quality checks:
```bash
make lint
```
