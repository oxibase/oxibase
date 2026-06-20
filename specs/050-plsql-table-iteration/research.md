# Technical Research & Design Decisions: plsql-table-iteration

## Decisions & Alternatives

### 1. Representation of Rows and Tables
- **Decision**: Represent single rows as `Value::Json` objects (dictionaries mapping column names to serializable JSON values) and multi-row tables as `Value::Json` arrays (list of row dictionaries).
- **Rationale**: Reuses the database's highly robust, fully-implemented JSON serialization and deserialization architecture, avoiding the overhead, catalog dependencies, and parsing complexity of introducing custom table-tuple schema definitions inside procedural stack frames.
- **Alternatives considered**: Creating a dedicated `Value::Record` or `Value::Row` stack-only type. This would require substantial changes to the database core `Value` and type-handling systems, creating widespread ripples across execution operators, linter tools, and index engines.

### 2. Sugar-syntax Type `TABLE`
- **Decision**: Map variable declarations of type `TABLE` (e.g., `v_rows TABLE;`) directly to type `JSON` in `PlSqlParser::parse_variable_declaration`.
- **Rationale**: Delivers perfect Postgres-like syntactic clarity for non-technical or standard SQL developers while utilizing the existing, thoroughly tested JSON storage engine.

### 3. Parsing `FOR row IN table LOOP`
- **Decision**: Add `PlSqlStatement::ForLoop` AST node:
  ```rust
  pub struct ForLoopStatement {
      pub token: Token,
      pub loop_variable: String,
      pub collection_expr: Expression,
      pub body: Vec<PlSqlStatement>,
  }
  ```
- **Rationale**: Minimal additions to the parser and AST. Reuses the standard `Expression` parser for the collection expression (which can be a variable name, a function call like `query_rows()`, or a subquery).

### 4. Evaluation of Query Execution Inside Procedures
- **Decision**: Execute all procedural database queries through the thread-local transaction context via the static `execute_sql_query` runner bridge.
- **Rationale**: Guarantees that query statements executed from within procedural scripts or user-defined functions observe the exact transaction snapshot, MVCC isolation levels, and lock settings of the parent transaction, preventing data inconsistency.
