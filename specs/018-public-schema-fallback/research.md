# Research: Public Schema Fallback

## 1. Schema Fallback for Procedures, Functions, and Triggers

- **Decision**: Update `execute_create_procedure`, `execute_create_function`, and `execute_create_trigger` to explicitly resolve the schema name using `ctx.current_schema().unwrap_or("public").to_string()` if `stmt.name.schema()` returns `None`.
- **Rationale**: Currently, `schema: stmt.procedure_name.schema().map(|s| s.to_uppercase())` results in `None` (which serializes as `NULL` in the system table) if no schema is provided. Changing this to an explicit fallback ensures that `public` is populated consistently.
- **Alternatives considered**: Leaving it as `NULL` and trying to map `NULL` to `public` at runtime. Rejected because having explicitly defined schemas simplifies lookups and reflects standard database behavior.

## 2. Refactoring Views and Sequences in `MVCCEngine`

- **Decision**: Change the signature of `MVCCEngine.views` and `MVCCEngine.sequences` from `Arc<RwLock<FxHashMap<String, Arc<T>>>>` to `Arc<RwLock<FxHashMap<String, FxHashMap<String, Arc<T>>>>>`.
- **Rationale**: Currently, views and sequences are treated as globally scoped variables without any schema compartmentalization. By adopting a nested `FxHashMap`, they conform to the same standard as tables (`self.schemas`).
- **Alternatives considered**: Using composite string keys (e.g., `"public.my_view"`). Rejected because it requires pervasive string allocation and doesn't match the existing nested hash map approach used for `self.schemas`.

## 3. Storage Trait Updates

- **Decision**: We need to update `create_view`, `drop_view`, `view_exists`, `get_view`, `create_sequence`, `drop_sequence`, `alter_sequence`, `nextval`, `setval`, and `list_sequences` to either accept a schema parameter or rely on the DDL executor to always pass fully qualified names (though for separation of concerns, the engine should probably still accept a schema, or the executor should resolve it). Given how `create_table` works (the `Schema` struct has the `table_name_lower` but the engine resolves it into `DEFAULT_SCHEMA`), we will adopt a similar strategy: `views` and `sequences` operations will fall back to `DEFAULT_SCHEMA` ("public") unless a schema is provided.
- **Rationale**: This aligns the behavior of views and sequences with tables.
