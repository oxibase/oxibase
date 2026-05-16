# Feature Specification: Public Schema Fallback

**Feature Branch**: `018-public-schema-fallback`  
**Created**: May 16, 2026  
**Status**: Draft  
**Input**: User description: "implement public schema fallback for all database objects"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create Object without explicitly setting a schema (Priority: P1)

As a database user, I want to create a database object (like a table, procedure, function, view, or trigger) without explicitly specifying a schema, and have the system automatically assign it to the default `public` schema.

**Why this priority**: Ensuring that all objects fall back to a consistent default schema provides standard behavior expected in SQL-compliant databases (similar to PostgreSQL) and avoids issues where objects have a `NULL` schema.

**Independent Test**: Can be fully tested by creating integration tests for each object type (table, view, procedure, function, trigger), creating them without specifying a schema, and verifying they are accessible under the `public` schema.

**Acceptance Scenarios**:

1. **Given** a connected session, **When** I run `CREATE FUNCTION my_func() RETURNS INT ...`, **Then** the function is created and assigned to the `public` schema instead of a `NULL` schema.
2. **Given** a connected session, **When** I run `CREATE PROCEDURE my_proc() ...`, **Then** the procedure is stored under the `public` schema.
3. **Given** a connected session, **When** I run `CREATE VIEW my_view AS SELECT 1`, **Then** the view is accessible via the `public` schema.
4. **Given** a connected session, **When** I run `CREATE SEQUENCE my_seq`, **Then** the sequence is assigned to the `public` schema.

---

### User Story 2 - Resolve Object References against the Public Schema (Priority: P2)

As a database user, I want to reference objects without their schema prefixes, and have the system automatically resolve them using the current schema (defaulting to `public`).

**Why this priority**: Users expect simple names (`my_table`, `my_view`) to resolve correctly against the default schema, especially after objects are no longer globally scoped.

**Independent Test**: Integration tests to drop, alter, or select from objects using un-prefixed names to ensure resolution correctly defaults to the active/public schema.

**Acceptance Scenarios**:

1. **Given** a function `public.my_func`, **When** I run `DROP FUNCTION my_func`, **Then** the system successfully resolves and drops the function from the `public` schema.

---

### Edge Cases

- What happens when a user explicitly uses `NULL` as a schema prefix? (Syntax error should prevent this, but the executor must ensure no `NULL` schema persists for logical objects).
- How do system tables (like `system.cron` or `system.triggers`) behave? They should continue to use the explicit `system` schema.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The executor MUST assign the active session schema (defaulting to `public`) to new procedures and functions if no schema is explicitly provided.
- **FR-002**: The storage engine MUST route `views` and `sequences` to their respective schemas instead of using a single global hash map.
- **FR-003**: The storage engine MUST fall back to the active schema (or `public`) when resolving, creating, or dropping views and sequences without explicit schema prefixes.
- **FR-004**: The executor MUST extract the schema from the target table for triggers or fall back to the active schema (defaulting to `public`), rather than inserting a `NULL` schema into the `system.triggers` table.
- **FR-005**: All existing schema-aware components MUST consistently respect `ctx.current_schema().unwrap_or("public")`.

### Key Entities

- **[Schema]**: Internal representation of a schema, routing objects (tables, views, sequences) correctly.
- **[StoredProcedure & StoredFunction]**: Metadata structs representing stored code, which will no longer have `NULL` as their schema.
- **[ViewDefinition & SequenceState]**: Objects that need to be organized per-schema inside `MVCCEngine`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of newly created objects without a schema prefix are correctly assigned to the `public` schema (or active schema context).
- **SC-002**: Zero `NULL` schemas present in `system.procedures` or `system.functions` after executing object creation without a prefix.
- **SC-003**: Passes all existing `make test` suites and new integration tests verifying schema fallback for all object types.

## Assumptions

- We assume `system.cron` objects (scheduled jobs) are globally scoped and are exempt from standard schema namespacing.
- We assume `public` is the literal string identifier for the default schema as per `DEFAULT_SCHEMA`.
