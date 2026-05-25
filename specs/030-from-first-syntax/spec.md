# Feature Specification: FROM-First Syntax

**Feature Branch**: `030-from-first-syntax`  
**Created**: 2026-05-25  
**Status**: Draft  
**Input**: User description: "i would like to add a similar support to duckdb to the from clause: FROM-First Syntax DuckDB's SQL supports the FROM-first syntax..."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - FROM-First Syntax with a SELECT Clause (Priority: P1)

Users should be able to write SQL queries where the `FROM` clause appears before the `SELECT` clause. This improves readability for queries with complex projections or when chaining operations, aligning with how data flows (source to projection).

**Why this priority**: This is the core functionality requested. It introduces the new syntactic structure while keeping all existing query components (projection, source) explicit.

**Independent Test**: Can be fully tested by a new integration test in the parser and executor suites verifying that `FROM table_name SELECT col1, col2` parses correctly and produces the same result set as `SELECT col1, col2 FROM table_name`.

**Acceptance Scenarios**:

1. **Given** a table `tbl` with columns `i` and `s`, **When** the query `FROM tbl SELECT i, s;` is executed, **Then** the result set should contain columns `i` and `s` from `tbl`.
2. **Given** a table `tbl` with columns `i` and `s`, **When** the query `FROM tbl SELECT *;` is executed, **Then** the result set should contain all columns from `tbl`.
3. **Given** a complex query with subqueries like `FROM (VALUES ('a'), ('b')) t1(s), range(1, 3) t2(i) SELECT s, i;`, **When** the query is executed, **Then** the result set should match the equivalent standard `SELECT ... FROM ...` query.

---

### User Story 2 - FROM-First Syntax without a SELECT Clause (Priority: P2)

Users should be able to write SQL queries starting with a `FROM` clause and completely omitting the `SELECT` clause. This acts as a shorthand for `SELECT * FROM ...`, making it quicker to inspect the entire contents of a table or relation.

**Why this priority**: This is a direct consequence of supporting `FROM`-first syntax and provides a significant quality-of-life improvement for quick data exploration, matching the referenced DuckDB functionality.

**Independent Test**: Can be fully tested by a new integration test verifying that `FROM table_name;` parses correctly and produces the same result set as `SELECT * FROM table_name;`.

**Acceptance Scenarios**:

1. **Given** a table `tbl` with columns `i` and `s`, **When** the query `FROM tbl;` is executed, **Then** the result set should be identical to executing `SELECT * FROM tbl;`.
2. **Given** a table-valued function or complex join in the `FROM` clause like `FROM (VALUES ('a'), ('b')) t1(s), range(1, 3) t2(i);`, **When** the query is executed, **Then** the result set should be identical to executing `SELECT * FROM (VALUES ('a'), ('b')) t1(s), range(1, 3) t2(i);`.

---

### Edge Cases

- What happens when a `FROM` clause is followed by other clauses like `WHERE`, `GROUP BY`, `ORDER BY`, or `LIMIT` without a `SELECT` clause? (e.g., `FROM tbl WHERE i > 1;`)
- What happens when a `FROM` clause is followed by a `SELECT` clause, and then other clauses? Is the order strict (`FROM` -> `SELECT` -> `WHERE` vs `FROM` -> `WHERE` -> `SELECT`)?
- How does this interact with `UNION`, `INTERSECT`, and `EXCEPT`? (e.g., `FROM tbl1 UNION FROM tbl2;`)
- How does the parser distinguish between a `FROM` clause acting as the main query block versus a `FROM` clause inside a subquery or `EXISTS` expression?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The SQL parser MUST successfully parse queries starting with the `FROM` keyword.
- **FR-002**: The SQL parser MUST support an optional `SELECT` clause following a leading `FROM` clause.
- **FR-003**: The SQL parser MUST implicitly add a `SELECT *` projection to the logical plan if a query starts with `FROM` and omits the `SELECT` clause.
- **FR-004**: The optimizer and executor MUST treat `FROM tbl SELECT ...` and `SELECT ... FROM tbl` as semantically identical.
- **FR-005**: The parser MUST support standard query clauses (`WHERE`, `GROUP BY`, `ORDER BY`, `LIMIT`, etc.) in conjunction with the `FROM`-first syntax, allowing these clauses to appear in any order following the initial `FROM` clause (e.g., `FROM tbl SELECT y WHERE x` or `FROM tbl WHERE x SELECT y`).
- **FR-006**: The feature MUST be implemented without breaking existing standard SQL syntax (`SELECT ... FROM ...`).

### Key Entities

- **Parser/AST**: The Abstract Syntax Tree representation of a query must be updated to reflect that `SELECT` is optional and `FROM` can be the leading clause, or the parser rules must rewrite `FROM`-first queries into the standard AST representation during parsing.
- **Logical Plan**: The logical plan generation must ensure that `FROM`-first queries produce identical logical plans to their standard SQL counterparts.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Passes all new parsing and execution tests for `FROM`-first syntax with and without `SELECT` clauses.
- **SC-002**: Existing `make test` and `make test-all` suites pass completely without regressions (100% pass rate).
- **SC-003**: Overall test coverage does not drop below the minimum threshold (`make coverage-check` passes).
- **SC-004**: The code passes `make lint` with zero warnings and introduces no new `unwrap()` or `expect()` calls in library code.

## Assumptions

- The underlying execution engine does not need modification; the changes are entirely contained within the parser, AST construction, and potentially the initial logical plan building phases.
- The `FROM`-first syntax should support all standard relations in the `FROM` clause, including base tables, views, subqueries, table functions, and joins.
- The default behavior when `SELECT` is omitted is functionally equivalent to `SELECT *`.
