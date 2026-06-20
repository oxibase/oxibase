# Feature Specification: PL/SQL Table Type and FOR Loop Iteration

**Feature Branch**: `050-plsql-table-iteration`  
**Created**: June 20, 2026  
**Status**: Draft  
**Input**: User description: "add a sugar syntax data type named TABLE to represent rows of json object, and support a grammar similar to FOR row IN my_table LOOP"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Executing SQL and Storing Results in Procedural Languages (Priority: P1)

Developers writing stored procedures and user-defined functions in Rhai, Python, and PL/SQL need a unified, idiomatic way to execute raw SQL queries, retrieve the resulting rows, and manipulate them inside procedural variables.

**Why this priority**: Core integration requirement. Provides parity across all scripting backends for executing query statements and returning values back into variables.

**Independent Test**: Verified via integration tests in `tests/rhai_scripting_test.rs` and `tests/procedure_multilang_tests.rs`.

**Acceptance Scenarios**:
1. **Given** a Rhai procedure execution, **When** calling `oxibase::query("SELECT 100 as val;")`, **Then** the result must be returned as an Array of Maps, and accessing `rows[0]["val"]` must return `100`.
2. **Given** a Python procedure execution, **When** calling `oxibase.query("SELECT 200 as val;")`, **Then** the result must be returned as a List of Dictionaries, and accessing `rows[0]["val"]` must return `200`.
3. **Given** a PL/SQL procedure execution, **When** calling `query_value('SELECT 300')`, **Then** the result must be a single scalar integer `300` assignable to a procedural variable.

---

### User Story 2 - PL/SQL `TABLE` Type Declaration (Priority: P1)

Database developers writing PL/SQL procedures want a sugar-syntax type named `TABLE` to represent collections of database rows. This type acts as an alias for `JSON` under the hood, enabling seamless document-based row representation with type clarity.

**Why this priority**: High priority as it introduces clean type notation for multi-row results without introducing massive compiler database-catalog dependencies.

**Independent Test**: Verified via PL/SQL scalar/procedure test suites.

**Acceptance Scenarios**:
1. **Given** a PL/SQL declaration block, **When** declaring `v_users TABLE;`, **Then** the variable is successfully initialized with a default `Null` value of type `JSON` / `Table` and compiles without syntax errors.
2. **Given** a table variable `v_users TABLE;`, **When** assigning a JSON array returned from `query_rows()`, **Then** the assignment succeeds.

---

### User Story 3 - PL/SQL `FOR ... IN ... LOOP` Iteration (Priority: P1)

PL/SQL programmers need an easier, cleaner way to loop over tables or multi-row query results using a standard loop construct, rather than manually tracking counters, checking array bounds, and calling JSON extract helpers.

**Why this priority**: Dramatically simplifies procedural business logic, eliminating index-tracking boilerplate and making PL/SQL code highly readable.

**Independent Test**: Tested in `tests/plsql_functions.rs` using custom procedures.

**Acceptance Scenarios**:
1. **Given** a `TABLE` variable `v_rows` populated with rows, **When** executing a loop `FOR row_var IN v_rows LOOP ... END LOOP;`, **Then** the interpreter iterates over each row in the array sequentially, binding the current row to `row_var` as a JSON object, and executes the loop body.
2. **Given** a loop variable `row_var` inside the loop body, **When** accessing or updating properties via dot-notation (e.g., `row_var.name := 'Bob'`), **Then** field extraction and local assignment operate correctly and isolate changes to that iteration's variable.

---

### Edge Cases

- **Empty Tables**: When iterating over an empty table (e.g., a JSON array of `[]`), the `FOR` loop must execute zero times, successfully bypassing the body, and proceed to the statement following the loop.
- **Null Values**: If the table variable is `NULL`, the `FOR` loop must treat it as an empty collection, executing zero times, without panicking or failing.
- **Type Coercion during Dot Assignment**: Assigning a new value to a field (e.g., `row.age := 'thirty'`) must follow standard database type coercion and handle errors gracefully if the new value violates schema types during eventual DML inserts.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Rhai backend MUST support `oxibase::query(sql)` returning a Rhai `Array` of `Map` objects.
- **FR-002**: The Python backend MUST support `oxibase.query(sql)` returning a Python list of dictionaries.
- **FR-003**: The PL/SQL parser MUST recognize `TABLE` as a valid type name in variable declarations, treating it as an alias for `JSON` under the hood.
- **FR-004**: The PL/SQL parser and AST MUST support the `FOR variable IN table_expr LOOP ... END LOOP;` statement syntax.
- **FR-005**: The PL/SQL interpreter MUST implement the execution logic of the `FOR` loop by evaluating the `table_expr` as a JSON array, pushing a new loop-local frame to the environment, and iterating over each element.
- **FR-006**: The PL/SQL interpreter MUST support dot-notation variable access (reading) and variable updates (writing) for JSON-based row objects.
- **FR-007**: System MUST register built-in scalar database functions `QUERY_VALUE` (returns first column of first row) and `QUERY_ROWS` (returns all rows as a JSON array) available to all procedural engines.

### Key Entities

- **[PlSqlParser]**: Responsible for parsing the new variable type `TABLE` and the `FOR ... IN ... LOOP` loop syntax into AST nodes.
- **[PlSqlStatement::ForLoop]**: New AST statement node containing the loop variable name, the collection expression, and the statement body vector.
- **[PlSqlInterpreter]**: Evaluates statements, handles pushing/popping of local stack frames for loop variables, and manages the execution flow.
- **[RhaiBackend]** & **[PythonBackend]**: Expose the thread-local query API `query` returning structural native types.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of all compiled tests pass under `cargo nextest run`.
- **SC-002**: A dedicated integration test `test_plsql_table_for_loop` executes, proving that declaring `TABLE` and iterating via `FOR row IN table LOOP` correctly accesses and aggregates row attributes.
- **SC-003**: Memory footprint and execution speed for looping over 1,000 JSON rows inside PL/SQL remain within standard millisecond limits without leaking memory.
- **SC-004**: `make lint` returns zero warnings or formatting errors.

## Assumptions

- We assume a row representation as a dynamic JSON object is highly flexible, matches typical embedded/autonomous document store requirements, and avoids the overhead of formal catalog-bound tuple structs.
- We assume that declaring a loop variable in a `FOR` loop implicitly declares it for the scope of the loop body (standard PL/pgSQL convention).
