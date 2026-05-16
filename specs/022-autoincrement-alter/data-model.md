# Data Model Updates

No new entities are introduced. We will modify the existing `Schema` and `Table` methods to accept the new constraints.

### Modified Methods

1. **`Schema::modify_column`**
   ```rust
   pub fn modify_column(
       &mut self,
       name: &str,
       data_type: Option<DataType>,
       nullable: Option<bool>,
       auto_increment: Option<bool>,
       check_expr: Option<Option<String>>,
   ) -> Result<()>
   ```

2. **`Table::modify_column`**
   ```rust
   fn modify_column(
       &self,
       name: &str,
       column_type: Option<DataType>,
       nullable: Option<bool>,
       auto_increment: Option<bool>,
       check_expr: Option<Option<String>>,
   ) -> Result<()>
   ```

3. **`Engine::record_alter_table_modify_column`** (WAL)
   Needs to be updated to serialize the new constraints.

4. **`AlterTableStatement`** execution in `ddl.rs`
   Extract constraints from `ColumnDefinition` and apply them:
   - `AUTOINCREMENT` -> pass to `modify_column`
   - `UNIQUE` -> call `table.create_index_with_type(...)`
   - `CHECK` -> pass to `modify_column`
