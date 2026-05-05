# Feature Specification: Foreign Key Constraint

**Feature Directory**: `specs/001-foreign-key`  
**Created**: May 05, 2026  
**Status**: Draft  
**Input**: User description: "i want to add a new constraint type, the foreign key. I want to be able to enforce referential integrity in the database"

## Clarifications

### Session 2026-05-05

- Q: How can a user define a foreign key? → A: Both CREATE TABLE and ALTER TABLE should be supported.
- Q: Should the MVP support composite (multi-column) foreign keys, or only single-column foreign keys? → A: Single-column only
- Q: Should foreign key constraints support deferred validation? → A: Immediate only

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Defining a Foreign Key Constraint (Priority: P1)

As a database user, I want to define a foreign key constraint when creating a table or altering an existing table so that I can establish a relationship between columns in two tables.

**Why this priority**: Defining the relationship is the foundation of referential integrity. Without this, no enforcement can occur.

**Independent Test**: Can be tested by parsing `CREATE TABLE` and `ALTER TABLE ADD CONSTRAINT` statements with `FOREIGN KEY` definitions and verifying the resulting AST and logical schema.

**Acceptance Scenarios**:

1. **Given** a `CREATE TABLE` or `ALTER TABLE` statement with a valid `FOREIGN KEY` clause referencing an existing table and column, **When** the statement is parsed and executed, **Then** the constraint is recorded in the schema.
2. **Given** a `CREATE TABLE` or `ALTER TABLE` statement with a `FOREIGN KEY` clause referencing a non-existent table or column, **When** the statement is parsed and executed, **Then** the system returns an error indicating the referenced entity does not exist.
3. **Given** a statement defining a `FOREIGN KEY` where the referencing and referenced columns have incompatible types, **When** the statement is executed, **Then** the system returns a type mismatch error.
4. **Given** an `ALTER TABLE` statement adding a `FOREIGN KEY` to a table with existing data, **When** the existing data violates the constraint, **Then** the system returns an error and does not add the constraint.

---

### User Story 2 - Enforcing Referential Integrity on Insert (Priority: P1)

As a database user, I want the database to prevent me from inserting rows into a table if the foreign key value does not exist in the referenced table.

**Why this priority**: Preventing invalid data entry is the primary purpose of a foreign key constraint.

**Independent Test**: Can be tested by inserting valid and invalid rows into a table with an established foreign key constraint and verifying success or failure.

**Acceptance Scenarios**:

1. **Given** a table with a foreign key constraint and an existing row in the referenced table, **When** I insert a row with a matching foreign key value, **Then** the insertion succeeds.
2. **Given** a table with a foreign key constraint, **When** I insert a row with a foreign key value that does not exist in the referenced table, **Then** the insertion fails with a referential integrity violation error.
3. **Given** a table with a foreign key constraint, **When** I insert a row with a `NULL` foreign key value, **Then** the insertion succeeds (assuming the column is nullable).

---

### User Story 3 - Enforcing Referential Integrity on Delete/Update (Priority: P2)

As a database user, I want the database to prevent me from deleting or updating rows in the referenced table if those rows are currently referenced by a foreign key in another table.

**Why this priority**: This ensures that existing relationships are not broken by modifications to the referenced data.

**Independent Test**: Can be tested by attempting to delete or update referenced rows and verifying that the operation is blocked when appropriate.

**Acceptance Scenarios**:

1. **Given** two tables with a foreign key relationship and an existing linked record, **When** I attempt to delete the referenced row, **Then** the deletion fails with a referential integrity violation error.
2. **Given** two tables with a foreign key relationship and an existing linked record, **When** I attempt to update the primary key of the referenced row, **Then** the update fails with a referential integrity violation error.

---

### Edge Cases

- What happens when a self-referencing foreign key is defined?
- How does the system handle circular foreign key dependencies during table creation?
- How does MVCC handle concurrent transactions where one transaction deletes a referenced row while another inserts a referencing row?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The parser MUST understand `FOREIGN KEY` constraint definitions in both `CREATE TABLE` and `ALTER TABLE` statements.
- **FR-002**: The schema manager MUST store and validate foreign key relationships.
- **FR-003**: The execution engine MUST validate insertions and updates against foreign key constraints immediately during statement execution (no deferred validation).
- **FR-004**: The execution engine MUST validate deletions and updates on referenced tables to ensure no existing foreign key constraints are violated.
- **FR-005**: The system MUST support defining and executing `SET NULL` and `CASCADE` referential actions for `ON DELETE` and `ON UPDATE` triggers.

### Key Entities

- **Constraint Definition**: Represents the AST node for a foreign key constraint definition.
- **Schema Metadata**: Stores the foreign key relationships between tables.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully define foreign key constraints using standard SQL `CREATE TABLE` and `ALTER TABLE` syntax.
- **SC-002**: The database rejects 100% of `INSERT` or `UPDATE` operations that violate a foreign key constraint.
- **SC-003**: The database rejects 100% of `DELETE` or `UPDATE` operations on referenced tables that would orphan existing foreign key records.
- **SC-004**: Performance impact of foreign key validation during `INSERT` is kept to a minimum (e.g., < 15% regression).

## Assumptions

- The referenced column must be a Primary Key or have a Unique constraint.
- The system will support `ON DELETE CASCADE`, `ON DELETE SET NULL`, `ON UPDATE CASCADE` and `ON UPDATE SET NULL` referential actions.
- The MVP only supports single-column foreign key definitions (no composite keys).
- Constraint validation is immediate; `DEFERRABLE` constraints are not supported.