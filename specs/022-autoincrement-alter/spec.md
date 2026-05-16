# Feature Specification: autoincrement-alter

**Feature Branch**: `022-autoincrement-alter`  
**Created**: May 16 2026
**Status**: Draft  
**Input**: User description: "i want to be able to add support to autoincrement and other constraints"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add AUTOINCREMENT to existing column (Priority: P1)

As a database administrator, I want to add an AUTOINCREMENT constraint to an existing INTEGER primary key column using ALTER TABLE, so that I don't have to manually recreate the table and migrate data when I realize I need auto-generation.

**Why this priority**: Modifying columns to be auto-incremental is the core requirement requested by the user.

**Independent Test**: Can be tested by creating a table without AUTOINCREMENT, running the ALTER TABLE command, and then verifying that subsequent INSERTs without the ID automatically generate sequential IDs.

**Acceptance Scenarios**:

1. **Given** a table `users` with an `id INTEGER PRIMARY KEY` column, **When** executing `ALTER TABLE users MODIFY COLUMN id INTEGER AUTOINCREMENT`, **Then** the column is updated with the AUTOINCREMENT constraint.
2. **Given** the modified `users` table, **When** executing `INSERT INTO users (name) VALUES ('Alice')`, **Then** the row is inserted with `id = 1` (or next in sequence).

---

### User Story 2 - Add other constraints (e.g., UNIQUE, CHECK) via ALTER TABLE (Priority: P2)

As a database user, I want to be able to add other standard constraints like UNIQUE or CHECK using ALTER TABLE, so that I can evolve my schema and enforce data integrity over time without table recreation.

**Why this priority**: Expanding `ALTER TABLE` to handle other constraints makes schema evolution more robust and aligns with the user's request for "other constraints".

**Independent Test**: Can be tested by executing ALTER TABLE to add a UNIQUE or CHECK constraint, and verifying that violating inserts are subsequently rejected.

**Acceptance Scenarios**:

1. **Given** a table `users` with an `email TEXT` column, **When** executing `ALTER TABLE users MODIFY COLUMN email TEXT UNIQUE`, **Then** the column is updated with the UNIQUE constraint.
2. **Given** the modified `users` table, **When** attempting to insert a duplicate email, **Then** a constraint violation error is returned.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The executor MUST apply constraints parsed during `ALTER TABLE ... MODIFY COLUMN` to the target column definition.
- **FR-002**: The `AUTOINCREMENT` constraint MUST be supported in `ALTER TABLE ... MODIFY COLUMN` statements.
- **FR-003**: The engine MUST update the schema with the newly modified column definition, preserving existing column attributes that weren't modified unless explicitly overridden.
- **FR-004**: System MUST NOT allow modifying a column to `AUTOINCREMENT` if it is not an integer type.

### Key Entities *(include if feature involves data)*

- **`AlterTableStatement`**: The AST node that contains the `column_def` with its parsed constraints.
- **`ColumnConstraint`**: The AST enum representing the constraints (e.g., `AutoIncrement`, `Unique`).
- **`Schema` / `ColumnDefinition`**: The core data structures that must be updated to reflect the new constraints.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully run `ALTER TABLE ... MODIFY COLUMN ... AUTOINCREMENT` without syntax errors or silent failures.
- **SC-002**: Data inserted into a column modified to `AUTOINCREMENT` automatically receives sequence numbers.
- **SC-003**: Passes all new and existing `make test` suites.

## Assumptions

- We assume the parser already correctly parses constraints like `AUTOINCREMENT`, `UNIQUE`, etc. in the `ALTER TABLE ... MODIFY COLUMN` statement (reusing the `parse_column_definition` logic), and the limitation is strictly in the executor engine ignoring them.
