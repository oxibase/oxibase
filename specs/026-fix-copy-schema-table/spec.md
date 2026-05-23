# Feature Specification: Fix COPY Schema-Qualified Table Syntax

**Feature Branch**: `026-fix-copy-schema-table`
**Created**: 2026-05-23
**Status**: Draft
**Input**: User description: "The oxibase SQL parser's implementation for the COPY statement (parse_copy_statement) was strictly hardcoded to only accept a simple, single-word Identifier for the table name. It did not support the schema.table (qualified) syntax. When you asked to make sure it was loaded into the cdm schema, I modified the Python script to generate COPY cdm.concept FROM .... This immediately failed in oxibase because the parser saw the dot (.) and errored out, expecting the FROM keyword instead."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Load Data into Schema-Qualified Table (Priority: P1)

Users need to load data from external files (like CSVs) directly into tables organized within specific schemas (e.g., `cdm.concept`), using the `COPY ... FROM` statement.

**Why this priority**: Essential for data loading pipelines that organize data into different schemas for logical separation, security, or domain boundaries. Without this, users cannot easily populate namespaced tables using the built-in COPY tool.

**Independent Test**: Can be fully tested by a parser integration test validating that `COPY schema.table FROM 'file'` parses into the correct AST, and an execution test verifying data is loaded into a created schema and table.

**Acceptance Scenarios**:

1. **Given** a schema `cdm` containing a table `concept`, **When** the user executes `COPY cdm.concept FROM 'data.csv'`, **Then** the parser successfully interprets the qualified name without throwing an error at the dot (`.`) and the data is loaded into `cdm.concept`.
2. **Given** a standard table `concept` in the default schema, **When** the user executes `COPY concept FROM 'data.csv'`, **Then** the parser interprets the single identifier correctly (backward compatibility) and the data is loaded into `concept`.

---

### Edge Cases

- What happens when a user provides a three-part identifier (e.g., `db.schema.table`)? It should be handled gracefully (either supported if the parser generally supports it, or rejected with a clear error, not a generic syntax error on the dot).
- What happens when the schema provided does not exist during execution?
- Does quoting work correctly for schema and table names (e.g., `COPY "my schema"."my table" FROM ...`)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The SQL parser MUST support parsing schema-qualified table names (two-part identifiers like `schema.table`) in the `COPY ... FROM` statement.
- **FR-002**: The SQL parser MUST continue to support parsing single-part table names (`table`) in the `COPY ... FROM` statement.
- **FR-003**: The SQL parser MUST correctly handle quoted identifiers for both schema and table parts (e.g., `"schema"."table"`) in the `COPY` statement.
- **FR-004**: The AST generated for the `COPY` statement MUST accurately reflect the table reference, preserving the schema information if provided.
- **FR-005**: The execution engine MUST use the parsed schema name (if provided) to locate the target table for the `COPY` operation.

### Key Entities

- **ObjectName**: The AST node structure representing a potentially qualified database object name (like a table).
- **Copy Statement AST Node**: The syntax tree representation of the parsed `COPY` command.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Statements of the form `COPY schema_name.table_name FROM 'path'` parse successfully 100% of the time, replacing the previous syntax error on the dot character.
- **SC-002**: 100% of existing unit and integration tests for single-identifier `COPY` statements continue to pass without modification.
- **SC-003**: Data is successfully inserted into the correct schema when executing a valid `COPY schema.table FROM ...` command.

## Assumptions

- The underlying storage and execution engines already support schema-qualified table operations for other statements (like `INSERT` or `SELECT`), and only the `COPY` parser logic needs updating.
- The SQL dialect implemented by the parser aligns with standard SQL conventions for qualified identifiers (`schema.table`).