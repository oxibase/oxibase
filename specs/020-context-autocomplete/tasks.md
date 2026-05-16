---
description: "Task list for context-autocomplete feature implementation"
---

# Tasks: context-autocomplete

**Input**: Design documents from `/specs/020-context-autocomplete/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Define `SqlHelper` struct in `src/bin/oxibase.rs` that implements `rustyline::Helper`, `rustyline::hint::Hinter`, `rustyline::highlight::Highlighter`, and `rustyline::validate::Validator` (with default/empty implementations for now)
- [x] T002 Update `Cli::new` in `src/bin/oxibase.rs` to initialize `SqlHelper` and set it as the helper for `rustyline::Editor`

---

## Phase 2: User Story 1 - SQL Keyword Autocomplete (Priority: P1) 🎯 MVP

**Goal**: Users typing SQL queries in the CLI can hit Tab to complete standard SQL keywords (SELECT, INSERT, CREATE, etc.) or CLI commands (help, exit).

**Independent Test**: Unit tests for the completer parsing logic.

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T003 [P] [US1] Create unit tests for keyword autocomplete logic in `src/bin/oxibase.rs`

### Implementation for User Story 1

- [x] T004 [US1] Implement `rustyline::completion::Completer` for `SqlHelper` in `src/bin/oxibase.rs` to return a static list of SQL keywords and CLI commands based on the current word prefix
- [x] T005 [US1] Run `make lint` and fix any warnings
- [x] T006 [US1] Run `make test` to verify passing tests

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 3: User Story 2 - Context-Aware Schema Autocomplete (Priority: P2)

**Goal**: Users typing SQL queries in the CLI can hit Tab to complete database schema objects, such as table names after `FROM` or `UPDATE`, based on the actual current state of the database.

**Independent Test**: Unit tests with a dummy database to verify context-aware suggestions.

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [P] [US2] Create unit tests for context-aware schema autocomplete logic in `src/bin/oxibase.rs` (using a real or mock `Database` instance)

### Implementation for User Story 2

- [x] T008 [US2] Update `SqlHelper` in `src/bin/oxibase.rs` to hold a reference (`Arc` or clone) to the active `Database` instance
- [x] T009 [US2] Enhance `rustyline::completion::Completer` implementation in `src/bin/oxibase.rs` to detect table contexts (e.g., after `FROM`, `INTO`, `UPDATE`)
- [x] T010 [US2] Implement logic in `src/bin/oxibase.rs` to fetch table names from `information_schema.tables` using the `Database` instance when in a table context, filtering by the current prefix
- [x] T011 [US2] Run `make lint` and fix any warnings
- [x] T012 [US2] Run `make test` to verify passing tests

**Checkpoint**: At this point, User Story 2 should be fully functional and testable independently

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T013 Verify `make license` passes
- [x] T014 Verify `unwrap()` and `expect()` are not used inappropriately in the new CLI code
- [x] T015 Verify autocomplete performance is responsive (under 50ms)
