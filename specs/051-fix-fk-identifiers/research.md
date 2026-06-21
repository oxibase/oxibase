# Phase 0 Research: Schema-Qualified Foreign Keys

## Unknowns & Key Choices

### Choice 1: AST representation for foreign table names
* **Decision**: Refactor `TableConstraint::ForeignKey` and `ColumnConstraint::References` to hold `TableName` instead of a raw `Identifier`.
* **Rationale**: Reusing the existing `TableName` type represents simple or qualified table identifiers perfectly without introducing redundant types (such as `ObjectName` as suggested in the initial issue). It aligns with other statements like `CREATE TABLE` and `SELECT` which already use `TableName` for table specification.
* **Alternatives considered**:
  * **Option A**: Defining a new `ObjectName` type. Rejected as YAGNI and over-engineering since `TableName` has identical semantics and fully implemented formatting and helper methods.
  * **Option B**: Representing as `String` or `Vec<String>`. Rejected because it loses precise token/position metadata essential for accurate syntax error reporting.

### Choice 2: Cross-Schema Resolution of Unqualified Foreign Keys
* **Decision**: Resolve unqualified referenced tables dynamically using the schema of the referencing table (obtained from `stmt.table_name.schema()`) with a fallback to the session's active schema (`ctx.current_schema()`).
* **Rationale**: In standard SQL databases, an unqualified reference to a target table in a foreign key constraint must resolve to the same schema as the table being created or modified. 
* **Alternatives considered**:
  * **Option A**: Fallback strictly to `"public"`. Rejected because creating a table in a custom schema (e.g., `sales.orders`) with an unqualified foreign key (e.g., `REFERENCES customers(id)`) would look for `public.customers` instead of `sales.customers`, violating SQL standards and expectations.
  * **Option B**: Require all foreign keys to be fully qualified. Rejected because it breaks standard SQL portability and backward compatibility with existing unqualified definitions.
