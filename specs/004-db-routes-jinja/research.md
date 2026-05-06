# Research: Database-Driven Routes and Jinja Templates

## Templating Engine Selection

**Decision**: Use `minijinja` for template rendering.

**Rationale**:
- **Dynamic Compilation**: We need to load templates at runtime from the database. `minijinja` compiles templates extremely quickly at runtime, unlike `askama` which requires ahead-of-time compilation.
- **Performance & Constitution**: `minijinja` is built for high performance and low overhead, minimizing allocations and executing fast. This strictly aligns with the "Zero-Copy Unikernel Efficiency" principle.
- **Integration**: It has excellent integration with `serde`, which we already use for JSON serialization of our query results in the `server` module.

**Alternatives considered**:
- `tera`: A popular Jinja port in Rust, but it is heavier, has more dependencies, and is generally slower to instantiate than `minijinja`.
- `askama`: Extremely fast because it compiles templates into Rust code, but impossible to use for our use case since templates are defined dynamically in the database at runtime.

## Server Integration Approach

**Decision**: Integrate Jinja rendering into the existing Axum router via a wildcard/fallback route or by dynamically building routes if the routing table allows parameter matching. Given dynamic insertion without server restart is a requirement (SC-001), a wildcard route that performs a database lookup is the most robust approach.

**Rationale**: 
- A wildcard `GET /*path` and `POST /*path` route can intercept requests, look up the `path` in `system_routes`, execute the `context_query`, load the template from `system_templates`, render it, and return the HTML.
- Ensures zero downtime when adding new routes.

**Alternatives considered**:
- **Rebuilding the Axum Router**: Trying to rebuild and swap the Axum router on every database change is highly complex and error-prone in a running Tokio application. Wildcard lookup is much safer and easier to maintain.
