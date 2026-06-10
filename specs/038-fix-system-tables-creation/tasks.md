# Implementation Tasks: Fix System Tables Creation

**Branch**: `038-fix-system-tables-creation`
**Created**: 2026-06-08

## Phase 1: Setup

*No infrastructure or external dependencies required.*

## Phase 2: Foundational

*No foundational schema or trait changes required.*

## Phase 3: User Story 1 & 2 - Reliable Initial Boot and Migration (Priority: P1 & P2)

**Goal**: Ensure all system tables are explicitly created with their constraints (`PRIMARY KEY`, `UNIQUE`), and gracefully migrate old `_sys_*` data if it exists.
**Independent Test**: `SELECT schema_name, table_name FROM system.tables;` after booting the DB in a fresh instance.

- [ ] T001 [US1] Create `ensure_functions_table_exists` method in `src/executor/mod.rs`
- [ ] T002 [US1] Create `ensure_procedures_table_exists` method in `src/executor/mod.rs`
- [ ] T003 [US1] Create `ensure_triggers_table_exists` method in `src/executor/mod.rs`
- [ ] T004 [US1] Create `ensure_table_stats_table_exists` method in `src/executor/mod.rs`
- [ ] T005 [US1] Create `ensure_column_stats_table_exists` method in `src/executor/mod.rs`
- [ ] T006 [US2] Update `ensure_system_schema_and_migrations` in `src/executor/mod.rs` to call the new ensure methods.
- [ ] T007 [US2] Update migration logic in `ensure_system_schema_and_migrations` within `src/executor/mod.rs` to use `INSERT INTO system.X SELECT * FROM _sys_X` instead of `CREATE TABLE AS SELECT`.
- [ ] T008 [US1] Execute unit/integration tests to verify correct booting and migrations (run `make test`).

## Phase 4: Polish

- [ ] T009 Run `make lint` and `make license` to ensure codebase meets standards.

## Dependencies

- Phase 3 encompasses both User Story 1 and 2, as the creation and migration logic reside within the same boot sequence.

## Parallel Execution Examples

- T001 through T005 can be implemented in parallel since they create independent initialization methods.

## Implementation Strategy

1. Focus on `src/executor/mod.rs`.
2. Model the new `ensure_*` methods strictly after the existing `ensure_cron_tables_exist()` method.
3. Replace the `CREATE TABLE ... AS SELECT` statements in the migration sequence with `INSERT INTO ... SELECT` statements.
4. Verify with `make test`.