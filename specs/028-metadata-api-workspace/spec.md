# Feature Specification: Metadata API and Workspace App

**Feature Branch**: `028-metadata-api-workspace`  
**Created**: 2026-05-24  
**Status**: Draft  
**Input**: User description: "I want to create a workspace app... in stead of using vue.js, i want to use an EXTREMELY thin layer of JS and let the endpoint to do most of the heavylifting. can you use Unpoly ?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Database Schema Explorer (Priority: P1)

As a database user, I want to view all schemas, tables, views, columns, functions, stored procedures, triggers, constraints, indexes, and primary keys in a graphical interface so that I can easily understand the database structure without writing SQL queries.

**Why this priority**: Discoverability is the foundation of any database GUI. Without the ability to explore the schema, other features lack context.

**Independent Test**: Can be fully tested by starting the server and accessing the `GET /api/meta/tables` and `GET /api/meta/columns` endpoints via curl or a test client to verify they return correctly formatted JSON metadata.

**Acceptance Scenarios**:

1. **Given** a database with multiple schemas, tables, views, procedures, and triggers, **When** I make a `GET` request to `/api/meta/tables` (or `/views`, `/functions`, `/schemas`, `/constraints`, `/indexes`), **Then** I receive a JSON list of objects grouped by schema.
2. **Given** a specific table `public.users`, **When** I make a `GET` request to `/api/meta/columns?table_id=public.users`, **Then** I receive a JSON list of columns, their data types, and nullability.
3. **Given** the workspace app is loaded, **When** I view the sidebar, **Then** I see an expandable tree view of schemas and their respective tables, views, functions, stored procedures, triggers, indexes, and constraints.

---

### User Story 2 - Raw SQL Execution Editor (Priority: P1)

As a database user, I want a text editor where I can write and execute arbitrary SQL queries (DDL and DML) and see the results instantly in a data grid.

**Why this priority**: A SQL IDE is fundamentally built around a query editor. This is essential for advanced operations not covered by the GUI.

**Independent Test**: Can be tested by sending `POST /api/sql` requests with various `SELECT`, `CREATE`, and `INSERT` statements and verifying the JSON response structure (rows/columns or affected_rows).

**Acceptance Scenarios**:

1. **Given** an active database connection, **When** I send `{"query": "SELECT 1 as num"}` to `POST /api/sql`, **Then** I receive a JSON response with `columns: ["num"]` and `rows: [{"num": 1}]`.
2. **Given** an active database connection, **When** I send `{"query": "CREATE TABLE test (id INT)"}` to `POST /api/sql`, **Then** the table is created and I receive `{"rows_affected": 0}`.
3. **Given** the workspace app, **When** I type a valid `SELECT` query in the Query Editor tab and click "Run", **Then** the results are displayed in a dynamic data grid below the editor.

---

### User Story 3 - Visual Table and Column Management (Priority: P2)

As a database user, I want to create new tables, drop existing tables, and add columns to tables using visual forms instead of writing DDL SQL.

**Why this priority**: Core GUI functionality for schema management. Lowers the barrier to entry for users who don't want to memorize DDL syntax.

**Independent Test**: Can be tested by sending `POST /api/meta/tables`, `DELETE /api/meta/tables/{schema}.{name}`, and `POST /api/meta/columns` requests and verifying the schema changes reflect in subsequent metadata GET requests.

**Acceptance Scenarios**:

1. **Given** the workspace app, **When** I use the "Create Table" modal and submit a name and columns, **Then** the `POST /api/meta/tables` API generates and executes the correct `CREATE TABLE` statement.
2. **Given** an existing table, **When** I click the delete icon in the sidebar, **Then** the `DELETE /api/meta/tables/{schema}.{name}` API generates and executes the correct `DROP TABLE` statement.
3. **Given** the Schema Tab for a table, **When** I use the "Add Column" form, **Then** the `POST /api/meta/columns` API generates and executes the correct `ALTER TABLE ... ADD COLUMN` statement.

---

### User Story 4 - Visual Data Management (Priority: P2)

As a database user, I want to view, insert, update, and delete rows in a specific table using a spreadsheet-like data grid.

**Why this priority**: Essential for managing actual data within the tables easily.

**Independent Test**: Can be tested by hitting the namespaced `/api/data/{table}` endpoints (GET, POST, PATCH, DELETE) and verifying data changes.

**Acceptance Scenarios**:

