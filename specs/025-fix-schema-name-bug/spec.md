# Feature Specification: Fix Schema Name Bug

**Feature Branch**: `025-fix-schema-name-bug`  
**Created**: May 23, 2026  
**Status**: Draft  
**Input**: User description: "I am trying to achieve a full fix for the \"Schema Name Bug\" where tables created in specific schemas (like system) are incorrectly stored in the public schema with the schema name prepended to the table name (e.g., system.cron_runs inside the public schema instead of cron_runs inside the system schema).
Here is my plan to achieve this:
1. Update Core Data Structures (src/core/schema.rs):
   - Add schema_name and schema_name_lower fields to the Schema struct.
   - Update SchemaBuilder and Schema::new to accept and store both the schema name and the table name independently.
   - Set a fallback so that if no schema is provided, it defaults to \"public\".
2. Fix Table Creation (src/storage/mvcc/engine.rs & src/executor/ddl.rs):
   - Update MVCCEngine::create_table so it places the table into the schema map corresponding to schema.schema_name_lower, rather than hardcoding it to DEFAULT_SCHEMA (\"public\").
   - Fix the DDL execution layer (execute_create_table and execute_create_table_as_select) so it passes the parsed schema_name and just the base table_name (e.g., cron_runs instead of system.cron_runs) to the SchemaBuilder.
3. Smart Schema Resolution in Engine Methods (src/storage/mvcc/engine.rs):
   - Modifying the StorageEngine trait methods (like table_exists, get_table_schema, update_table_schema) to accept a separate schema_name argument would require massive refactoring across dozens of files and tests. 
   - Instead, my plan is to update the implementations of these methods inside MVCCEngine to parse the string. If the table_name string contains a dot (e.g., \"system.cron_runs\"), it will split it and query the \"system\" schema for \"cron_runs\". If it doesn't, it will query the \"public\" schema.
4. Verify and Test:
   - Ensure that system.tables correctly outputs schema_name = system and table_name = cron_runs.
   - Ensure the entire test suite (make test-all) passes, keeping all existing backward compatibility intact."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create Table in Specific Schema (Priority: P1)

Users (or system initialization scripts) need to create tables in specific schemas (e.g., `system`) and have them properly segregated from the `public` schema.

**Why this priority**: This is the core bug that needs fixing. Without this, namespace segregation is broken and features relying on system tables will fail.

**Independent Test**: Can be fully tested by creating a new test case that executes `CREATE TABLE system.cron_runs (id INT)` and verifies the output of querying `system.tables` (or equivalent system catalog).

**Acceptance Scenarios**:

1. **Given** a running database instance, **When** the user executes `CREATE TABLE system.cron_runs (id INT)`, **Then** the engine should store the table `cron_runs` within the `system` schema internally, not as `system.cron_runs` in the `public` schema.
2. **Given** the table `system.cron_runs` has been created, **When** the user queries the `tables` system table, **Then** it should return a row with `schema_name = 'system'` and `table_name = 'cron_runs'`.

### User Story 2 - Query Table from Specific Schema (Priority: P1)

Users need to query, update, and manage tables in specific schemas by fully qualifying the table name in their queries.

**Why this priority**: Segregation is only useful if the engine can correctly resolve the tables during execution of subsequent queries.

**Independent Test**: Can be tested alongside User Story 1 by executing `SELECT * FROM system.cron_runs` after table creation and ensuring it succeeds.

**Acceptance Scenarios**:

1. **Given** a table `system.cron_runs` exists, **When** the user executes `SELECT * FROM system.cron_runs`, **Then** the engine should correctly resolve the table from the `system` schema and return the results.
2. **Given** a table `cron_runs` exists in the `public` schema, **When** the user executes `SELECT * FROM cron_runs`, **Then** the engine should correctly resolve the table from the `public` schema.

### Edge Cases

- What happens if no schema is specified (e.g., `CREATE TABLE cron_runs`)? It MUST default to the `public` schema.
- What happens if the table name contains multiple dots (e.g., `system.sub.cron_runs`)? The engine should correctly parse the first segment as the schema and the rest as the table name, or return a clear error if multi-level namespaces aren't supported.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Schema struct MUST store the schema name (and its lowercase variant) independently from the base table name.
- **FR-002**: If no schema is explicitly provided during schema creation, the system MUST default the schema name to "public".
- **FR-003**: The storage engine MUST place the table into the schema map corresponding to the defined schema name rather than hardcoding placement into the "public" schema.
- **FR-004**: The DDL execution layer MUST parse fully qualified table names (e.g., `schema.table`) and pass the separated schema name and base table name to the SchemaBuilder.
- **FR-005**: StorageEngine methods (e.g., `table_exists`, `get_table_schema`, `update_table_schema`) in `MVCCEngine` MUST parse incoming table name strings containing dots (e.g., `system.cron_runs`) to correctly target the appropriate schema and base table name, avoiding the need to change the global `StorageEngine` trait signature.

### Key Entities

- **Schema Struct**: The core definition of a table, which will now hold `schema_name` and `schema_name_lower` properties.
- **MVCCEngine**: The core storage component that manages the mapping of schemas to tables, and needs to be updated to support dynamic schema resolution based on string parsing.
- **DDL Executor**: The component responsible for executing `CREATE TABLE` and similar statements, which must now properly separate schema namespaces.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Creating a table `system.cron_runs` and querying `system.tables` returns exactly one row with `schema_name = 'system'` and `table_name = 'cron_runs'`.
- **SC-002**: The entire test suite (`make test-all`) passes, demonstrating that existing backward compatibility and existing schema logic are intact.
- **SC-003**: The engine successfully resolves tables prefixed with schema names (e.g., `schema.table`) in standard CRUD operations.

## Assumptions

- We assume that splitting strings by `.` is sufficient to determine the schema and table name in the engine methods without needing to change the traits across the entire codebase.
- We assume that `public` is the hardcoded default schema name for tables created without a specified schema.