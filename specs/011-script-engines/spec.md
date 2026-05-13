# Feature Specification: Script Engine Event Triggers (Boa and RustPython)

**Feature Branch**: `011-script-engines`  
**Created**: 2026-05-11  
**Status**: Draft  
**Input**: User description: "what about the other scripting engines ?"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Validation Trigger in Python (BEFORE INSERT/UPDATE) (Priority: P1)

Database users familiar with Python need to define validation logic that runs before a row is inserted or updated. If the logic fails (e.g., detecting a negative balance), the trigger can abort the operation, ensuring only valid data enters the database.

**Why this priority**: Ensuring feature parity across all supported scripting languages is critical for user adoption. Python is a heavily requested language.

**Independent Test**: Can be fully tested by creating a `BEFORE INSERT` trigger with `LANGUAGE python` that throws a runtime error when a column is less than zero, then attempting to insert a negative value and verifying the transaction aborts.

**Acceptance Scenarios**:

1. **Given** a table and a `BEFORE INSERT` trigger in Python that throws an error on invalid data, **When** a user inserts invalid data, **Then** the statement aborts, an error is returned, and the data is not inserted.
2. **Given** a table and a `BEFORE INSERT` trigger in Python, **When** a user inserts valid data, **Then** the statement succeeds and the data is successfully inserted.

---

### User Story 2 - Audit Logging in JavaScript (AFTER UPDATE/DELETE) (Priority: P1)

Database administrators familiar with JavaScript need to track changes to critical tables by logging old and new values into a separate audit table automatically whenever a record is updated or deleted.

**Why this priority**: Auditing is a core enterprise database requirement. Providing JavaScript (Boa) support ensures frontend developers can easily write database triggers.

**Independent Test**: Can be fully tested by creating an `AFTER UPDATE` trigger with `LANGUAGE js` that calls `oxibase.execute("INSERT INTO audit...")`, updating a row in the primary table, and querying the audit table to verify the `OLD` and `NEW` values are recorded.

**Acceptance Scenarios**:

1. **Given** a primary table, an audit table, and an `AFTER UPDATE` trigger on the primary table in JavaScript, **When** a user updates a row, **Then** the update succeeds AND a corresponding record is automatically inserted into the audit table containing the `OLD` and `NEW` values.

---

### User Story 3 - Data Transformation in Python/JS (BEFORE INSERT/UPDATE) (Priority: P2)

Users want to automatically normalize or transform data (e.g., lowercase strings) before it is written to the table using Python or JavaScript syntax.

**Why this priority**: Modifying data via the `NEW` proxy object is a key requirement for comprehensive trigger support.

**Independent Test**: Test a `BEFORE INSERT` trigger in Python/JS that modifies the `NEW` record's values (e.g., `NEW.name = "PREFIX_" + NEW.name`), verifying the modified values are written to storage.

**Acceptance Scenarios**:

1. **Given** a `BEFORE INSERT` trigger in Python/JS that modifies the `NEW` record, **When** a user inserts a row with original input, **Then** the modified data is saved to the table instead of the original input.

### Edge Cases

- **Missing State**: Ensure that `OLD` and `NEW` records are represented correctly as `null`/`None` in JS and Python during `DELETE` and `INSERT` events, respectively.
- **Type Coercion**: Ensure that values assigned to the `NEW` object in Python or JS are correctly coerced back to the appropriate internal Oxibase `DataType` without crashing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support the `NEW` and `OLD` row proxy objects within the RustPython execution environment.
- **FR-002**: System MUST support the `NEW` and `OLD` row proxy objects within the Boa (JavaScript) execution environment.
- **FR-003**: System MUST allow Python and JS triggers to mutate the `NEW` object properties and have those changes reflected in the underlying data row.
- **FR-004**: System MUST inject the `oxibase.execute` binding into both Python and JS environments for `AFTER` triggers to perform side effects.

### Key Entities 

- **PyNewRowProxy / PyOldRowProxy**: RustPython proxy objects mapped to the Thread-Local trigger context.
- **JsNewRowProxy / JsOldRowProxy**: Boa Engine proxy objects mapped to the Thread-Local trigger context.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully define and execute `BEFORE` and `AFTER` triggers using `LANGUAGE python` and `LANGUAGE js`.
- **SC-002**: Python and JS triggers can successfully read `NEW` and `OLD` values without cloning the underlying row data (Zero-Copy).
- **SC-003**: Python and JS triggers can successfully mutate the `NEW` object.
- **SC-004**: All Python and JS specific trigger integration tests pass when compiled with `--features python,js`.

## Assumptions

- **Existing Infrastructure**: The AST parsing, DDL execution, catalog storage (`_sys_triggers`), and executor hooks (`execute_row_triggers`) implemented in feature `010-event-triggers` are fully functional and language-agnostic.
- **Thread Locals**: The thread-local state (`CURRENT_NEW_ROW`, etc.) established in `010-event-triggers` is safe to access from the Boa and RustPython execution threads.

## Clarifications

### Session 2026-05-11

- Q: What does "native as possible" mean for Python and JS proxy implementations? → A: Instead of injecting isolated variables named `NEW` and `OLD` in the global scope, they should be exposed via the existing `oxibase` module binding (e.g. `oxibase.NEW` and `oxibase.OLD` or similar namespace injection) for cleaner integration with the context. *Note: Since trigger bodies generally expect `NEW` and `OLD` as top-level globals per SQL standard conventions, we will inject them into the global scope as native-feeling dictionary/object proxies rather than forcing an `oxibase.NEW` prefix, but we will ensure they behave identically to native JS Objects or Python Dictionaries (supporting standard property access `NEW.column_name`). The `oxibase` library will be fully available to interact with the transaction object.*