1. **Given** an existing table with data, **When** I click it in the sidebar, **Then** the Data Tab fetches from `GET /api/data/{schema}.{table}` and displays the rows in a grid.
2. **Given** the Data Tab, **When** I fill out the "Add Row" form and submit, **Then** a `POST /api/data/{schema}.{table}` request is made and the new row appears in the grid.
3. **Given** a selected table, **When** I query the data API, **Then** the backend `table_exists` validation correctly parses the `schema.table` format.

---

### User Story 5 - Automated App Deployment (Priority: P3)

As a database administrator, I want to deploy the Workspace app directly into the database using a CLI command executed on demand, without requiring external static file hosting.

**Why this priority**: Ensures the workspace app is fully self-contained within the Oxibase ecosystem, but its installation is explicit and opt-in.

**Independent Test**: Can be tested by executing the `install-workspace` command via the CLI and verifying that `GET /workspace` successfully serves the HTML application.

**Acceptance Scenarios**:

1. **Given** a fresh database, **When** I execute the `install-workspace` CLI command, **Then** the `routes` and `templates` schemas/tables are created and seeded.
2. **Given** the executed script, **When** I navigate to `http://localhost:8080/workspace`, **Then** the Unpoly-driven HTML application is served correctly.

### Edge Cases

- What happens when a user submits an invalid SQL query to `/api/sql`? The API should return a standardized JSON error response with a 400 or 500 HTTP status code.
- How does the system handle table names with special characters or reserved keywords? The backend Metadata API must properly quote identifiers when generating SQL (e.g., `CREATE TABLE "my-table"`).
- What if a user tries to access a table in a schema that doesn't exist? The `/api/data/{table}` endpoint should return a clear 404 Not Found error.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST expose a set of RESTful endpoints under `/api/meta/*` for listing schemas, tables, columns, views, functions, stored procedures, triggers, constraints, and indexes.
- **FR-002**: The system MUST expose a `POST /api/meta/tables` endpoint that accepts JSON and executes a `CREATE TABLE` statement.
- **FR-003**: The system MUST expose a `DELETE /api/meta/tables/{id}` endpoint that executes a `DROP TABLE` statement.
- **FR-004**: The system MUST expose a `POST /api/meta/columns` endpoint that accepts JSON and executes an `ALTER TABLE ... ADD COLUMN` statement.
- **FR-005**: The system MUST namespace existing data operations (GET, POST, PATCH, DELETE) under `/api/data/{table}` instead of `/api/{table}`.
- **FR-006**: The system MUST expose a `POST /api/sql` endpoint that accepts a raw SQL query string and returns either the result set (rows and columns) or the number of affected rows.
- **FR-007**: The `table_exists` validation function MUST correctly parse schema-qualified table names (e.g., `schema.table`), defaulting to the `public` schema if none is provided.
- **FR-008**: The system MUST provide an `install-workspace` CLI command (or script meant to be executed via the CLI on demand) that inserts Unpoly-driven HTML pages into the database's template engine and maps them to the `/workspace` route.
- **FR-009**: The Workspace application MUST use Unpoly to handle client-side interactions (like partial page updates and modals) while relying on Minijinja templates on the server to render the HTML.

### Key Entities

- **Metadata API Payload**: JSON structures representing table definitions (name, columns) and column definitions (name, type, nullability).
- **SQL Execution Response**: JSON structure representing either a result set (`{columns: [], rows: []}`) or an execution status (`{rows_affected: 0}`).
- **Template Source**: Database row containing the raw HTML/JS/CSS of the Workspace application.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully list, create, and drop tables entirely via the `/api/meta/tables` endpoints without writing manual SQL.
- **SC-005**: Users can list schemas, views, functions, stored procedures, triggers, constraints, and indexes using their respective `/api/meta/*` endpoints.
- **SC-002**: Users can execute arbitrary `SELECT` and `CREATE` statements via the `/api/sql` endpoint and receive correctly formatted JSON responses.
- **SC-003**: The Workspace application loads successfully and renders the database schema tree in its sidebar using Unpoly for seamless navigation.
- **SC-004**: Existing data API functionality (CRUD operations on tables) works correctly under the new `/api/data/*` namespace.

## Assumptions

- The frontend will rely on CDNs for Unpoly and TailwindCSS.
- The server will render HTML fragments using Minijinja to satisfy Unpoly's requirements for partial updates, shifting logic from the client to the server.
- Authentication/Authorization is out of scope for this iteration and will be handled by future RBAC/ABAC features.
- The Metadata API will initially support basic table creation (columns and types) and will not cover complex constraints (foreign keys, checks) in the POST operations in the first iteration, but WILL support listing them via GET endpoints.
- The raw SQL endpoint will execute queries within the context of the HTTP request, without persistent multi-statement transaction state across separate API calls.