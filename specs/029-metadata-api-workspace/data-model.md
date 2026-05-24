# Data Model: Metadata API and Workspace

## Entities

### 1. TableMetadata
Represents the definition of a database table.
- `schema`: String (default "public")
- `name`: String
- `columns`: Array of `ColumnMetadata`

### 2. ColumnMetadata
Represents the definition of a column within a table.
- `name`: String
- `data_type`: String (e.g., "INT", "VARCHAR")
- `is_nullable`: Boolean
- `default_value`: Optional String

### 3. SqlQueryResult
Represents the result of executing an arbitrary SQL query.
- `columns`: Optional Array of Strings (present for SELECT queries)
- `rows`: Optional Array of Objects (key-value pairs mapping column names to values, present for SELECT queries)
- `rows_affected`: Optional Integer (present for DML/DDL queries like INSERT, UPDATE, DELETE, CREATE)

### 4. WorkspaceTemplate (Internal Storage)
Represents a stored HTML/JS/CSS template for the Workspace app inside the database.
- `id`: Integer (Primary Key)
- `route`: String (e.g., "/workspace", "/workspace/tables")
- `content`: Text (Minijinja template string containing HTML/DaisyUI/Unpoly)
- `content_type`: String (e.g., "text/html")

## State Transitions & Workflows

1. **Workspace Installation**:
   - The CLI command `install-workspace` connects to the database and executes `INSERT` statements to populate the `routes` and `templates` (WorkspaceTemplate) tables with the predefined HTML strings.

2. **API Data Operations**:
   - `POST /api/data/{table}` -> Parses request body -> Validates `table_exists` -> Generates `INSERT` SQL -> Executes -> Returns updated data or success status.
   - `DELETE /api/meta/tables/{schema}.{name}` -> Generates `DROP TABLE` SQL -> Executes -> Table metadata is removed from the system.