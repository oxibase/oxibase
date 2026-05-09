# Feature Specification: Stored Procedures (CREATE PROCEDURE / CALL)

**Feature Branch**: `007-stored-procedures`  
**Created**: May 08, 2026  
**Status**: Draft  
**Input**: User description: "feat: Core implementation for Stored Procedures (CREATE PROCEDURE / CALL)" (from issue #27)


## Clarifications
### Session 2026-05-08
- Q: What about returning a value? What is the difference with a function? -> A: Postgres Style. **Functions** return values and run inside queries (`SELECT func()`). **Procedures** are executed via `CALL`, can manage transactions, and do *not* return values directly (they return data using `OUT` or `INOUT` parameters).
- Q: I would like to have SQL Procedural Language too, add it. How complex? -> A: Full PL/SQL clone.
- Q: Which execution strategy for debugging? -> A: Dedicated PL Interpreter. A separate interpreter makes it easier to implement a user-facing debugger (DAP server) later.
- Q: Shouldn't the table `_sys_procedures` be `system.procedures`? -> A: Yes. The procedures catalog should be exposed in the new `system` schema as `system.procedures`.
- Q: I was expecting in a procedure to be able to perform select/insert/updates. Is this the case? -> A: Yes. The PL/SQL dialect MUST support executing standard SQL statements (INSERT, UPDATE, DELETE, etc.) directly within the procedural blocks. The PL/SQL interpreter must bridge these calls back to the main database executor.


## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and Execute a Stored Procedure (Priority: P1)

As a database user, I want to define a stored procedure with custom logic and execute it later, so that I can encapsulate business rules directly within the database and run them on demand.

**Why this priority**: This is the core functionality. Without the ability to define and execute procedures, the feature does not exist.

**Independent Test**: Can be tested via a SQL integration test that creates a procedure using `CREATE PROCEDURE` and executes it using `CALL`, verifying the result.

**Acceptance Scenarios**:

1. **Given** a database connection, **When** I execute `CREATE PROCEDURE do_something() LANGUAGE rhai AS $$ /* perform side effects, no return */ $$;`, **Then** the procedure is successfully stored in the database catalog.
2. **Given** the `do_something` procedure exists, **When** I execute `CALL do_something();`, **Then** the logic runs and returns an empty successful response.

---

### User Story 2 - Procedure with Arguments (Priority: P2)

As a database user, I want to pass arguments to a stored procedure, so that the logic can operate dynamically on the input data.

**Why this priority**: While parameterless procedures are useful, passing arguments makes them significantly more powerful and practically useful for business logic.

**Independent Test**: Can be tested via a SQL integration test that creates a procedure with arguments and calls it with specific values.

**Acceptance Scenarios**:

1. **Given** a database connection, **When** I execute `CREATE PROCEDURE add_numbers(a INT, b INT, INOUT res INT) LANGUAGE rhai AS $$ res = a + b; $$;`, **Then** it is successfully stored.
2. **Given** the `add_numbers` procedure exists, **When** I execute `CALL add_numbers(10, 5, 0);`, **Then** the logic runs and the `CALL` statement returns a row with the updated `OUT/INOUT` parameter (`res = 15`).

---


### User Story 3 - PL/SQL-like Procedural Logic (Priority: P2)

As a database user, I want to write procedures using a native, standard PL/SQL-like language (`LANGUAGE sql` or `LANGUAGE pl/sql`) so that I can use standard database control flows (IF, WHILE, variables) without relying on external scripting languages like Rhai or Python.

**Why this priority**: It provides a native, familiar, and highly requested way to write procedural logic directly in SQL.

**Independent Test**: Can be tested via a SQL integration test that creates a procedure using `LANGUAGE pl/sql` with variables and an IF/ELSE block, executing it and verifying the result.

**Acceptance Scenarios**:

1. **Given** a database connection, **When** I execute `CREATE PROCEDURE check_val(val INT, INOUT is_positive BOOLEAN) LANGUAGE pl/sql AS $$ BEGIN IF val > 0 THEN is_positive := true; ELSE is_positive := false; END IF; END; $$;`, **Then** the procedure is stored.
2. **Given** the procedure exists, **When** I execute `CALL check_val(5, false);`, **Then** the `CALL` returns `is_positive = true`.

---

### User Story 4 - Execute SQL inside PL/SQL (Priority: P1)

As a database user, I want to execute standard database queries (INSERT, UPDATE, DELETE) natively inside a PL/SQL block, so that I can modify data transactionally alongside my procedural logic.

**Why this priority**: Without the ability to query or modify data, the procedural language is essentially a glorified calculator. Database access is the entire point of a stored procedure.

**Independent Test**: Can be tested via a SQL integration test that creates a table, then defines a PL/SQL procedure that runs an `INSERT` into that table, and calls the procedure.

**Acceptance Scenarios**:

1. **Given** a table `logs` exists, **When** I execute `CREATE PROCEDURE log_event(msg TEXT) LANGUAGE pl/sql AS $$ BEGIN INSERT INTO logs(message) VALUES (msg); END; $$;`, **Then** the procedure is stored.
2. **Given** the procedure exists, **When** I execute `CALL log_event('Hello');`, **Then** the row is inserted into the `logs` table.

### Edge Cases

- What happens if the `LANGUAGE` specified is not supported or not enabled in the current build?
- What happens if the procedure source code contains syntax errors (in the specified language)?
- How does the system handle calling a procedure that does not exist?
- What happens if a procedure is called with the wrong number of arguments or incorrect argument types?
- The system MUST support `CREATE OR REPLACE PROCEDURE` to allow redefining an existing procedure without having to explicitly drop it first.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The parser MUST understand the `CREATE OR REPLACE PROCEDURE <name>([args]) LANGUAGE <lang> AS $$ <source> $$` syntax.
- **FR-002**: The parser MUST understand the `CALL <name>([args])` syntax.
- **FR-003**: The engine MUST persist procedure definitions (name, arguments, language, source code) in the system catalog upon successful creation.
- **FR-004**: The execution engine MUST retrieve the procedure definition from the catalog when a `CALL` statement is executed.
- **FR-005**: The execution engine MUST invoke the appropriate scripting backend (e.g., Rhai) based on the specified language and pass the provided arguments.
- **FR-006**: The execution engine MUST return the updated values of any `OUT` or `INOUT` parameters to the caller as a single-row result set. If there are no such parameters, it returns an empty success response.
- **FR-007**: The system MUST gracefully handle errors from the scripting backend (e.g., runtime errors in Rhai).

- **FR-008**: The system MUST support `LANGUAGE sql`, implementing a dedicated parser and interpreter for a PL/SQL-like procedural language (supporting DECLARE, BEGIN/END, IF/ELSE, loops).
- **FR-010**: The execution engine MUST validate the syntax of the procedure source code using the specified language's parser during the execution of the CREATE PROCEDURE statement. If the syntax is invalid, the creation MUST be aborted and an error returned to the user.
- **FR-011**: The PL/SQL interpreter MUST be able to delegate unrecognized statements back to the main SQL parser, and the backend MUST bridge execution of those statements back to the primary database `Executor`.
- **FR-009**: The PL/SQL interpreter MUST be designed to maintain execution state (call stack, local variables, line numbers) in a way that allows a future Debug Adapter Protocol (DAP) server to attach and step through the code.

### Key Entities

- **Procedure Definition**: The logical representation of a stored procedure (name, signature, language, source code) stored in the metadata catalog.
- **CreateProcedure AST Node**: The parsed representation of the `CREATE PROCEDURE` statement.
- **CallProcedure AST Node**: The parsed representation of the `CALL` statement.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `CREATE PROCEDURE` and `CALL` statements execute successfully in end-to-end integration tests.
- **SC-002**: Arguments can be successfully passed from the SQL `CALL` statement into the scripting environment, and return values can be passed back out.
- **SC-003**: The implementation passes all linters (`make lint`) and introduces no new `unwrap()` or `expect()` calls in the library code.

## Assumptions

- We are starting with `Rhai` as the primary scripting language, given it is the default backend. Other backends (`js`, `python`) follow the same architectural pattern.
- Transaction management *within* the procedure (BEGIN, COMMIT) is deferred to a future milestone (Issue #28).
- Dropping procedures (`DROP PROCEDURE`) is a necessary implicit requirement for lifecycle management, even if not explicitly stated in the core description.
