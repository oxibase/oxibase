# Feature Specification: Generic Database-Driven Router

**Feature Branch**: `052-generic-db-router`  
**Created**: Sun Jun 21 2026  
**Status**: Draft  
**Input**: User description: "Refactor Oxibase Server to a Generic Database-Driven Router (Zero-Rust Workspace Handlers)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generic Router Dynamic Fallback Rendering (Priority: P1)

Users access the Oxibase Workspace dashboard to manage their database and execute queries. The server handles these requests through a completely generic, dynamic, database-driven template router, loading both routing configurations and Jinja templates directly from the database without any hardcoded Rust endpoints.

**Why this priority**: Core engine architecture change. Represents the fundamental MVP of the refactoring that eliminates dual route definitions and establishes a pure, lightweight engine.

**Independent Test**: Can be verified by running the Oxibase workspace installer, making HTTP GET requests to `/workspace`, `/workspace/sidebar/data`, and `/workspace/sql_editor`, and confirming they render correct HTML pages with a 200 OK status.

**Acceptance Scenarios**:

1. **Given** the database-driven routes and templates are bootstrapped, **When** a user performs a GET request to `/workspace`, **Then** the server matches the path `/workspace` in `interface.routes` and renders `workspace_sidebar_compute.html` with a 200 OK status.
2. **Given** the database-driven routes and templates are bootstrapped, **When** a user performs a GET request to `/workspace/sidebar/data`, **Then** the server executes the associated `context_query` to fetch schema/table lists, and renders `workspace_sidebar_data.html` populated with the database's schema metadata.

---

### User Story 2 - Request Variable Binding & Parameter Forwarding (Priority: P1)

Users filter logs, inspect trace timelines, or view detailed trace hierarchies in the workspace. The generic fallback router extracts request parameters (including path variables like `{trace_id}`, URL query filters like `?search=pizza&level=ERROR`, and POST payload values) and automatically forwards them as named parameters to the `context_query` database block.

**Why this priority**: Required to make complex telemetry and data-grid explorers fully dynamic. Prevents the need for hardcoded parameter parsing in Rust handlers.

**Independent Test**: Verified by checking that a GET request to `/workspace/traces/12345` correctly forwards `trace_id = '12345'` to the context query, extracting the correct spans and rendering the trace Gantt chart.

**Acceptance Scenarios**:

1. **Given** a route defined as `/workspace/traces/{trace_id}` with a `context_query` using named parameter `:trace_id`, **When** a user requests `/workspace/traces/54321`, **Then** the fallback router binds `:trace_id` to `54321`, executes the query, and renders the trace Gantt timeline.
2. **Given** a route defined as `/workspace/observe/logs` with query parameter inputs, **When** a user requests `/workspace/observe/logs?level=ERROR&search=connection`, **Then** the fallback router parses `level` and `search`, forwards them as parameters, and returns only the filtered ERROR logs containing "connection".

---

### User Story 3 - Database-Driven Telemetry & Domain Logic (Priority: P2)

All telemetry grouping, Loki-style log filtering, trace timeline/hierarchy structuring, histograms, and workspace actions are defined as SQL views, PL/SQL blocks, or Rhai scripting functions in the database, avoiding any workspace-specific domain code in Rust.

**Why this priority**: Fully separates the system engine from application-specific domain logic. Allows users to completely customize the workspace experience by updating database seeds without recompiling Rust.

**Independent Test**: Run standard workspace operations and check that the Rust server codebase has zero lines referencing `logs`, `traces`, `Gantt chart`, or workspace form actions in `src/server/`.

**Acceptance Scenarios**:

1. **Given** a SQL view or PL/SQL stored function for calculating log histograms, **When** the logs dashboard is rendered, **Then** the log counts by level are fetched entirely via the database's internal query rather than Rust-side aggregation.

---

### Edge Cases

- **Invalid route pattern or template name**: If the path does not match any pattern in `interface.routes`, or reference a template that does not exist in `interface.templates`, the router MUST return a clean 404 Not Found response or 500 Internal Server Error with user-friendly logs rather than panicking.
- **Malformed inputs / parameter type mismatch**: If path variables or query parameters do not match database constraints or the expected types in the dynamic `context_query`, appropriate database-level or application-level errors MUST be captured gracefully and rendered without crashing the server.
- **Concurrent transactions & MVCC isolation**: Database queries executed during fallback routing must respect the standard transactional and isolation boundaries of the storage engine.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Axum server MUST ONLY expose completely generic endpoints (`/api/rpc/...`, `/api/data/...`, `/api/sql`), WebSocket gateways (like DAP at `/workspace/dap-ws`), and a single fallback template rendering router (`dynamic_route_handler`).
- **FR-002**: The `dynamic_route_handler` MUST match the request HTTP method and path against routes stored in `interface.routes`, supporting parameterized path wildcards (e.g., `{variable}` syntax).
- **FR-003**: The `dynamic_route_handler` MUST extract request parameters from the request path (variables matching `{variable}` wildcards), query strings (URI query parameters), and POST JSON bodies.
- **FR-004**: The `dynamic_route_handler` MUST bind all extracted request variables as named parameters (using the `:` syntax) and execute the route's `context_query` via `db.query_named` or `db.execute_named`.
- **FR-005**: The `dynamic_route_handler` MUST inject all extracted parameters into the Jinja rendering context (under a designated key, e.g., `params`) to ensure they can be referenced inside templates.
- **FR-006**: All workspace-specific application logic (including Loki-style log filtering, trace grouping, histograms, Gantt tree formatting, modal parameters) MUST execute inside the database as SQL views, PL/SQL scripts, or Rhai functions.
- **FR-007**: The codebase in `src/server/` MUST NOT contain any mention of logs, traces, Gantt chart, workspace schemas, or modal forms.

### Key Entities *(include if feature involves data)*

- **`interface.routes`**: Database table storing registered routes, HTTP methods, template names, and corresponding dynamic context queries.
- **`interface.templates`**: Database table storing Jinja HTML template names and contents.
- **`NamedParams`**: The parameter structure holding variables passed to the named context query execution.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of workspace-specific Rust handlers are removed, resulting in zero hardcoded references to logs, traces, Gantt chart, or modal parameters within the `src/server/` directory.
- **SC-002**: All 13+ workspace routes and views render perfectly with active filtering and interactive features using the dynamic fallback router.
- **SC-003**: The project passes `make lint` and `cargo nextest run` without any errors or regressions.

## Assumptions

- Named query parameters in Oxibase SQL can use the standard `:parameter` syntax.
- The workspace database seed installer can bootstrap all required SQL schemas, views, tables, stored procedures, and routing entries successfully without Rust modifications.
