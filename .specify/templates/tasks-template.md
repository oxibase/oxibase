---
description: "Task list template for feature implementation"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- `src/api/` for database entrypoint logic
- `src/executor/` for query execution
- `src/storage/` for MVCC and engine state
- `tests/` for integration tests

<!-- 
  ============================================================================
  IMPORTANT: The tasks below are SAMPLE TASKS for illustration purposes only.
  
  The /speckit.tasks command MUST replace these with actual tasks based on:
  - User stories from spec.md (with their priorities P1, P2, P3...)
  - Feature requirements from plan.md
  
  DO NOT keep these sample tasks in the generated tasks.md file.
  ============================================================================
-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Initialize test files in `tests/` directory
- [ ] T002 Verify project compiles (`cargo build`)

---

## Phase 2: User Story 1 - [Title] (Priority: P1) 🎯 MVP

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [Integration test filename]

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T010 [P] [US1] Create failing test in `tests/[test_name].rs`

### Implementation for User Story 1

- [ ] T012 [P] [US1] Create [AST Node] model in `src/parser/[file].rs`
- [ ] T013 [P] [US1] Implement optimizer logic in `src/optimizer/[file].rs`
- [ ] T014 [US1] Implement executor in `src/executor/[file].rs` (depends on AST and optimizer)
- [ ] T015 [US1] Ensure ACID/MVCC constraints in `src/storage/[file].rs` if applicable
- [ ] T016 [US1] Run `make lint` and fix any warnings
- [ ] T017 [US1] Run `make test` to verify passing integration test

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

[Add more user story phases as needed, following the same pattern]

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] TXXX [P] Verify `make license` passes
- [ ] TXXX Verify `unwrap()` and `expect()` are not used inappropriately
- [ ] TXXX Code cleanup and refactoring
