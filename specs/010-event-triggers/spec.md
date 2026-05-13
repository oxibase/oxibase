# Feature Specification: Event Triggers (BEFORE/AFTER INSERT, UPDATE, DELETE)

**Feature Branch**: `010-event-triggers`  
**Created**: 2026-05-11  
**Status**: Draft  
**Input**: User description: "feat: Event Triggers (BEFORE/AFTER INSERT, UPDATE, DELETE)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Validation Trigger (BEFORE INSERT/UPDATE) (Priority: P1)

Database users need to define validation logic that runs before a row is inserted or updated. If the logic fails (e.g., detecting a negative balance or invalid format), the trigger can abort the operation, ensuring only valid data enters the database.

**Why this priority**: Data integrity is paramount. If validation fails, we must prevent bad data from persisting at the lowest possible layer.

**Independent Test**: Can be fully tested by creating a `BEFORE INSERT` trigger that throws an error when a column is less than zero, then attempting to insert a negative value and verifying the transaction aborts and the row is not visible in subsequent queries.

**Acceptance Scenarios**:

1. **Given** a table and a `BEFORE INSERT` trigger that throws an error on invalid data, **When** a user inserts invalid data, **Then** the statement aborts, an error is returned to the client, and the data is not inserted.
2. **Given** a table and a `BEFORE INSERT` trigger, **When** a user inserts valid data, **Then** the statement succeeds and the data is successfully inserted.

---

### User Story 2 - Audit Logging (AFTER UPDATE/DELETE) (Priority: P1)

Database administrators need to track changes to critical tables by logging old and new values into a separate audit table automatically whenever a record is updated or deleted.

**Why this priority**: Auditing is a core enterprise database requirement for security and compliance.

**Independent Test**: Can be fully tested by creating an `AFTER UPDATE` trigger that inserts a row into an audit table, updating a row in the primary table, and querying the audit table to verify the `OLD` and `NEW` values are correctly recorded.

**Acceptance Scenarios**:

1. **Given** a primary table, an audit table, and an `AFTER UPDATE` trigger on the primary table, **When** a user updates a row, **Then** the update succeeds AND a corresponding record is automatically inserted into the audit table containing the `OLD` and `NEW` values.
2. **Given** a primary table, an audit table, and an `AFTER DELETE` trigger, **When** a user deletes a row, **Then** the record is deleted AND the `OLD` values are saved to the audit table.

---

### User Story 3 - Data Transformation (BEFORE INSERT/UPDATE) (Priority: P2)

Users want to automatically normalize or transform data (e.g., lowercase strings, calculate derived columns) before it is written to the table.

**Why this priority**: While highly useful, preventing bad data (Validation) and tracking changes (Audit) are more critical starting points for triggers.

**Independent Test**: Test a `BEFORE INSERT` trigger that modifies the `NEW` record's values within the procedural context, verifying the modified values are written to storage instead of the original input.

**Acceptance Scenarios**:

1. **Given** a `BEFORE INSERT` trigger that modifies the `NEW` record, **When** a user inserts a row with original input, **Then** the modified data is saved to the table instead of the original input.

### Edge Cases

- **Recursion**: What happens if a trigger causes an infinite loop (e.g., an `AFTER INSERT` trigger inserts a row into the same table)? The system must implement a recursion depth limit or stack overflow protection.
- **Exceptions**: What happens if a trigger throws an unhandled exception in the procedural language? The current statement/transaction should abort cleanly without leaking state.
- **Missing State**: How are `OLD` and `NEW` records represented during a `DELETE` (`NEW` is null/empty) and `INSERT` (`OLD` is null/empty)? The execution context must handle these safely without panics.
- **Schema Changes**: What happens if the table a trigger references is dropped? Dropping the table should cascade and drop the trigger, or at minimum, the trigger should not leave dangling references that crash the database.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support parsing standard `CREATE TRIGGER` and `DROP TRIGGER` SQL syntax.
- **FR-002**: System MUST intercept DML operations (`INSERT`, `UPDATE`, `DELETE`) at the executor level to fire configured `BEFORE` and `AFTER` triggers.
- **FR-003**: System MUST expose the `NEW` and `OLD` row states to the procedural execution environment (e.g., Rhai, JS, Python) based on the event type.
- **FR-004**: System MUST allow `BEFORE` triggers to modify the `NEW` row representation before it is passed to the storage engine.
- **FR-005**: System MUST abort the active transaction/statement if a trigger explicitly signals an abort or raises an unhandled error during execution.
- **FR-006**: System MUST prevent infinite trigger recursion (e.g., via a maximum call stack depth or trigger execution depth limit).

### Key Entities 

- **Trigger Definition**: Catalog object storing the trigger name, target table, timing (`BEFORE`/`AFTER`), event (`INSERT`/`UPDATE`/`DELETE`), and procedural code.
- **Execution Context (Trigger Context)**: The environment passed to the procedural engine containing the `OLD` row (if applicable) and `NEW` row (if applicable) for the current operation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully define and execute `BEFORE` and `AFTER` triggers for `INSERT`, `UPDATE`, and `DELETE` operations using the supported procedural languages.
- **SC-002**: Triggers can successfully block/abort invalid DML operations, effectively preventing the data from being persisted.
- **SC-003**: Triggers can successfully perform side-effects, such as inserting records into an audit table.
- **SC-004**: Base DML operations (inserts/updates/deletes) on tables *without* triggers experience less than a 5% performance regression.
- **SC-005**: All new functionality passes the existing `make test` suite without introducing `unwrap()` calls in the core execution path.

## Assumptions

- **Transactions**: Transaction management within the storage engine is fully functional, ensuring that if a trigger throws an error, the entire DML statement is rolled back safely.
- **Procedural Engine**: Procedural language backends (Rhai, JS, Python) are already capable of executing generic stored procedure logic; this feature focuses on injecting the trigger context (`NEW`/`OLD`) and hooking into the executor.
- **Row-Level Focus**: Triggers will primarily fire `FOR EACH ROW` (as opposed to `FOR EACH STATEMENT`), as the primary use cases involve referencing `NEW` and `OLD` row values.
