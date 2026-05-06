# Implementation Analysis: Database-Driven Routes and Jinja Templates

## Overview

The `004-db-routes-jinja` feature introduces a powerful capability to Oxibase: serving dynamic, HTML-rendered web pages where both the routing logic and the templates themselves are stored entirely within the database. By integrating `minijinja` with the Axum web server, the system enables CMS-like behavior. Administrators can create, update, and delete web routes and views using standard SQL commands, and changes take effect instantly without needing a server restart or deployment.

## Implementation Details

### Dependencies and Infrastructure
- Added `minijinja` as an optional dependency tied to the `server` feature in `Cargo.toml`.
- Initialized two system tables upon router creation (`src/server/mod.rs`):
  - `routes.definitions` (columns: `method`, `path`, `template_name`, `context_query`)
  - `templates.source` (columns: `name`, `content`)

### Custom Template Loader
- Created `src/server/template.rs` which provides a custom Minijinja template loader (`db_template_loader`). 
- This loader queries the `templates.source` table dynamically to fetch the content of requested templates. This approach perfectly supports Minijinja features like `{% include %}` and `{% extends %}` out of the box because the engine recursively asks the loader for any referenced template.

### Dynamic Route Handler
- Implemented `dynamic_route_handler` in `src/server/handlers.rs` and registered it as a fallback handler in the Axum router.
- **Routing Resolution**: The handler extracts the HTTP `method` and `path`, and queries `routes.definitions`. If no match is found, it correctly falls back to returning a `404 Not Found`.
- **Dynamic Context**: If the route definition includes a `context_query`, the handler executes the SQL query against the database, serializes the resulting rows into a JSON array, and provides it to the Jinja template under the `data` context variable.
- **Rendering**: The handler spins up a Minijinja environment configured with the database template loader, evaluates the template, and returns the result as an `Html` response.

## Fulfillment of Requirements and User Stories

### User Story 1: View dynamically rendered page based on database route
**Status:** Completed.
- **FR-001 (Intercept and Resolve)**: The `dynamic_route_handler` intercepts requests that bypass the Auto-API routes and looks them up in `routes.definitions`.
- **FR-002 (Jinja Engine Integration)**: Minijinja was successfully integrated and handles HTML rendering.
- **FR-003 (Database Templates)**: Templates are successfully loaded from `templates.source` via the custom Minijinja loader.
- **FR-004 (Dynamic Context)**: The `context_query` is executed and safely converted into JSON before being passed to the template.

### User Story 2: Manage web routes and templates via SQL
**Status:** Completed.
- Because templates and routes are read directly from the database on each request (or fetched dynamically by the loader), any `INSERT`, `UPDATE`, or `DELETE` SQL statement instantly alters the application's behavior. Updates to templates immediately reflect on the next HTTP request, and deleted routes correctly yield 404s.

## Error Handling and Code Standards

- **No Panics**: Replaced potential unwraps during template rendering or database querying with graceful error mappings that return `500 Internal Server Error` containing the error message.
- **Error Types**: Standardized on passing the errors out as HTTP Status Codes.
- **Lints**: All code adheres strictly to the existing codebase conventions and has been checked against `cargo clippy`.

## Testing

The implementation includes comprehensive integration tests in `tests/server_test.rs`:
- `test_dynamic_route_rendering`: Verifies that an inserted template and route definition can be fetched via HTTP, and that a `context_query` is correctly evaluated and passed into the template (e.g., resolving `{{ data[0].name }}`).
- `test_dynamic_route_updates`: Verifies the live update capabilities. Tests an initial insertion, followed by a SQL `UPDATE` changing the template content (verifying the change via HTTP), and finally a SQL `DELETE` to ensure the route is correctly destroyed and returns a `404`.

## Conclusion

The "Database-Driven Routes and Jinja Templates" feature has been fully and successfully implemented according to the specification. It integrates seamlessly into the existing Axum and storage architecture, bringing dynamic CMS functionality natively to the database engine.
