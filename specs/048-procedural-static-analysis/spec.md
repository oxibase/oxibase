# Feature Specification: AST-in-AST Static Analysis for Related Objects Detection

**Feature Branch**: `048-procedural-static-analysis`  
**Created**: June 19, 2026  
**Status**: Draft  
**Input**: User description: "AST-in-AST Static Analysis for Related Objects Detection"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Rhai Procedural Static Analysis (Priority: P1)

Users editing Rhai stored procedures and scripts need the system to statically analyze their scripts to extract database calls (`oxibase::execute`, `oxibase::query`, `oxibase::call`) and automatically populate a "Related Objects" view with the tables, procedures, or functions referenced in the embedded SQL.

**Why this priority**: High priority because Rhai is the default, always-enabled procedural language. Providing static analysis for Rhai is the foundation of the Semantic Layer/TUI experience.

**Independent Test**: Can be tested via unit/integration tests running `Database::analyze_script` on a Rhai procedure that contains various SQL strings and asserting the extracted `RelatedObject`s match expected tables/functions.

**Acceptance Scenarios**:

1. **Given** a Rhai script with calls to `oxibase::execute("SELECT * FROM schema.table")`, **When** requesting script analysis, **Then** the extracted related objects list contains table `schema.table`.
2. **Given** a Rhai script with call to `oxibase::call("proc_name()")`, **When** requesting script analysis, **Then** the extracted related objects list contains procedure `proc_name`.

---

### User Story 2 - Python Procedural Static Analysis (Priority: P2)

Users editing Python stored procedures and scripts need the system to statically parse their scripts, identify calls to `oxibase.execute()`, `oxibase.query()`, or `oxibase.call()`, extract the literal SQL strings, and identify referenced objects.

**Why this priority**: Medium priority because Python is an optional backend, but necessary to keep parity with multi-language support.

**Independent Test**: Can be tested conditionally using `#[cfg(feature = "python")]` inside tests, invoking `Database::analyze_script` with Python backend and verifying extracted dependencies.

**Acceptance Scenarios**:

1. **Given** a Python script calling `oxibase.execute("INSERT INTO users VALUES (1)")`, **When** requesting script analysis, **Then** the extracted related objects list contains table `users`.

---

### User Story 3 - PL/SQL and SQL Procedural Static Analysis (Priority: P1)

Users editing PL/SQL procedures or raw SQL scripts need the system to directly parse the entire SQL AST and extract all table, procedure, and function dependencies.

**Why this priority**: High priority because PL/SQL is standard for traditional database procedures and is natively tied to the SQL engine.

**Independent Test**: Tested by passing PL/SQL procedures directly to `Database::analyze_script` and verifying that referenced tables and called procedures are correctly listed.

**Acceptance Scenarios**:

1. **Given** a PL/SQL script with a query `SELECT * FROM test_table JOIN other_table ON test_table.id = other_table.id`, **When** analyzing the script, **Then** both `test_table` and `other_table` are listed as related table objects.

---

### Edge Cases

- **Dynamic Queries**: If a database call in Rhai or Python uses a variable or string concatenation (e.g., `oxibase::execute("SELECT * FROM " + table_name)`), static analysis cannot resolve the literal SQL. The analyzer should return a `Dynamic` flag or include a `RelatedObject` of type `"Dynamic"` (or with a custom name `"Dynamic"`) to indicate that dynamic/runtime queries exist that could not be statically resolved.
- **Unsupported Backends / Invalid Syntax**: If the backend is unsupported, or there is a compilation error in the script, the analyzer should handle the error gracefully without crashing.
- **Deduplication and Ordering**: Multiple references to the same table or procedure must be deduplicated and returned in a deterministic order (e.g., sorted).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001 (SQL AST Visitor)**: Implement a `Visitor` pattern in `src/parser/visitor.rs` that traverses `Statement` and `Expression` nodes to find table names (e.g., `SimpleTableSource`, inserts, updates, deletes, truncates) and procedure/function references.
- **FR-002 (Rhai AST Walker)**: Implement a Rhai AST traversal that parses Rhai code, walks all statements and expressions to find `Expr::FnCall`, checks if they reference `oxibase::query`, `oxibase::execute`, or `oxibase::call`, and extracts string literals from the first argument.
- **FR-003 (Python AST Walker)**: Implement a Python AST traversal (using the native `rustpython_compiler` or `rustpython_vm` parser) that parses Python code, walks nodes to find `oxibase` calls, and extracts string literals from the arguments.
- **FR-004 (PL/SQL Parser Integration)**: For PL/SQL or raw SQL scripts, direct parser integration must be supported by parsing the entire script into SQL Statements and visiting them directly.
- **FR-005 (Database API Endpoint)**: Expose a read-only method on `Database`:
  ```rust
  pub fn analyze_script(&self, script: &str, backend: &str) -> Result<Vec<RelatedObject>>;
  ```
- **FR-006 (Dynamic Query Detection)**: If a query/execute call argument is not a static string literal, the analyzer MUST detect this and add a `RelatedObject` with type `"Dynamic"` and name `"Dynamic"` to indicate presence of unresolved runtime queries.

### Key Entities

- **[RelatedObject]**: A public struct representing an extracted dependency:
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
  pub struct RelatedObject {
      pub object_type: String, // "Table", "Procedure", "Function", "Dynamic"
      pub name: String,
  }
  ```
- **[Visitor]**: A trait in `src/parser/visitor.rs` that provides hooks to visit `Statement` and `Expression` nodes.
- **[DependencyExtractor]**: A struct that implements the `Visitor` trait to collect referenced tables, procedures, and functions.
- **[ASTAnalyzer]**: Module `src/functions/analyzer.rs` that coordinates AST extraction across backends and passes SQL strings to the SQL parser and visitor.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Passes all new and existing integration test suites verified via `cargo nextest run`.
- **SC-002**: Accurately extracts referenced tables, procedures, and functions from nested queries inside scripts without executing them.
- **SC-003**: Fully passes `make lint` and formatting checks with zero warnings/errors.
- **SC-004**: No new use of `unwrap()` or `expect()` in library code.

## Assumptions

- We assume that standard SQL and PL/SQL scripts are syntactically valid SQL statements that can be processed by our native SQL parser.
- We assume that a non-literal argument (e.g. variable, concatenation) in a database call is easily detected at compile time as a non-string literal node in the respective AST.
