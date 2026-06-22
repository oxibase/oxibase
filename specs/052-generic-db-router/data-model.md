# Data Model: Generic Database-Driven Router

This document defines the schema and key entities required to support completely generic, database-driven dynamic routing and template rendering.

## 1. Schema Definition

All dynamic router configurations are located under the `interface` schema, which is separate from built-in query tables and system telemetry tables.

### Table: `interface.routes`

Stores the route pattern definitions, matching HTTP methods, and associated template and context query logic.

| Column | Type | Nullable | Description |
|---|---|---|---|
| `method` | `TEXT` | No | HTTP Method (e.g., `'GET'`, `'POST'`, `'PATCH'`, `'DELETE'`) |
| `path` | `TEXT` | No | Route path pattern, optionally parameterized using `{var_name}` or `:var_name` wildcards (e.g., `/workspace/traces/{trace_id}`) |
| `template_name` | `TEXT` | No | Foreign key mapping to `interface.templates.name` |
| `context_query` | `TEXT` | Yes | The SQL, PL/SQL block, or procedure call executing to populate the template's rendering context. Can contain `:param_name` named variables matching path/query/POST parameters. |

### Table: `interface.templates`

Stores the Jinja HTML/XML/JSON layout and rendering templates loaded on-demand by the template engine.

| Column | Type | Nullable | Description |
|---|---|---|---|
| `name` | `TEXT` | No | The unique filename/identifier of the template (e.g., `'workspace_layout.html'`) |
| `content` | `TEXT` | No | Full Jinja HTML content |

---

## 2. Parameter Lifecycle & Flow

1. **Extraction**:
   - Path variables are matched and extracted via segment comparison against `interface.routes.path`.
   - Query string filters (e.g. `?search=xyz`) are extracted as standard key-value string pairs.
   - POST bodies (application/json) are parsed as JSON maps.
2. **Coalescing**:
   - All parameters are coalesced into a unified key-value parameter mapping (`HashMap<String, Value>`).
3. **Execution**:
   - The query compiler processes `context_query`.
   - Coalesced parameter map is transformed into `NamedParams` and executed against the database.
4. **Rendering**:
   - The dataset result from the query execution is placed under the `"data"` key of the template context.
   - All coalesced input parameters are placed under the `"params"` key of the template context.
   - MiniJinja renders the template using the combined context.
