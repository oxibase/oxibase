# Tasks: System Information Schema

**Input**: Design documents from `specs/006-system-information-schema/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup & Foundational Constraints

**Purpose**: Establish the core rules and constraints for the virtual schema spaces before creating the virtual tables.

- [ ] T001 Define `system` and `information_schema` as reserved namespaces in the core logic (`src/core/schema.rs` or relevant validation file).
- [ ] T002 Implement logic in the DDL handler (`src/executor/ddl.rs`) to reject any `CREATE`, `ALTER`, or `DROP` commands targeting the `system` or `information_schema` namespaces.
- [ ] T003 Implement logic in the DML handler (`src/executor/dml.rs` or `src/executor/insert.rs`, etc.) to reject `INSERT`, `UPDATE`, or `DELETE` against tables in the `system` and `information_schema` namespaces.

---

## Phase 2: User Story 3 - Exposing Internal State (Priority: P3)

*Note: Proceeding with US3 first as it builds the foundational `system` virtual tables that `information_schema` will eventually rely on conceptually.*

**Goal**: Expose the core database metadata and active engine state as virtual tables within a `system` schema.

**Independent Test**: `tests/system_schema_tests.rs`

### Tests for User Story 3

- [ ] T010 [P] [US3] Create failing integration tests in `tests/system_schema_tests.rs` that verify `SELECT * FROM system.tables` and `system.columns` reflect created tables.

### Implementation for User Story 3

- [ ] T011 [P] [US3] Create a new executor module `src/executor/system_schema.rs` to handle dynamic generation of virtual `system` tables.
- [ ] T012 [US3] Implement dynamic row generation for `system.tables` within `src/executor/system_schema.rs`, pulling directly from `engine.schemas`.
- [ ] T013 [US3] Implement dynamic row generation for `system.columns` within `src/executor/system_schema.rs`, pulling directly from `engine.schemas`.
- [ ] T014 [US3] Update the main query executor (`src/executor/query.rs` or `src/executor/select.rs`) to intercept queries to the `system` schema and route them to `src/executor/system_schema.rs`.
- [ ] T015 [US3] Verify tests in `tests/system_schema_tests.rs` pass.

**Checkpoint**: At this point, users can query `system.tables` and `system.columns` to see raw internal metadata.

---

## Phase 3: User Story 1 - Querying Public Metadata (Priority: P1) 🎯 MVP

**Goal**: Implement or refine standard metadata views in `information_schema` (`tables` and `columns`) for compatibility with external tools.

**Independent Test**: `tests/information_schema_tests.rs`

### Tests for User Story 1

- [ ] T020 [P] [US1] Create failing integration tests in `tests/information_schema_tests.rs` to verify standard columns in `information_schema.tables` and `information_schema.columns`.

### Implementation for User Story 1

- [ ] T021 [US1] Update `src/executor/information_schema.rs` to ensure `information_schema.tables` correctly queries and maps data from the internal state (can utilize the same logic/state built in US3).
- [ ] T022 [US1] Update `src/executor/information_schema.rs` to add `character_octet_length` and `datetime_precision` columns to `information_schema.columns`.
- [ ] T023 [US1] Verify tests in `tests/information_schema_tests.rs` pass.

**Checkpoint**: At this point, ecosystem tools can seamlessly introspect the database using standard queries.

---

## Phase 4: User Story 2 - Querying Internal Debug Information (Priority: P2)

**Goal**: Expose internal debug state (e.g., transactions) in the `system` schema.

**Independent Test**: `tests/system_debug_tests.rs`

### Tests for User Story 2

- [ ] T030 [P] [US2] Create failing integration tests in `tests/system_debug_tests.rs` that start a transaction and verify it appears in `SELECT * FROM system.transactions`.

### Implementation for User Story 2

- [ ] T031 [US2] Implement dynamic row generation for `system.transactions` in `src/executor/system_schema.rs`, mapping from the `TransactionRegistry` or active transaction state in the `MVCCEngine`.
- [ ] T032 [US2] Ensure the query executor routes `system.transactions` queries correctly.
- [ ] T033 [US2] Verify tests in `tests/system_debug_tests.rs` pass.

**Checkpoint**: Administrators can now monitor active transactions via SQL.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final verification and code quality.

- [ ] T040 Verify `make lint` passes (runs formatting and clippy).
- [ ] T041 Verify `make license` passes (ensures all new `.rs` files have Apache-2.0 headers).
- [ ] T042 Ensure no `unwrap()` or `expect()` are used in the new modules; propagate all errors via `Result`.
- [ ] T043 Run `make test-all` to ensure no regressions in existing tests.