# Data Model: Metadata API and Workspace App

## Core Entities

The core entities here represent the structures exchanged over the new `/api/meta/*` endpoints, as well as the database schema additions.

### Metadata API Payloads

#### `TableDefinition` (POST /api/meta/tables)
- `name`: String (required)
- `schema`: String (optional, defaults to `public`)
- `columns`: Array of `ColumnDefinition` (required)

#### `ColumnDefinition` (POST /api/meta/columns)
- `table_id`: String (required, format: `schema.table`)
- `name`: String (required)
- `type`: String (required, e.g., `INTEGER`, `TEXT`)
- `is_nullable`: Boolean (optional, defaults to `true`)

### System Entities

#### `routes.definitions` (Table)
- `method`: TEXT
- `path`: TEXT
- `template_name`: TEXT
- `context_query`: TEXT

#### `templates.source` (Table)
- `name`: TEXT
- `content`: TEXT