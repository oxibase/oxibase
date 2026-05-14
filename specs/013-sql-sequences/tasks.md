# Implementation Tasks: SQL Sequences

**Feature**: SQL Sequences

## Phase 1: Setup & Data Model
**Goal**: Define the sequence catalog data structures and AST parser nodes.
**Independent Test Criteria**: AST parses `CREATE SEQUENCE`, `ALTER SEQUENCE`, and `DROP SEQUENCE` statements into internal rust structs.

- [x] T001 [P] Define `SequenceOptions`, `SequenceState`, and updates to `SessionContext` models in `src/catalog/sequence.rs` (create this file if necessary, or put in `src/catalog/schema.rs`)
- [x] T002 [P] Define `CreateSequence`, `AlterSequence`, `DropSequence` AST nodes in `src/parser/ast.rs`
- [x] T003 Implement `CREATE SEQUENCE`
- [x] T004 Implement `ALTER SEQUENCE`
- [x] T005 Write AST parsing unit tests

## Phase 2: Core Execution & Scalar Functions
**Goal**: Implement the execution handlers for sequence creation and the built-in functions to increment/read them.
**Independent Test Criteria**: Functions `nextval()`, `currval()`, and `setval()` correctly mutate sequence state and session state in isolation.

- [ ] T006 [P] [US1] Implement DDL execution logic for `CREATE SEQUENCE` in `src/executor/ddl.rs`
- [ ] T007 [P] [US1] Implement DDL execution logic for `ALTER SEQUENCE` and `DROP SEQUENCE` in `src/executor/ddl.rs`
- [x] T008 [US2] Implement atomic `nextval()` natively via VM opcodes
- [x] T009 [US2] Implement `currval()` using session context lookup via VM opcodes
- [x] T010 [US3] Implement `setval()` updating catalog state via VM opcodes
- [x] T011 Test sequence functions

## Phase 3: Information Schema Integration
**Goal**: Expose active sequence metadata via the information schema catalog.
**Independent Test Criteria**: `SELECT * FROM information_schema.sequences` returns the expected sequences.

- [x] T012 [US4] Update information schema generator
- [x] T013 Write integration tests

## Dependencies

- Phase 2 depends on Phase 1 (AST and Data Model must exist before execution).
- Phase 3 depends on Phase 2 (Creation execution must work for info schema to be populated).

## Parallel Execution Examples

- **Example 1**: T001 (Data Model) and T002 (AST Nodes) can be executed concurrently as they touch distinct internal domains.
- **Example 2**: T006 (Creation Execution) and T007 (Alter/Drop Execution) can be implemented in parallel once AST nodes (Phase 1) exist.

## Implementation Strategy
Start with the AST and internal data structures. Get `CREATE SEQUENCE` running through the pipeline first, mapping it into an in-memory `DashMap` in the catalog. Then, build `NEXTVAL` utilizing `AtomicI64` and verify its concurrency using threading tests. Finally, wire the `information_schema`.
