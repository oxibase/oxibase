# Feature Specification: generate_series Table-Valued Function

**Feature Branch**: `024-generate-series`  
**Created**: 2026-05-23  
**Status**: Draft  
**Input**: User description: "Please implement the `generate_series` table-valued function by porting it from the `stoolap` repository into `oxibase`..."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate Series with Start and Stop (Priority: P1)

Users want to query a sequence of numbers starting from a given value up to a stop value with a default step of 1, so they can quickly mock up data or create series of ranges in SQL queries.

**Why this priority**: It's the most basic and common usage of the `generate_series` function. Providing a minimal API initially enables many downstream use cases.

**Independent Test**: Can be fully tested by porting the `stoolap` integration tests for `generate_series(start, stop)` and verifying correct results are returned.

**Acceptance Scenarios**:

1. **Given** a connected database session, **When** the user executes `SELECT * FROM generate_series(1, 3)`, **Then** the system returns a table with values 1, 2, and 3.
2. **Given** a connected database session, **When** the user executes `SELECT * FROM generate_series(5, 5)`, **Then** the system returns a table with the value 5.

---

### User Story 2 - Generate Series with Start, Stop, and Step (Priority: P2)

Users want to specify a custom step value when generating a series, allowing them to increment by numbers other than 1 (e.g., negative increments or larger gaps) to model non-contiguous sequences.

**Why this priority**: It completes the full expected API of `generate_series`. It handles more complex data generation workloads, such as counting backwards.

**Independent Test**: Can be tested via integration tests using `generate_series(start, stop, step)`.

**Acceptance Scenarios**:

1. **Given** a connected database session, **When** the user executes `SELECT * FROM generate_series(1, 10, 2)`, **Then** the system returns a table with values 1, 3, 5, 7, and 9.
2. **Given** a connected database session, **When** the user executes `SELECT * FROM generate_series(10, 1, -2)`, **Then** the system returns a table with values 10, 8, 6, 4, and 2.

### Edge Cases

- What happens when the step value is `0`? (It should return an error)
- How does the system handle huge ranges where `stop - start` is millions or billions? (Should stream values iteratively to avoid memory exhaustion)
- How does the system handle negative step when start is less than stop? (Should return an empty set)
- How does the system handle NULL inputs, e.g., `generate_series(1, NULL)`?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Engine MUST parse statements containing the `generate_series` table-valued function in the `FROM` clause.
- **FR-002**: Engine MUST execute the function iteratively to emit values, rather than pre-allocating the entire output table in memory, ensuring memory efficiency.
- **FR-003**: System MUST handle error conditions gracefully (e.g. `step = 0`) and return standard execution errors instead of crashing.
- **FR-004**: System MUST support `generate_series(start, stop)` returning values from start to stop inclusive with step 1.
- **FR-005**: System MUST support `generate_series(start, stop, step)` returning values from start to stop inclusive with the defined step.

### Key Entities

- **Table-Valued Function Node**: Represents a function call used as a data source in a query.
- **Table Function Scan Logical Plan**: Logical representation of the table-generating source.
- **Table Function Executor**: Execution processor that acts as an iterator emitting generated values one by one based on `start`, `stop`, and `step`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully query `SELECT * FROM generate_series(...)` and obtain the correct number of rows.
- **SC-002**: System passes all ported integration tests covering `generate_series` scenarios.
- **SC-003**: Function execution does not exhaust memory on very large ranges (e.g. 1 to 100,000,000), verified by maintaining a stable memory footprint.
- **SC-004**: System handles error conditions gracefully without crashing or abruptly terminating.

## Assumptions

- We assume `generate_series` produces values of a standard integer type.
- We assume integration tests exist in `stoolap` and can be cleanly integrated into the existing oxibase test framework.