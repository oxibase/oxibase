# Feature Specification: Schema-Qualified Foreign Keys

**Feature Branch**: `051-fix-fk-identifiers`  
**Created**: June 20, 2026  
**Status**: Draft  
**Input**: User description: "solve the fk identifier limitations"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create Table-level and Column-level Foreign Key Constraints referencing Tables in different Schemas (Priority: P1)

Users should be able to create cross-schema relationships easily. For example, a table `orders` in the `public` schema or a custom schema `sales` should be able to define a `FOREIGN KEY` that references `customers` in a separate `crm` schema.

**Why this priority**: Cross-schema foreign keys are an industry standard for multi-tenant and clean modular database architectures. This is the core MVP functionality requested.

**Independent Test**: Can be fully tested by a new integration test in `tests/foreign_key_test.rs` verifying successful execution of table-level and column-level cross-schema `FOREIGN KEY` definitions under `make test`.

**Acceptance Scenarios**:

1. **Given** schema `crm` and `sales` exist, and `crm.customers(id)` exists, **When** executing:
   ```sql
   CREATE TABLE sales.orders (
       id INTEGER PRIMARY KEY,
       customer_id INTEGER,
       FOREIGN KEY (customer_id) REFERENCES crm.customers(id)
   )
   ```
   **Then** table `sales.orders` is created successfully with the cross-schema constraint.

2. **Given** schema `crm` exists, and `crm.customers(id)` exists, **When** executing:
   ```sql
   CREATE TABLE orders (
       id INTEGER PRIMARY KEY,
       customer_id INTEGER REFERENCES crm.customers(id)
   )
   ```
   **Then** table `public.orders` is created successfully with the column-level cross-schema constraint.

---

### User Story 2 - Add Cross-Schema Foreign Key Constraints via ALTER TABLE (Priority: P2)

# Users should be able to add cross-schema `FOREIGN KEY` constraints to an existing table using the `ALTER TABLE` statement.

**Why this priority**: Schema migrations often add foreign keys after tables have been created, especially when importing tables or upgrading applications.

**Independent Test**: Can be tested via integration tests in `tests/foreign_key_test.rs` executing `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ... REFERENCES ...`.

**Acceptance Scenarios**:

1. **Given** schema `crm` exists, table `crm.customers(id)` exists, and table `orders` exists in the `public` schema, **When** executing:
   ```sql
   ALTER TABLE orders ADD CONSTRAINT fk_orders_customer FOREIGN KEY (customer_id) REFERENCES crm.customers(id)
   ```
   **Then** the constraint is successfully added to `public.orders`.

---

### Edge Cases

- **Schema Resolution fallback**: When creating or altering a table and a referenced table in a `FOREIGN KEY` is unqualified (e.g. `REFERENCES customers(id)`), the system should resolve `customers` relative to the *referencing table's* schema (e.g., if the referencing table is `sales.orders`, it resolves to `sales.customers`) rather than defaulting strictly to `"public"`.
- **Invalid schemas/tables**: If the referenced schema or referenced table does not exist, the DDL command must fail gracefully with appropriate error messages (`SchemaNotFound` or `TableNotFoundByName`).
- **Referential Integrity violations across schemas**: Insertion and deletion operations must trigger foreign key constraint validations correctly across different schemas, including cascading deletes or restrict actions.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: SQL parser MUST support dotted paths (e.g. `schema_name.table_name`) in the `REFERENCES` clause for both column-level and table-level `FOREIGN KEY` definitions.
- **FR-002**: System MUST resolve unqualified referenced tables in a `FOREIGN KEY` context using the schema of the referencing table (or the active connection schema) instead of defaulting hardcodedly to the `"public"` schema.
- **FR-003**: Metadata engine MUST persist fully qualified table names for foreign keys and `referenced_by` tables.
- **FR-004**: System MUST maintain cross-schema referential integrity for inserts, updates, and deletes (such as checking parent existence and executing cascades).

### Key Entities

- **TableName**: Represents a simple or dot-separated qualified table identifier in the AST.
- **TableConstraint::ForeignKey**: Represents a table-level foreign key constraint in the AST, holding referencing/referenced columns and the `TableName` representing the foreign table.
- **ColumnConstraint::References**: Represents a column-level reference in the AST, holding the referenced table as a `TableName` and optional referenced column identifier.
- **ForeignKeyMetadata**: The persisted schema metadata holding details about foreign key relationships.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Passes all new and existing `cargo nextest run` test suites.
- **SC-002**: Successfully creates, alters, validates, and cascades foreign keys across custom non-public schemas.
- **SC-003**: No warnings or formatting failures under `make lint`.

## Assumptions

- **Schema Separation**: Users have already created the target schemas using `CREATE SCHEMA` prior to establishing the cross-schema foreign keys.
- **Transaction/MVCC Isolation**: Normal transactional guarantees apply; creating or verifying cross-schema constraints locks/verifies referencing and referenced schemas under the active transaction.
