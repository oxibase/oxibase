# Research Findings

## Unresolved Aspects from Technical Context

### Current Metadata Architecture
- **Decision:** Store metadata physically in `system.tables` and `system.columns` using the standard `MVCCTable` machinery, replacing the pure in-memory hash maps in `MVCCEngine` as the source of truth, but utilizing an in-memory cache for performance.
- **Rationale:** Currently, `oxibase` stores schema metadata in memory (`MVCCEngine::schemas` via nested HashMaps) and persists it directly via custom WAL operations (`WALOperationType::CreateTable`) and snapshots. By moving metadata to actual system tables (`system.tables`, `system.columns`), we unify the storage model. DDL operations (`CREATE TABLE`) will become standard `INSERT` transactions into these system tables, automatically gaining MVCC, locking, recovery, and persistence guarantees.
- **Alternatives considered:** Keeping the current WAL-based custom logging for schemas and only exposing virtual tables. Rejected because the feature spec demands physical storage of metadata as standard tables ("I want the core database metadata... to be physically stored as standard tables").

### Bootstrap Sequence
- **Decision:** Implement a layered bootstrap sequence in `MVCCEngine::open_engine()`. First, manually construct and register the raw `MVCCTable` instances for `system.tables` and `system.columns`. Then, run normal WAL/Snapshot recovery which will populate these tables with any persisted schema data. Finally, use the contents of `system.tables`/`columns` to instantiate and register all other user tables in memory.
- **Rationale:** The "chicken and egg" problem requires that the tables holding table definitions exist before table definitions can be read from disk. Currently, physical engine initialization just loads snapshots/WAL. By eagerly creating the `system` tables as the first step of `open_engine()`, the recovery process can subsequently load data into them just like any other table.
- **Alternatives considered:** Lazily bootstrapping system tables on first access (like `_sys_functions`). Rejected because core schema data is required immediately upon startup to validate user queries and manage transactions; it cannot be lazy. Bootstrapping at the server level (like `routes` schemas). Rejected because metadata must be available at the lowest storage tier, regardless of how the database is accessed (CLI, embedded, or HTTP).

### Information Schema Compatibility
- **Decision:** Keep the existing virtual table implementation in `src/executor/information_schema.rs`, but update its source to query the new `system.tables` and `system.columns` (or an in-memory cache synced with them) instead of iterating over the old `MVCCEngine::schemas` HashMap. Add missing compatibility columns: `character_octet_length` and `datetime_precision`.
- **Rationale:** `oxibase` already implements `information_schema.tables` and `information_schema.columns` as virtual views, providing 95% of what tools like DBeaver and JDBC need. The standard mandates these as views over system catalogs. Keeping them as virtual tables in the executor is correct and performant. Adding the two missing columns ensures robust compatibility with strict ORMs and drivers.
- **Alternatives considered:** Physically writing data to `information_schema` tables. Rejected because `information_schema` is defined by the SQL standard as a set of read-only views representing database state, not physical storage. The physical storage belongs in the `system` schema.
