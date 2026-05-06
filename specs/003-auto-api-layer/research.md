# Research: Auto-API Layer

## Resolving Unknowns

### Dependency: Axum vs Actix-Web
- **Decision:** Axum
- **Rationale:** The user explicitly requested Axum during the clarification phase. It integrates perfectly with Tokio, which is already a dependency (when the `web-server` or `server` feature is enabled). Axum's extractor pattern makes it easy to handle dynamic path parameters (`/:table`) and query parameters (`?limit=10&offset=0`).
- **Alternatives considered:** Actix-Web was considered but rejected per user preference.

### Architecture: Subcommand integration in Oxibase CLI
- **Decision:** Refactor `Args` to use `clap::Subcommand`.
- **Rationale:** The current CLI launches a REPL by default. To support `oxibase serve`, we need a subcommand structure (e.g., `oxibase repl` and `oxibase serve`). We will make the `db_path` a global argument or keep it specific to the command if appropriate, but standard `clap` subcommand patterns apply.
- **Alternatives considered:** Adding a simple flag (e.g., `--serve`) instead of a subcommand. Rejected because subcommands provide better organization for completely different modes of operation.

### Dynamic Schema Validation
- **Decision:** Query `information_schema.tables` inside the handler.
- **Rationale:** The feature requires validating the requested table exists. The `information_schema` is the canonical source of truth. We can execute a fast `SELECT 1 FROM information_schema.tables WHERE table_name = '...'` before processing the actual request.
- **Alternatives considered:** Caching the schema in memory. Rejected as it introduces cache invalidation complexity. Querying the internal system tables directly via the `Database` API is fast enough for the MVP.

### JSON Serialization
- **Decision:** Map `oxibase::Value` to `serde_json::Value`.
- **Rationale:** Axum natively supports returning JSON via the `axum::Json` wrapper. The CLI already has a `value_to_json` function we can reuse or extract to a shared location.
- **Alternatives considered:** Writing a custom HTTP response serializer. Rejected as unnecessary overhead given `serde_json` is standard.
