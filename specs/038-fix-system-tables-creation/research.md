# Research: Fix System Tables Creation

## 1. Table Initialization Pattern
- **Decision**: Adopt the explicit schema creation pattern used by `ensure_cron_tables_exist()` for all internal system tables.
- **Rationale**: The previous `CREATE TABLE AS SELECT` approach drops all `PRIMARY KEY`, `UNIQUE`, and `AUTO_INCREMENT` constraints, and fails silently on an empty database (since the source table `_sys_*` doesn't exist).
- **Alternatives considered**: None, as explicit schema creation is the only way to enforce strict data constraints upon database creation.

## 2. Data Migration Strategy
- **Decision**: Use `INSERT INTO system.X SELECT * FROM _sys_X` after explicitly creating the schema.
- **Rationale**: Ensures old table data is gracefully ported to the strictly defined tables without violating schemas or losing existing data on version upgrades.
- **Alternatives considered**: Dropping old metadata entirely (rejected: causes unacceptable data loss).