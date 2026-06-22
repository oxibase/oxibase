# Research: Generic Database-Driven Router

## 1. Request Variables Extraction & Fallback Route Matching

### Decision
Implement path pattern matching directly inside the generic `dynamic_route_handler` fallback route in `src/server/handlers.rs`. Extract path variables, query parameters, and POST payloads, then unify them into a single map of bound values.

### Rationale
Axum's fallback handler does not natively have access to Axum path variables because it is invoked when no hardcoded Axum route is matched. Therefore, we must match the request path (e.g., `/workspace/traces/12345`) against the registered pattern paths (e.g., `/workspace/traces/{trace_id}`) in `interface.routes`. 

By splitting the route pattern and the request path by `/`, we can easily compare segments:
- If a segment in the pattern starts with `{` and ends with `}` (or starts with `:`), it matches any value in the corresponding segment of the request path and is extracted as a parameter.
- Otherwise, the segments must be exactly equal.
This approach requires zero extra external router dependencies (100% Rust stdlib), is completely self-contained, and works with standard placeholder styles.

### Alternatives Considered
- **Using Axum Wildcards / Router nesting**: We could define dynamic path routers directly in Axum using `/*path` catch-alls. However, this still requires manual path parsing inside the fallback handler, and wouldn't be as clean or generic as matching directly against DB-registered routes.
- **External routing crate (e.g., `matchit`)**: An external router crate could match routes, but it would add a new dependency. A simple stdlib string-split segment matcher is only a few lines of code, completely robust for our path structure, and adds zero overhead (YAGNI / Ponytail).

---

## 2. Parameter Binding & Executing Named Queries

### Decision
Unify all extracted parameters into a `HashMap<String, Value>`. Use Oxibase's native named query parameters (`NamedParams`) and execute context queries using `db.query_named` or `db.execute_named` with standard parameter placeholders like `:param_name`.

### Rationale
Oxibase already includes a robust `NamedParams` struct and supports executing named queries. This matches our requirements perfectly. Unifying path, query, and POST JSON body parameters into a unified HashMap and passing it as a `NamedParams` argument ensures that SQL statements inside `interface.routes` can securely, flexibly, and safely reference any request inputs.

### Alternatives Considered
- **Positional Query Parameters (`?`)**: Executing context queries with positional parameters is extremely error-prone when mixing path variables, query filters, and POST payloads because the position and number of arguments can vary dynamically. Named parameter binding is clean, self-documenting, and robust.

---

## 3. Shifting Domain Logic into the Database

### Decision
Relocate telemetry dashboard logic (log filtering, trace hierarchies, Gantt timeline processing) into the database itself by utilizing standard SQL queries, database views, or stored scripts/procedures. Bootstrap these database-side elements inside the workspace seed installer (`src/bin/workspace/mod.rs`).

### Rationale
By moving domain aggregations (such as trace durations, error tracking, and log counts) to the database, we can eliminate more than 500 lines of hardcoded, highly specific Rust handler code. This leaves behind a pure, clean, generic server engine and allows the database platform to manage and execute its own operational dashboard dynamically.
