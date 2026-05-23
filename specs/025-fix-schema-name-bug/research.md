# Research: Fix Schema Name Bug

## Decision 1: Trait signature updates vs internal parsing

- **Decision**: Update internal methods inside `MVCCEngine` rather than modifying the global `StorageEngine` trait and all implementations.
- **Rationale**: Changing `table_exists`, `get_table_schema`, and others in the `StorageEngine` trait to explicitly take a `schema_name` parameter alongside `table_name` would necessitate massive refactoring across tests, mock implementations, and alternative storage engines.
- **Alternative considered**: Refactoring `StorageEngine` trait. Rejected due to excessively high blast radius.

## Decision 2: Schema Parsing Strategy in MVCCEngine

- **Decision**: Use dynamic `split('.')` within `MVCCEngine`'s trait implementations on the provided `table_name` string.
- **Rationale**: When receiving `"system.cron_runs"`, it can be parsed efficiently into `("system", "cron_runs")`. If no `.` is present, fallback to `"public"` as the schema namespace. This cleanly routes operations to `self.schemas` map while transparently maintaining backward compatibility for all higher-level traits.
- **Alternative considered**: Forcing the AST layer to separate it in every struct. Rejected because trait methods take a single string, demanding string manipulation inside the engine anyway.

## Decision 3: Default Schema fallback

- **Decision**: Default to `"public"` if no schema is explicitly targeted or parsed.
- **Rationale**: Backwards compatibility standard in Postgres and Oxibase default behaviors.