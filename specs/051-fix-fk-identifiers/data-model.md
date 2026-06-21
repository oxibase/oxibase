# Phase 1 Data Model: Schema-Qualified Foreign Keys

## Entities

### 1. `TableName` (AST Enum)
Exposing table name specifications from the SQL parser.
* **Fields**:
  * `Simple(Identifier)`: A simple table name (e.g. `orders`).
  * `Qualified(QualifiedIdentifier)`: A qualified table name with schema (e.g. `sales.orders`).

### 2. `TableConstraint::ForeignKey` (AST Variant)
Represents a table-level foreign key constraint.
* **Fields**:
  * `name`: `Option<String>` (Constraint name)
  * `column`: `Identifier` (Referencing column)
  * `foreign_table`: `TableName` (The referenced table - changed from `Identifier` to `TableName`)
  * `foreign_column`: `Identifier` (Referenced column)
  * `on_delete`: `ReferentialAction`
  * `on_update`: `ReferentialAction`

### 3. `ColumnConstraint::References` (AST Variant)
Represents a column-level foreign key constraint.
* **Fields**:
  * `0`: `TableName` (Referenced table - changed from `Identifier` to `TableName`)
  * `1`: `Option<Identifier>` (Optional referenced column name)

### 4. `ForeignKeyMetadata` (Schema Core Entity)
Persisted in schema metadata bytes.
* **Fields**:
  * `column_id`: `usize` (Local column index)
  * `referenced_table`: `String` (Fully qualified referenced table name, e.g., `"sales.customers"`)
  * `referenced_column_name`: `String` (Referenced column name)
  * `on_delete`: `ReferentialAction`
  * `on_update`: `ReferentialAction`

---

## State Transitions & Lifecycle

```text
[CREATE TABLE / ALTER TABLE SQL]
               │
               ▼ (parse_table_name & parse_program)
       [AST: TableName]
               │
               ▼ (ddl.rs: resolve referencing_schema and foreign_full_name)
    [Schema Validation Checks]
               │
               ▼ (Persist to Catalog)
    [ForeignKeyMetadata] (persisted with full qualified strings)
```
