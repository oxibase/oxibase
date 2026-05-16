# Implementation Tasks: Public Schema Fallback

**Feature**: Public Schema Fallback
**Spec**: [spec.md](./spec.md)
**Plan**: [plan.md](./plan.md)

## Dependency Graph

```text
Phase 1: Setup & Foundational
  │
  ├──> Phase 2: User Story 1 (Create Object Schema Fallback)
  │      │
  │      └──> Phase 3: User Story 2 (Resolve Object References)
  │
  └──> Phase 4: Polish & Integration Tests
```

## Parallel Execution Examples

- `T001` and `T002` can be executed independently (updating different object types in `ddl.rs`).
- `T004` and `T005` can be executed in parallel (updating views vs sequences in `engine.rs`).

## Implementation Strategy

1. **Foundational Updates**: Refactor the inner storage of `MVCCEngine` for views and sequences so that they are nested by schema.
2. **Executor Updates (US1)**: Update DDL execution for functions, procedures, views, sequences, and triggers to fall back to `ctx.current_schema().unwrap_or("public")`.
3. **Reference Resolution (US2)**: Update how the engine resolves names for views and sequences (tables are largely already handled, but we'll ensure uniformity).
4. **Testing**: Write comprehensive integration tests ensuring objects without schemas land in `public`.

---

## Phase 1: Setup & Foundational

**Goal**: Prepare the `MVCCEngine` to compartmentalize views and sequences by schema.

- [ ] T001 Refactor `MVCCEngine` `views` property in `src/storage/mvcc/engine.rs` from `FxHashMap<String, Arc<ViewDefinition>>` to `FxHashMap<String, FxHashMap<String, Arc<ViewDefinition>>>`.
- [ ] T002 Refactor `MVCCEngine` `sequences` property in `src/storage/mvcc/engine.rs` from `FxHashMap<String, Arc<SequenceState>>` to `FxHashMap<String, FxHashMap<String, Arc<SequenceState>>>`.
- [ ] T003 Update `MVCCEngine::new` and `MVCCEngine::in_memory` in `src/storage/mvcc/engine.rs` to initialize the nested `FxHashMap` structures for `views` and `sequences`.

## Phase 2: User Story 1 - Create Object without explicitly setting a schema

**Goal**: Ensure `CREATE` statements automatically assign objects to the `public` schema.

- [ ] T004 [P] [US1] Update `execute_create_procedure` in `src/executor/ddl.rs` to extract schema via `stmt.procedure_name.schema().unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())`.
- [ ] T005 [P] [US1] Update `execute_create_function` in `src/executor/ddl.rs` to extract schema via `stmt.function_name.schema().unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())`.
- [ ] T006 [P] [US1] Update `execute_create_trigger` in `src/executor/ddl.rs` to extract schema via `stmt.trigger_name.schema().unwrap_or_else(|| ctx.current_schema().unwrap_or("public").to_string())`.
- [ ] T007 [P] [US1] Update `execute_create_view` in `src/executor/ddl.rs` to pass the correct schema down to the engine.
- [ ] T008 [P] [US1] Update `MVCCEngine::create_view` in `src/storage/mvcc/engine.rs` to insert into the correct schema bucket.
- [ ] T009 [P] [US1] Update `execute_create_sequence` in `src/executor/ddl.rs` to extract and handle the schema correctly.
- [ ] T010 [P] [US1] Update `MVCCEngine::create_sequence` in `src/storage/mvcc/engine.rs` to insert into the correct schema bucket.

## Phase 3: User Story 2 - Resolve Object References against the Public Schema

**Goal**: Ensure `DROP`, `ALTER`, and lookups default to the `public` schema.

- [ ] T011 [P] [US2] Update `MVCCEngine::drop_view` and `MVCCEngine::get_view` in `src/storage/mvcc/engine.rs` to look up views within `DEFAULT_SCHEMA` ("public") if no schema is provided.
- [ ] T012 [P] [US2] Update `MVCCEngine::view_exists_lowercase` and `MVCCEngine::list_views` in `src/storage/mvcc/engine.rs` to handle nested view schemas.
- [ ] T013 [P] [US2] Update `MVCCEngine::drop_sequence`, `MVCCEngine::alter_sequence`, `MVCCEngine::nextval`, and `MVCCEngine::setval` in `src/storage/mvcc/engine.rs` to look up sequences within `DEFAULT_SCHEMA` ("public").
- [ ] T014 [P] [US2] Update `MVCCEngine::sequence_exists` and `MVCCEngine::list_sequences` in `src/storage/mvcc/engine.rs` to handle nested sequence schemas.
- [ ] T015 [P] [US2] Update the `Engine` trait definitions in `src/storage/traits/engine.rs` for `create_sequence`, `alter_sequence`, `drop_sequence`, `nextval`, `setval`, and `list_sequences` if their signatures need to accept a schema (or clarify how the schema is inferred in the engine). Note: the trait already takes `sequence_name` so we will assume the name passed from the executor is either bare or qualified, and the engine resolves it. Wait, it's better if the engine resolves the default schema if the executor passes a bare name.

## Phase 4: Polish & Integration Tests

**Goal**: Verify all objects successfully map to `public` schema and clean up.

- [ ] T016 Write an integration test `tests/public_schema_fallback_tests.rs` to verify creating a table, view, sequence, procedure, function, and trigger without a schema suffix defaults to `public`.
- [ ] T017 Run `make lint` and `cargo nextest run` to ensure all tests pass and no compilation regressions exist.
