# Data Model: COPY AST Node

The only data model changes are to the abstract syntax tree (AST) representation of the `COPY` statement.

## Entity Updates

### `CopyStatement` (Modified)
Located in `src/parser/ast.rs`.

**Fields:**
- `token: Token`
- `table_name: TableName` (Changed from `Identifier`)
- `columns: Vec<Identifier>`
- `file_path: String`
- `format: CopyFormat`
- `header: bool`
- `delimiter: u8`
- `null_string: Option<String>`

**Relationships:**
- Uses `TableName` (which internally holds `name: Identifier` and `schema: Option<Identifier>`).