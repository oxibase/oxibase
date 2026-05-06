# Data Model: Database-Driven Routes and Jinja Templates

This feature introduces two system tables to the database. These tables will be initialized on startup or created by the user to manage dynamic web pages.

## `templates.source`

Stores the raw Jinja template strings.

| Column | Type | Description |
|--------|------|-------------|
| `name` | `TEXT` | Primary key. Unique identifier for the template. |
| `content`| `TEXT` | The Jinja template source code. |

## `routes.definitions`

Maps HTTP requests to templates and provides dynamic context.

| Column | Type | Description |
|--------|------|-------------|
| `method` | `TEXT` | HTTP method (e.g., `'GET'`, `'POST'`). |
| `path`   | `TEXT` | The URL path to match (e.g., `'/dashboard'`). |
| `template_name` | `TEXT` | Foreign key to `templates.source.name`. |
| `context_query` | `TEXT` | SQL query to execute. Results are passed to the template. Can be `NULL`. |

## State Transitions

- **Route Resolution**: Incoming HTTP Request `->` Lookup `routes.definitions` by `method` and `path`.
- **Context Fetching**: If route found `->` Execute `context_query` using the existing `Database` engine `->` Serialize `Vec<Row>` to JSON value.
- **Template Rendering**: Load `content` from `templates.source` by `template_name` `->` The engine should have access to other templates in `templates.source` for `include` and `extends` `->` Render template with `minijinja` using the serialized context `->` Return HTML response.
