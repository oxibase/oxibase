# Feature Specification: Transaction Management in Procedures

**Feature Branch**: `008-procedure-transactions`  
**Created**: May 09, 2026  
**Status**: Draft  
**Input**: User description: "Transaction management within the procedure (BEGIN, COMMIT)"

## Clarifications

### Session 2026-05-09
- Q: What is the preferred API for exposing `commit` and `rollback` to JS, Python, and Rhai? → A: Global functions for JS and Rhai (`commit()`, `rollback()`), and module functions for Python (`oxibase.commit()`, `oxibase.rollback()`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Commit Transactions Inside a Procedure (Priority: P1)

As a database user, I want to execute `COMMIT` within a stored procedure so that I can persist partial progress of long-running operations or batch inserts without holding locks for the entire duration.

**Why this priority**: Transaction control is essential for complex procedural logic, allowing procedures to commit intermediate results.

**Independent Test**: Can be fully tested by creating a procedure that inserts a row, calls `COMMIT`, and then throws an error or rolls back. The first row should still persist in the database.

**Acceptance Scenarios**:

1. **Given** a database connection, **When** I execute `CALL process_batch();` which performs `INSERT` then `COMMIT`, **Then** the inserted data is visible to other concurrent transactions immediately after the `COMMIT` statement executes, even while the procedure is still running.
2. **Given** a procedure with multiple `COMMIT` statements, **When** it fails halfway through, **Then** all data committed before the failure remains in the database.

---

### User Story 2 - Rollback Transactions Inside a Procedure (Priority: P1)

As a database user, I want to execute `ROLLBACK` within a stored procedure so that I can discard changes if a business rule validation fails or an exception is caught, without affecting prior committed data.

**Why this priority**: Complementary to `COMMIT`, `ROLLBACK` allows graceful error handling and data consistency restoration from within the procedure logic.

**Independent Test**: Can be fully tested by a procedure that inserts data, calls `ROLLBACK`, and finishes. The data should not be visible in the database.

**Acceptance Scenarios**:

1. **Given** a procedure that inserts a row and then executes `ROLLBACK`, **When** I call the procedure, **Then** the row is not persisted in the database.
2. **Given** a procedure that executes `COMMIT` then `INSERT` then `ROLLBACK`, **When** I call the procedure, **Then** the first changes are persisted but the changes after the `COMMIT` are discarded.

---

### User Story 3 - PL/SQL Syntax Support for Transaction Control (Priority: P2)

As a database developer, I want to use standard SQL transaction commands (`COMMIT`, `ROLLBACK`, `BEGIN`) within my `LANGUAGE pl/sql` blocks natively.

**Why this priority**: We recently implemented a PL/SQL native interpreter; transaction control statements need to be parsed and executed by this interpreter.

**Independent Test**: Can be tested via a SQL integration test verifying that `COMMIT` and `ROLLBACK` parse correctly as PL/SQL AST nodes and trigger the appropriate transaction state changes via the `SqlRunner`.

**Acceptance Scenarios**:

1. **Given** a PL/SQL procedure source code containing `COMMIT;` and `ROLLBACK;`, **When** I run `CREATE PROCEDURE`, **Then** the PL/SQL parser accepts it.
2. **Given** a PL/SQL procedure, **When** a `COMMIT` statement is reached during execution, **Then** the interpreter commands the database executor to commit the current active transaction.

### Edge Cases

- What happens if a `CALL` statement is executed inside an explicit transaction block (`BEGIN; CALL proc();`) and the procedure executes `COMMIT`? PostgreSQL behavior applies: The system must throw an error indicating "invalid transaction termination" (transaction control is not allowed in a nested explicit context).
- How does transaction control inside a procedure interact with nested procedure calls (a procedure calling another procedure that commits)?
- What happens if the scripting language (e.g., Rhai or JS) attempts to execute a transaction control statement directly? Does the `SqlRunner` bridge permit it? Yes, we need to expose a function to JS, Python, and Rhai for transaction management.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The PL/SQL parser MUST recognize and parse `COMMIT` and `ROLLBACK` as valid procedural statements.
- **FR-002**: The `ScriptingBackend` interface and `SqlRunner` trait MUST be extended to allow the backend to request the executor to commit or roll back the current transaction.
- **FR-003**: When a `COMMIT` is executed within a procedure, the executor MUST persist all current changes to the MVCC storage and immediately start a new transaction context for the remainder of the procedure execution.
- **FR-004**: When a `ROLLBACK` is executed, the executor MUST discard all uncommitted changes in the current transaction context and start a new transaction context.
- **FR-005**: The system MUST handle the interaction between the caller's transaction and the procedure's transaction statements by matching PostgreSQL semantics: procedure commits directly operate on the current transaction. If the `CALL` statement was executed within an explicit transaction block, executing `COMMIT` or `ROLLBACK` within the procedure MUST throw an error. Autonomous transactions are not supported.

- **FR-006**: The system MUST expose transaction management functions to supported scripting languages. For Javascript (Boa) and Rhai, it MUST expose global functions `commit()` and `rollback()`. For Python, it MUST expose them via the existing native module as `oxibase.commit()` and `oxibase.rollback()`.

### Key Entities

- **`TransactionState` / `TxContext`**: The internal representation of the active transaction in the executor, which must be mutable or replaceable during procedure execution.
- **`PlSqlStatement::Commit` / `PlSqlStatement::Rollback`**: New AST nodes in the PL/SQL parser.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A test suite demonstrating data persistence upon `COMMIT` and data discarding upon `ROLLBACK` within procedures passes successfully (`make test`).
- **SC-002**: The implementation passes all linters (`make lint`) without warnings and introduces no new `unwrap()` calls.
- **SC-003**: The execution of transaction control commands does not leak database locks or violate ACID properties in concurrent execution tests.

## Assumptions

- We assume that `BEGIN` within a procedure is a no-op if a transaction is already implicitly active (as is typical in Postgres).
- It is assumed the storage engine (`storage/`) already has robust MVCC `commit()` and `rollback()` methods that can be safely invoked mid-execution.