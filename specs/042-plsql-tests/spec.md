# Feature Specification: PL/SQL Extended Type Testing

## 1. Description and Goals

**What are we building?**
A comprehensive suite of unit tests for the newly implemented PL/SQL extended data types and operations. The tests will specifically target the validation of `FLOAT`, `TIMESTAMP`, and `JSON` types across different execution contexts inside PL/SQL scripts.

**Why are we building it?**
To ensure that the newly extended PL/SQL data types and logic (implemented in Issue #121) work correctly, maintain data integrity during type coercion, handle arithmetic operations correctly across integer and float combinations, and to guarantee high test coverage.

**Who is it for?**
Engineers and QA responsible for maintaining the robustness of the PL/SQL execution backend.

## 2. Scope and Boundaries

**In Scope:**
*   Unit test confirming `FLOAT`, `TIMESTAMP`, and `JSON` parameters can be processed within PL/SQL functions.
*   Unit test validating arithmetic operations (+, -, *, /) combining `INT` and `FLOAT` types (including implicit coercion).
*   Unit test covering logical comparison expressions (`<`, `>`, `<=`, `>=`, `=`, `!=`) across mixed types.

**Out of Scope:**
*   Adding support for new data types to the DB engine itself.
*   End-to-end integration tests over the network.
*   Refactoring existing, unrelated test files.

## 3. User Scenarios & Testing

**Scenario 1: Mixed Type Declarations and Passing**
- **Given** a PL/SQL function expecting `FLOAT`, `TIMESTAMP`, and `JSON` inputs
- **When** the developer invokes the function with valid data for each type
- **Then** the function should successfully process them without type errors, execute internal branch logic, and return the expected `FLOAT` result.

**Scenario 2: Float and Integer Arithmetic**
- **Given** a PL/SQL block utilizing `INT` and `FLOAT` variables
- **When** the developer performs addition, subtraction, multiplication, and division combining these types
- **Then** the interpreter must correctly cast variables, evaluate the expressions, and yield a precise floating-point result.

**Scenario 3: Logical Comparisons**
- **Given** a PL/SQL block containing conditional logic
- **When** the developer compares `INT` to `FLOAT` utilizing standard logical operators (e.g. `=`, `<`)
- **Then** the interpreter correctly routes execution based on the expected outcome of the numeric comparison.

## 4. Functional Requirements

1. **Test: Multi-Type Function Arguments**
   * The test must create a PL/SQL function receiving `FLOAT`, `TIMESTAMP`, and `JSON`.
   * The test must invoke the function using standard SQL types and literals.
   * The test must verify the final coerced state returned by the function.

2. **Test: Mixed-Type Arithmetic**
   * The test must validate `INT + INT`, `FLOAT + FLOAT`, `INT + FLOAT`, and `FLOAT + INT`.
   * The test must validate `-`, `*`, and `/` for mixed configurations.
   * The test must confirm precision is not lost when casting an `INT` to a `FLOAT` internally.

3. **Test: Logical Type Parity**
   * The test must cover all standard operators (`<`, `<=`, `>`, `>=`, `=`, `!=`).
   * The test must validate cross-type numeric comparison.

## 5. Non-Functional Requirements
*   **Performance:** Tests must execute within milliseconds as part of the standard `cargo nextest` pipeline.
*   **Reliability:** Tests must be deterministic and fully isolated (in-memory execution).

## 6. Success Criteria

*   100% of newly added test cases pass via `cargo nextest run`.
*   Code coverage mapping over `src/functions/plsql/interpreter.rs` stays strictly at or above the 70% CI baseline constraint.

## 7. Assumptions & Dependencies
*   Assumes the PL/SQL testing infrastructure is already configured and uses the in-memory execution strategy provided by `Database::open_in_memory()`.
