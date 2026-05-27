---
description: "Task list for Rename Routes and Templates Interfaces"
---

# Tasks: Rename Routes and Templates Interfaces

**Input**: Design documents from `/specs/032-rename-routes-templates/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Verify project compiles (`cargo build`)

---

## Phase 2: User Story 1 - Consistent Interface Naming (Priority: P1) 🎯 MVP

**Goal**: As a developer, I want the codebase to use `interface.routes` and `interface.templates` instead of `routes.definitions` and `templates.sources` so that the module and interface naming convention is consistent and easier to navigate.

**Independent Test**: Can be fully tested by running `make test` and `make lint` and ensuring that all references have been successfully updated without breaking functionality.

### Implementation for User Story 1

- [x] T002 [US1] Replace occurrences of `routes.definitions` with `interface.routes` in `src/bin/workspace/mod.rs`
- [x] T003 [P] [US1] Replace occurrences of `routes.definitions` with `interface.routes` in `src/server/handlers.rs`
- [x] T004 [P] [US1] Replace occurrences of `routes.definitions` with `interface.routes` in `src/server/mod.rs`
- [x] T005 [P] [US1] Replace occurrences of `routes.definitions` with `interface.routes` in `tests/server_test.rs`
- [x] T006 [US1] Replace occurrences of `templates.source` with `interface.templates` in `src/bin/workspace/mod.rs`
- [x] T007 [P] [US1] Replace occurrences of `templates.source` with `interface.templates` in `src/server/mod.rs`
- [x] T008 [P] [US1] Replace occurrences of `templates.source` with `interface.templates` in `src/server/template.rs`
- [x] T009 [P] [US1] Replace occurrences of `templates.source` with `interface.templates` in `tests/server_test.rs`
- [x] T010 [P] [US1] Replace occurrences of `templates.source` with `interface.templates` in `examples/demo_server.rs`
- [x] T011 [US1] Replace occurrences of `templates.source` with `interface.templates` in documentation files (e.g. `docs/_docs/how-to/dynamic-web-pages.md`)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently.

---

## Phase 3: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T012 Verify `make lint` passes
- [x] T013 Verify `make test` passes
- [x] T014 Verify `make license` passes
