# Research: AUTOINCREMENT and Constraints via ALTER TABLE

## Approach for AUTOINCREMENT

**Decision**: Modify `AlterTableOperation::ModifyColumn` in `src/executor/ddl.rs` to process `AUTOINCREMENT` constraint.
**Rationale**: The parser already handles `AUTOINCREMENT` as a `ColumnConstraint`. We just need to check if it's present in `col_def.constraints` during `ModifyColumn` and pass it down to `SchemaColumn`.
Also, we need to enforce that `AUTOINCREMENT` can only be applied to integer types.

## Approach for UNIQUE and CHECK

**Decision**: 
1. For `UNIQUE`, if present in `col_def.constraints`, we need to create a unique index for the modified column, similar to what's done in `CREATE TABLE`.
2. For `CHECK`, we need to extract the check expression from `ColumnConstraint::Check(expr)` and save it in `SchemaColumn::check_expr`.
**Rationale**: `UNIQUE` is enforced via indexes in Oxibase. Updating the column schema is not enough; an index must be created. `CHECK` is evaluated during insert/update and is stored as an expression string.

## Engine updates

**Decision**: Update `modify_column` in `Schema`, `Table` trait, `MvccTable` to accept `auto_increment` and `check_expr` parameters, or pass a struct with changes. Currently `modify_column` in `Schema` takes:
`name: &str, data_type: Option<DataType>, nullable: Option<bool>`
We need to change it to accept `auto_increment: Option<bool>` and `check_expr: Option<Option<String>>`.
**Rationale**: To support modifying constraints, the underlying storage engine and schema manager need to understand these new modifiable properties.
