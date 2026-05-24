# Research: Metadata API and Workspace App

## Technical Decisions

### Decision 1: Metadata REST API Structure
- **Decision**: Implement `/api/meta/*` endpoints (e.g., `/api/meta/tables`, `/api/meta/columns`, `/api/meta/views`, `/api/meta/functions`, `/api/meta/schemas`, `/api/meta/triggers`, `/api/meta/indexes`, `/api/meta/constraints`) providing full CRUD functionality using JSON payloads.
- **Rationale**: This is a direct alignment with the `postgresmeta` approach. Providing standardized JSON REST endpoints abstract the underlying `CREATE/ALTER/DROP` DDL SQL statements, making it much easier for GUI applications like the Workspace App to consume.
- **Alternatives considered**: Expecting the frontend to generate and execute raw DDL strings via a `/api/sql` endpoint. This was rejected because it introduces complexity to the frontend and risks syntax errors, whereas structured endpoints are cleaner and type-safe.

### Decision 2: Schema-Qualified Name Parsing in Auto-API
- **Decision**: Update `table_exists` to parse `schema.table` dot notation. If no dot is found, default to `public`.
- **Rationale**: Currently, `table_name = 'information_schema.tables'` fails because the actual table name is just `'tables'`. Splitting it resolves the issue and unlocks access to system metadata views.
- **Alternatives considered**: Passing schema as a separate query parameter. Rejected because existing standard SQL notation (`schema.table`) is already supported in the path `GET /api/{schema}.{table}`.

### Decision 3: Unpoly for Frontend Interactions
- **Decision**: Use Unpoly as the frontend framework to handle partial page updates and modals, relying on Minijinja templates on the server for HTML rendering.
- **Rationale**: The user explicitly requested an extremely thin layer of JS that lets the endpoint do most of the heavy lifting. Unpoly fits this perfectly by intercepting links and form submissions and updating only the relevant parts of the DOM, avoiding conflicts with Minijinja's `{{ }}` delimiters since Unpoly uses HTML attributes (like `up-target`).
- **Alternatives considered**: Vue.js SPA was initially considered but rejected in favor of a lighter, server-driven approach.