# Data Model: AST-in-AST Static Analysis for Related Objects Detection

This document details the components, entities, and data models involved in static analysis and dependency extraction.

## Components & Entities

### 1. `RelatedObject` (API Layer Entity)
Represents a database object detected inside a script.
- **Fields**:
  - `object_type`: String (e.g. `"Table"`, `"Procedure"`, `"Function"`, `"Dynamic"`)
  - `name`: String (the extracted object name, e.g. `"users"`, `"pizza_demo.customer_order"`)
- **Properties**:
  - Implements `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`.

### 2. `Visitor` and `DependencyExtractor` (SQL AST Layer)
Trait and struct for traversing the native SQL AST.
- **`Visitor` Trait**:
  - `visit_statement(&mut self, stmt: &Statement)`
  - `visit_expression(&mut self, expr: &Expression)`
- **`DependencyExtractor` Struct**:
  - `tables: HashSet<String>`
  - `procedures: HashSet<String>`
  - `functions: HashSet<String>`
  - `is_dynamic: bool` (flag for dynamic/non-statically resolvable SQL statements)

### 3. `ASTAnalyzer` (Scripting Backends Layer)
Orchestrates backend-specific AST walking.
- **Rhai AST walker**:
  - Identifies calls where name is `"oxibase::execute"`, `"oxibase::query"`, or `"oxibase::call"`.
  - Extracts the first argument if it is a literal string. If it is non-literal, marks as `"Dynamic"`.
- **Python AST walker**:
  - Identifies calls targeting `oxibase.execute`, `oxibase.query`, or `oxibase.call`.
  - Extracts first argument if constant string. If it is non-literal, marks as `"Dynamic"`.
- **PL/SQL parser**:
  - Directly feeds PL/SQL procedure scripts into the native SQL parser and visits nodes using `DependencyExtractor`.
