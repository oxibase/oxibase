# Routing Contract: Generic Database-Driven Router

This contract defines how incoming HTTP requests map to dynamic route handlers, data queries, and Jinja template outputs.

## 1. HTTP Endpoint Routing

Any HTTP request that does not match a hardcoded Rust Auto-API endpoint or WebSocket protocol is evaluated by the dynamic fallback handler.

```text
HTTP Request (Method, Path)
       │
       ▼
Match segment-by-segment against registered routes in `interface.routes`
       │
 ┌─────┴────────────────────────┐
 │                              ▼
 │                        No match found
 │                              │
 │                              ▼
 │                        HTTP 404 Not Found
 ▼
Match found: Route(template_name, context_query)
       │
       ├─► Extract path parameters from route wildcards (e.g., {id})
       ├─► Extract query string parameters (e.g., ?level=ERROR)
       └─► Parse POST payload as JSON (if applicable)
       │
       ▼
Coalesce into Map of parameters (Params)
       │
       ├─► Map into `NamedParams` (e.g., :id, :level)
       └─► Map into Template Context (`params` namespace)
       │
       ▼
Execute `context_query` (if present) -> fetch Rows
       │
       ▼
Map result Rows to JSON array (`data` namespace)
       │
       ▼
Render Template(name) with combined context: { "data": [...], "params": {...} }
       │
       ▼
Return HTTP 200 OK with rendered HTML
```

## 2. Path Matching Protocol

- Segment matching is case-sensitive.
- A path variable starts with `{` and ends with `}`. It matches any sequence of characters within a path segment.
- If multiple route patterns match, the router selects the first match returned by the query `SELECT template_name, context_query, path FROM interface.routes WHERE method = ?`.

## 3. Template Context namespaces

The following JSON namespaces are guaranteed in the Jinja environment rendering context:

- `data`: An array of objects representing the rows returned by the `context_query` (e.g., `[{"id": 1, "name": "Alice"}]`). If no `context_query` is defined or execution returns no results, `data` is an empty list `[]`.
- `params`: A flat object containing all matched path variables, query string parameter values, and POST body fields.
