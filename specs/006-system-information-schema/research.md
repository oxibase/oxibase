# Research Findings

## Unresolved Aspects from Technical Context

### Current Metadata Architecture
- **Decision:** Keep the existing highly optimized in-memory HashMaps (`MVCCEngine::schemas`) and WAL logging as the actual source of truth for the database. Expose this state via syntactic sugar as virtual tables in a new `system` schema namespace (similar to how `information_schema` operates).
- **Rationale:** Rewriting the core engine to physically persist metadata as standard relational tables would introduce severe performance penalties (requiring table lookups for every query validation) and architectural complexity. Virtual tables provide the requested SQL debugging capabilities with zero performance overhead to the core engine.
- **Alternatives considered:** Physically storing schema metadata as internal tables. Rejected due to the massive performance impact and lack of clear benefit over the current optimized memory structure.

### Bootstrap Sequence
- **Decision:** No changes needed to the core `MVCCEngine::open_engine()` bootstrap sequence. The virtual `system` tables will simply become available as soon as the executor is ready to handle queries.
- **Rationale:** Because `system` tables are virtual and dynamically generated from memory, the "chicken and egg" problem of loading schema definitions from disk before the table abstraction exists completely disappears.
- **Alternatives considered:** Layered physical bootstrapping. Rejected as unnecessary now that we are using virtual tables.

### Information Schema Compatibility
- **Decision:** Keep the existing virtual table implementation in `src/executor/information_schema.rs`, but update its source to query the new `system.tables` and `system.columns` (or an in-memory cache synced with them) instead of iterating over the old `MVCCEngine::schemas` HashMap. Add missing compatibility columns: `character_octet_length` and `datetime_precision`.
- **Rationale:** `oxibase` already implements `information_schema.tables` and `information_schema.columns` as virtual views, providing 95% of what tools like DBeaver and JDBC need. The standard mandates these as views over system catalogs. Keeping them as virtual tables in the executor is correct and performant. Adding the two missing columns ensures robust compatibility with strict ORMs and drivers.
- **Alternatives considered:** Physically writing data to `information_schema` tables. Rejected because `information_schema` is defined by the SQL standard as a set of read-only views representing database state, not physical storage. The physical storage belongs in the `system` schema.