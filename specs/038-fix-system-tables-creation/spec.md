# Feature Specification: Fix System Tables Creation

**Feature Branch**: `038-fix-system-tables-creation`  
**Created**: 2026-06-08  
**Status**: Draft  
**Input**: User description: "the same way the cron tables are created always, thhe funcionts, procedures and triggers should be the same"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reliable Initial Boot (Priority: P1)

As a database user or administrator, when I start the database from scratch, the internal system tables (`system.functions`, `system.procedures`, `system.triggers`, `system.table_stats`, `system.column_stats`) must be explicitly created using their predefined exact SQL schema definitions, guaranteeing constraints like `PRIMARY KEY` and `UNIQUE` are in place.

**Why this priority**: Without strict schema enforcement during table creation, features that depend on these constraints (like row ID generation or preventing duplicates) will silently fail or behave unpredictably.

**Independent Test**: Can be fully tested by creating a new database in an empty directory and running `SELECT schema_name, table_name FROM system.tables;` to ensure they exist, followed by verifying their schema definition through introspection.

**Acceptance Scenarios**:

1. **Given** a fresh database instance with no prior data, **When** the executor initializes the system schemas, **Then** all system tables are created correctly using their explicit `CREATE TABLE` definitions.

---

### User Story 2 - Migration of Old Metadata (Priority: P2)

As a database upgrading from a previous version, any existing data in the old metadata tables (`_sys_functions`, `_sys_procedures`, etc.) must be safely migrated into the new `system.*` tables without losing schema constraints.

**Why this priority**: Ensures backward compatibility and zero data loss for existing users when upgrading the database binary.

**Independent Test**: Can be tested by seeding a test database with `_sys_functions`, starting the new engine, and asserting the data has been migrated to `system.functions` and the old table is dropped.

**Acceptance Scenarios**:

1. **Given** an existing database with `_sys_functions`, **When** the database boots up, **Then** the engine inserts the old records into the newly created `system.functions` table and drops `_sys_functions`.

### Edge Cases

- What happens if the `system.*` tables already exist? The creation step is safely skipped.
- What happens if the old `_sys_*` tables do not exist? The migration (`INSERT INTO ... SELECT`) step safely ignores them and does not fail.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Engine MUST create `system.functions` using its explicit `CREATE_FUNCTIONS_SQL` upon startup if it does not already exist.
- **FR-002**: Engine MUST create `system.procedures` using its explicit `CREATE_PROCEDURES_SQL` upon startup if it does not already exist.
- **FR-003**: Engine MUST create `system.triggers` using its explicit `CREATE_TRIGGERS_SQL` upon startup if it does not already exist.
- **FR-004**: Engine MUST optionally migrate data from `_sys_functions`, `_sys_procedures`, `_sys_triggers`, `_sys_table_stats`, and `_sys_column_stats` using `INSERT INTO system.X SELECT * FROM _sys_X` if those legacy tables exist, and subsequently drop them.
- **FR-005**: Engine MUST ensure schema constraints like `PRIMARY KEY AUTO_INCREMENT` and `UNIQUE` are preserved.

### Key Entities

- **System Tables**: Internal tables storing metadata (`system.functions`, `system.procedures`, `system.triggers`, `system.table_stats`, `system.column_stats`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Database boots successfully from scratch and all 5 mentioned system tables exist.
- **SC-002**: Passing all test suites (`make test`) without regressions.
- **SC-003**: Schema metadata correctly shows constraints for the created system tables.

## Assumptions

- Predefined SQL strings (e.g., `CREATE_FUNCTIONS_SQL`) are already correctly defined and accessible in the storage module.
- The executor has standard internal SQL execution methods capable of running the initialization logic.
