# Feature Specification: Database-Driven Routes and Jinja Templates

**Feature Branch**: `004-db-routes-jinja`  
**Created**: 2026-05-06  
**Status**: Draft  
**Input**: User description: "Database-Driven Routes and Jinja Templates"

### Clarifications

#### Session 2026-05-06
- Q: How should we structure the system tables for future extensibility and support template composition? → A: Use separate schemas (`routes` and `templates`), specifically `routes.definitions` and `templates.source`. Ensure the Jinja engine is configured to load other templates from the database so `{% include %}` and `{% extends %}` work.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View dynamically rendered page based on database route (Priority: P1)

As an end user, I want to visit specific URLs on the Oxibase server and receive HTML pages that are dynamically rendered using Jinja templates, so that the web interface can be fully driven by database content without requiring backend code changes.

**Why this priority**: Core functionality that delivers the primary value of the feature—serving web pages using database-driven routing and templates.

**Independent Test**: Can be tested via integration tests that start the Axum server, insert a dummy route and template into the database via SQL, and then perform an HTTP GET request to verify the rendered HTML output.

**Acceptance Scenarios**:

1. **Given** a route definition in the database (`routes.definitions`) for path `/hello` associated with a simple Jinja template, **When** an HTTP GET request is made to `/hello`, **Then** the server returns a `200 OK` response with the correctly rendered HTML.
2. **Given** a route definition for path `/dashboard` that includes dynamic data mapping, **When** an HTTP GET request is made to `/dashboard`, **Then** the server renders the Jinja template using the live database context and returns the populated HTML.

---

### User Story 2 - Manage web routes and templates via SQL (Priority: P2)

As an administrator or developer, I want to use standard SQL statements (INSERT, UPDATE, DELETE) to manage application routes and their associated templates directly in the database, allowing for instant live updates to the web application.

**Why this priority**: Essential for the "database-driven" aspect, enabling dynamic CMS-like behavior without application restarts.

**Independent Test**: Can be tested by inserting a route, querying it via HTTP, then updating the template via SQL, and verifying that a subsequent HTTP request reflects the updated template immediately.

**Acceptance Scenarios**:

1. **Given** a running server, **When** an administrator executes an `INSERT INTO routes.definitions ...` command, **Then** the new endpoint is immediately available for HTTP requests.
2. **Given** an existing route, **When** an administrator executes a `DELETE` command for that route, **Then** subsequent HTTP requests to that path return a `404 Not Found`.

### Edge Cases

- What happens if a Jinja template contains invalid syntax? (The server should safely catch the rendering error, log it, and return a `500 Internal Server Error` instead of panicking).
- What happens if the dynamic SQL query associated with a route fails to execute? (The server should handle the database error gracefully and return a 500 error).
- How are concurrent updates to the routing table handled by the server's cache or lookups? (Relies on the underlying MVCC of the database to ensure read consistency).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The server MUST intercept incoming HTTP requests and query a dedicated database table (`routes.definitions`) to resolve the requested URL path.
- **FR-002**: The server MUST integrate a Jinja-compatible templating engine (such as MiniJinja or Tera) to render HTML responses. The engine MUST be capable of loading included/extended templates from the database.
- **FR-003**: The system MUST store Jinja templates directly in the database within a dedicated `templates.source` table.
- **FR-004**: The system MUST provide dynamic context to templates by executing an associated SQL query defined in the `routes.definitions` table and passing the resulting data to the Jinja engine.

### Key Entities

- **Route Definition**: A database record mapping an HTTP method and path pattern to a template (and potentially a data-fetching SQL query).
- **Template Context**: The dynamic JSON or Rust data structure built from database query results that is passed into the Jinja engine during rendering.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new route and template can be added via SQL and successfully served via HTTP with 0 downtime or server restarts.
- **SC-002**: Template compilation and rendering overhead adds less than 5ms to request latency compared to standard JSON API endpoints.
- **SC-003**: All new code passes `make lint` with zero warnings, uses proper Rust `Result` types for error handling without any `unwrap()` or `expect()`.

## Assumptions

- We will use an established, fast Rust Jinja engine (like MiniJinja) that minimizes allocations.
- A dedicated system table (`routes.definitions`) will be used to store routing configurations.
- The web server architecture from the recent `003-auto-api-layer` (Axum) is the foundation for these routes.
