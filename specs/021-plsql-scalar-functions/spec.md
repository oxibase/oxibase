# Feature Specification: PL/SQL Scalar Functions

**Feature Branch**: `[###-plsql-scalar-functions]`  
**Created**: 2026-05-16  
**Status**: Draft  
**Input**: User description: "implement PL/SQL scalar functions! Since Oxibase already has a built-in native PL/SQL parser and interpreter for stored procedures..."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Define and Execute a PL/SQL Scalar Function (Priority: P1)

Users can write inline scalar functions using `LANGUAGE plsql` or `LANGUAGE sql` without having to rely on external scripting engines (like Rhai, JS, or Python), providing a more native, Postgres-like experience.

**Why this priority**: This is the core functionality that provides native scalar function capabilities to the database, removing the need for external dependencies for simple logic.

**Independent Test**: Can be fully tested by a new integration test in `tests/` verifying `make test` output, specifically creating a PL/SQL function and evaluating its output via a `SELECT` query.

**Acceptance Scenarios**:

1. **Given** a running database instance, **When** a user creates a PL/SQL scalar function that computes a value and then executes a `SELECT my_func(args);` query, **Then** the function executes correctly and the correct value is returned in the result set.
2. **Given** a PL/SQL function with control flow (e.g., loops, conditionals) and multiple variables, **When** the function is called with various arguments, **Then** it correctly executes the internal logic and returns the computed result.

### Edge Cases

- What happens when a PL/SQL function does not explicitly execute a `RETURN` statement before the end of its block?
- How are type mismatches handled between the computed expression in the `RETURN` statement and the declared return type of the function?
- How does the interpreter handle early `RETURN` statements from within deeply nested control structures (e.g., inside a loop inside an if-statement)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The PL/SQL AST MUST support capturing an optional `Expression` within the `RETURN` statement node.
- **FR-002**: The PL/SQL Parser MUST be able to identify and parse an expression following the `RETURN` keyword and preceding the statement's terminating semicolon.
- **FR-003**: The PL/SQL Interpreter MUST bubble up an optional `Value` when a `RETURN` statement is encountered during execution.
- **FR-004**: The PL/SQL backend's `execute` trait implementation MUST be fully implemented to parse the code, set up the execution environment with arguments, run the interpreter, and extract the returned value.

### Key Entities

- **PL/SQL AST Node**: The `PlSqlStatement::Return(Token, Option<Expression>)` represents the return statement with its optional return value.
- **Execution Status**: The `ExecutionStatus::Return(Option<Value>)` represents the signal emitted by the interpreter to indicate a return with a specific value.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully define and invoke a native PL/SQL scalar function that returns a value in a standard SQL query.
- **SC-002**: Passes all newly added integration tests verifying PL/SQL function execution, arguments, and return values.
- **SC-003**: Passes the `make lint` check and the `cargo nextest` suite across all features with zero regressions.

## Assumptions

- Existing PL/SQL language features (such as variables, assignments, loops, and conditional logic) will work seamlessly inside scalar functions.
- The feature is fully self-contained within the `src/functions/plsql/` directory as indicated.