# Data Model: Fix Schema Name Bug

## Entities

### `Schema` (Modified)
**Module**: `src/core/schema.rs`

#### New Fields:
- `schema_name: String`: The original casing name of the schema (e.g., "System"). Defaults to "public".
- `schema_name_lower: String`: The lowercase variant for map lookups (e.g., "system").

#### Modifications:
- `Schema::new()` and `SchemaBuilder::new()` might need updates or internal adjustments to handle parsing or separating the table namespace.
- A mechanism to determine schema scope. Since DDL often constructs `Schema` via `SchemaBuilder`, `SchemaBuilder` should optionally accept `schema_name` or parse it directly from `table_name` argument (if we adopt a string split approach at builder level).

### `MVCCEngine` Maps (Affected Logic)
**Module**: `src/storage/mvcc/engine.rs`

- `self.schemas` is already typed as `FxHashMap<String, FxHashMap<String, Schema>>` (`schema_name` -> `table_name` -> `Schema`).
- The logic inside `MVCCEngine` currently hardcodes the outer map lookup to `DEFAULT_SCHEMA` ("public") in places like `create_table`, which causes `system.cron_runs` to be injected into `schemas["public"]["system.cron_runs"]`.
- This logic will be fixed to route to `schemas["system"]["cron_runs"]`.