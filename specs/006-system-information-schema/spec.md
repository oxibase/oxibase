# Feature Specification: System Information Schema

**Feature Branch**: `006-system-information-schema`  
**Created**: May 07, 2026  
**Status**: Draft  
**Input**: "Can you reorder the internal tables to use the information schema as the place to store all the information or a new system schema ? I want as much of the metadata of the database to be stored in itself and to be exposed either through information schema for "public" information or "system" for internals. the idea is that by querying the system.table you could debug as much as possible"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Querying Public Metadata (Priority: P1)

As a database user or external tool, I want to query standard metadata about tables, columns, and views using the standard SQL `information_schema` so that my existing tools and drivers can introspect the database.

**Why this priority**: Essential for compatibility with ecosystem tools (ORMs, BI tools, database clients) that rely on `information_schema` to discover database structure.

**Independent Test**: Can be tested via SQL queries against `information_schema.tables` and `information_schema.columns` returning the expected rows based on the user-created tables.

**Acceptance Scenarios**:

1. **Given** an empty database, **When** a user creates a table `CREATE TABLE my_table (id INT)`, **Then** `SELECT * FROM information_schema.tables WHERE table_name = 'my_table'` should return exactly one row detailing the table.
2. **Given** the table `my_table`, **When** the user queries `SELECT * FROM information_schema.columns WHERE table_name = 'my_table'`, **Then** it should return a row describing the `id` column with its type.

---

### User Story 2 - Querying Internal Debug Information (Priority: P2)

As a database administrator or developer, I want to query a new `system` schema to access deep internal metadata and state (e.g., transaction status, storage page info, locks, query plans, metrics) so that I can debug and monitor the database engine using standard SQL.

**Why this priority**: Crucial for observability, troubleshooting, and performance tuning of the database internals without needing specialized external tools.

**Independent Test**: Can be tested by executing specific transactions or operations and then verifying the resulting state changes in the relevant `system.*` tables.

**Acceptance Scenarios**:

1. **Given** the database is running, **When** I execute `SELECT * FROM system.tables`, **Then** it returns a list of all tables including internal system tables.
2. **Given** an active transaction, **When** I query `SELECT * FROM system.transactions`, **Then** I should see the currently running transaction listed.

---

### User Story 3 - Exposing Internal State (Priority: P3)

As a system architect, I want the core database metadata and active engine state to be exposed as virtual tables within a `system` schema, so that I can easily query the real-time memory structures of the database.

**Why this priority**: Provides deep visibility into the engine's live state without requiring custom debugging endpoints or complex memory dumps.

**Independent Test**: Can be tested by creating a table and immediately querying `system.tables` to see the real-time memory state reflect the new table.

**Acceptance Scenarios**:

1. **Given** a running database, **When** the system initializes, **Then** queries to `system.tables` dynamically read from the active in-memory catalog.
2. **Given** a schema change (e.g., `ALTER TABLE`), **When** the transaction commits, **Then** subsequent queries to `system.columns` immediately reflect the updated in-memory state.

### Edge Cases

- What happens if a user tries to `DROP` or `ALTER` a table in the `information_schema` or `system` schema? (Should be rejected with a clear error).
- How is the "chicken and egg" problem solved during database bootstrap where system tables need to exist to store their own metadata?
- What happens if a user creates a table named `tables` in their default schema? Does it conflict with `system.tables`?
- How are permissions handled for `system` schema tables compared to `information_schema`?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST implement a read-only `information_schema` containing standard views/tables like `tables`, `columns`, and `views` conforming broadly to SQL standards.
- **FR-002**: The system MUST implement a `system` schema exposing internal database state and metadata.
- **FR-003**: The system MUST expose its internal in-memory metadata and active state as virtual tables under the `system` schema.
- **FR-004**: The system MUST NOT allow users to directly modify (INSERT/UPDATE/DELETE) data in the `information_schema` or `system` schema.
- **FR-005**: The query executor MUST intercept queries to the `system` and `information_schema` namespaces and route them to dynamic virtual table generators.
- **FR-006**: Queries to `information_schema` MUST filter results based on the current user's privileges (if authorization is implemented).

### Key Entities

- **`system.tables`**: The canonical storage for all table definitions in the database.
- **`system.columns`**: The canonical storage for all column definitions.
- **`information_schema`**: A logical schema containing views or tables providing a standardized, "public" view of metadata.
- **`system`**: A schema containing the raw metadata tables and other debug/internal views.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can successfully connect with a standard third-party SQL client (like DBeaver or psql-equivalent if compatible) and browse the database schema using `information_schema` queries.
- **SC-002**: Internal engine state (tables, columns, etc.) is successfully queryable via the `system` schema without impacting the performance of the core storage engine.
- **SC-003**: An attempt to execute `DROP TABLE system.tables` or `INSERT INTO information_schema.tables ...` results in a clear error message and no state change.
- **SC-004**: System startup remains fast, with no complex bootstrapping loops required for metadata.

## Assumptions

- The `information_schema` implementation will prioritize the most commonly used tables (`tables`, `columns`) first, aiming for broad compatibility rather than exhaustive implementation of the entire SQL standard immediately.
- The `system` schema will initially contain metadata storage but can be expanded later to include dynamic views (e.g., active queries, memory usage).
- Access control to `system` tables will be restricted to administrative roles if/when a robust role-based access control (RBAC) system is implemented; otherwise, it will be globally readable.