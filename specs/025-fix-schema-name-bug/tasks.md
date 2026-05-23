# Tasks: Fix Schema Name Bug

## Phase 1: Setup

*No specific setup tasks needed as this is a bugfix on existing structures.*

## Phase 2: Foundational

**Goal**: Update the core `Schema` data structures to support namespaces and split table strings appropriately.

- [ ] T001 Update `Schema` struct in `src/core/schema.rs` to include `schema_name` and `schema_name_lower` string fields with "public" as the default.
- [ ] T002 Update `Schema::new` and `Schema::with_timestamps` in `src/core/schema.rs` to parse the `table_name` argument (splitting on `.`) and assign `schema_name` and `table_name` appropriately.
- [ ] T003 Update `SchemaBuilder` in `src/core/schema.rs` to parse the `table_name` argument (splitting on `.`) and assign `schema_name` appropriately before building the `Schema`.

## Phase 3: User Story 1 - Create Table in Specific Schema (Priority: P1)

**Goal**: Ensure tables created in non-public schemas are stored in the correct underlying map.
**Independent Test**: `CREATE TABLE system.cron_runs (id INT)` correctly routes to the `system` schema bucket.

- [ ] T004 [P] [US1] Update `MVCCEngine::create_table` in `src/storage/mvcc/engine.rs` to place the schema into the map bucket corresponding to `schema.schema_name_lower` instead of hardcoding `DEFAULT_SCHEMA`.
- [ ] T005 [P] [US1] Update `execute_create_table` in `src/executor/ddl.rs` to ensure the DDL layer handles the separated schema and table components correctly when interacting with `SchemaBuilder`.

## Phase 4: User Story 2 - Query Table from Specific Schema (Priority: P1)

**Goal**: Ensure the storage engine resolves fully-qualified table names during CRUD operations.
**Independent Test**: Standard queries against `system.cron_runs` correctly resolve to the `system` schema map.

- [ ] T006 [P] [US2] Update `MVCCEngine::table_exists` in `src/storage/mvcc/engine.rs` to parse the incoming table name string and lookup using the resolved schema map instead of hardcoding `DEFAULT_SCHEMA`.
- [ ] T007 [P] [US2] Update `MVCCEngine::get_table_schema` in `src/storage/mvcc/engine.rs` to parse the incoming table name string and lookup using the resolved schema map instead of hardcoding `DEFAULT_SCHEMA`.
- [ ] T008 [P] [US2] Update `MVCCEngine::update_table_schema` in `src/storage/mvcc/engine.rs` to parse the incoming table name string and lookup using the resolved schema map instead of hardcoding `DEFAULT_SCHEMA`.
- [ ] T009 [P] [US2] Update `MVCCEngine::drop_table_internal` in `src/storage/mvcc/engine.rs` to parse the incoming table name string and lookup using the resolved schema map instead of hardcoding `DEFAULT_SCHEMA`.
- [ ] T010 [P] [US2] Update `MVCCEngine`'s WAL deserialization operations (like `deserialize_schema`) in `src/storage/mvcc/engine.rs` to appropriately split or reconstruct schema names if needed.
- [ ] T011 [US2] Add unit or integration tests in the test suite to verify table creation and querying across different schemas (e.g. `system`).

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T012 Run the entire test suite (`make test-all`) to ensure backward compatibility and fix any failing tests that relied on the old behavior.
- [ ] T013 Run `make lint` to ensure code quality standards are met.

## Dependencies

- Phase 2 (Foundational) blocks Phase 3 and Phase 4.
- Phase 3 (US1) must be implemented to test Phase 4 (US2) effectively.

## Implementation Strategy

1. **Foundational Updates**: Focus strictly on `src/core/schema.rs` parsing logic first. This enables the rest of the codebase to understand namespaces.
2. **MVCC Engine Isolation**: Update `MVCCEngine`'s map resolution logic (Phase 3 and 4). Start with table creation, then move on to resolution methods (get, update, drop).
3. **Verification**: After implementation, use existing and new test cases to guarantee that standard `public` queries work flawlessly while enabling explicit schema-driven operations.