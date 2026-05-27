# Data Model: Rename Routes and Templates Interfaces

## Entity Updates

### `interface.routes` (formerly `routes.definitions`)
This system table stores the routes for the web server integration.

- `method` (TEXT)
- `path` (TEXT)
- `template_name` (TEXT)
- `context_query` (TEXT)

*No schema changes, only table name refactoring.*

### `interface.templates` (formerly `templates.source`)
This system table stores the Minijinja HTML templates.

- `name` (TEXT)
- `content` (TEXT)

*No schema changes, only table name refactoring.*
