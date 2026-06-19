# Data Model: Procedural Random Support

This document details the components, entities, and data models involved in the procedural random support feature.

## Components & Entities

### 1. `RhaiBackend` (Rhai Scripting Module)
Exposes the native functions to Rhai script execution blocks.
- **Functions registered**: `oxibase::random() -> f64`
- **Output type**: Float (`f64`)
- **Range constraint**: `[0.0, 1.0)`

### 2. `PythonBackend` (Python Scripting Module)
Exposes the native functions to Python script execution blocks.
- **Functions registered**: `oxibase.random() -> PyObjectRef`
- **Output type**: Float (`f64`)
- **Range constraint**: `[0.0, 1.0)`

### 3. `PlSqlInterpreter` (PL/SQL Statement Interpreter)
Interprets procedural PL/SQL code blocks. We extend its expression evaluator (`eval_expr`) to support general function calls.
- **AST Node supported**: `Expression::FunctionCall`
- **Lookup mechanism**: `FunctionRegistry::get_scalar`
- **Validation**:
  - Arguments are evaluated sequentially prior to function invocation.
  - The resolved function must exist in the database registry.
  - Return type and value are coerced or returned natively.
