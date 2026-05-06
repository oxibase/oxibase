---
description: "Task list for App Scaffolding and Seeding CLI Commands implementation"
---

# Tasks: App Scaffolding and Seeding CLI Commands

**Input**: Design documents from `/specs/005-review-issue-39/`
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
- `src/bin/` for CLI entry points

---

## Phase 1: Setup & User Story 1 - Create a New App Scaffold (Priority: P1) 🎯 MVP

**Goal**: Implement the `create-app` CLI command to generate a standard directory structure with boilerplate files.

**Independent Test**: Create integration test for `create-app` command

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T001 [P] [US1] Create integration test in `tests/cli_create_app.rs` to verify running `oxibase create-app <name>` creates the expected directory structure and files, and fails if the directory already exists.

### Implementation for User Story 1

- [X] T002 [US1] Update `src/bin/oxibase.rs` to add `CreateApp { name: String }` variant to the `Commands` enum.
- [X] T003 [US1] Implement `handle_create_app` logic in `src/bin/oxibase.rs` (or a separated module):
  - Check if the `<name>` directory already exists and abort/return error if it does (FR-011).
  - Create the `<name>` directory and subdirectories: `data/`, `templates/`, `routes/`, `functions/`.
- [X] T004 [US1] Add boilerplate file generation logic to `handle_create_app`:
  - Write `data/001_init.sql` (dummy tables/inserts).
  - Write `templates/layout.html` and `templates/index.html`.
  - Write `routes/web.json` (mapping to index.html).
  - Write `functions/hello.rhai`.
- [X] T005 [US1] Print success message guiding the user to the `seed` command.
- [X] T006 [US1] Run `make lint` and `cargo nextest` to verify tests pass for US1.

**Checkpoint**: At this point, the `create-app` command should be fully functional and testable independently.

---

## Phase 2: User Story 2 - Seed Database from App Directory (Priority: P1)

**Goal**: Implement the `seed` CLI command to deterministically read the app directory and load it into the database within a transaction.

**Independent Test**: Create integration test for `seed` command

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T010 [P] [US2] Create integration test in `tests/cli_seed.rs` to verify running `oxibase seed <app_dir> -d file://...` loads SQL scripts in order, inserts templates and routes, and handles transactional rollbacks on errors.

### Implementation for User Story 2

- [X] T011 [US2] Update `src/bin/oxibase.rs` to add `Seed { app_dir: String, db: String }` variant to the `Commands` enum.
- [X] T012 [US2] Implement `handle_seed` transaction wrapper in `src/bin/oxibase.rs`:
  - Open DB connection and `db.begin()` transaction.
  - Ensure any errors inside the block cause the transaction to roll back instead of partially committing.
- [X] T013 [US2] Add system initialization and cleanup to `handle_seed`:
  - Execute `CREATE SCHEMA IF NOT EXISTS routes`, `templates`, `functions`.
  - Execute schema creation for system tables (`routes.definitions`, `templates.source`).
  - Clear existing states for routes, templates, and functions.
- [X] T014 [US2] Implement `data/` loading logic in `handle_seed`:
  - Read all `.sql` files in the `data/` directory.
  - Sort alphabetically.
  - Read content and run `tx.execute(sql)`.
- [X] T015 [US2] Implement `templates/` and `routes/` loading logic in `handle_seed`:
  - Recursively read `.html`/`.jinja` files in `templates/` and `tx.execute(INSERT INTO templates.source)`.
  - Read `.json` files in `routes/`, parse them using `serde_json`, and `tx.execute(INSERT INTO routes.definitions)`.
- [X] T016 [US2] Implement `functions/` loading logic:
  - Read script files in `functions/` and insert them into the appropriate system functions catalog/table.
- [X] T017 [US2] Commit transaction via `tx.commit()` on success.
- [X] T018 [US2] Run `make lint` and `cargo nextest` to verify tests pass for US2.

---

## Phase 3: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T020 [P] Verify `make license` passes (run `./scripts/fix_copyrights.sh` if needed).
- [X] T021 [P] Ensure no `unwrap()` or `expect()` calls were introduced in error handling (especially in `handle_seed`). Use proper `anyhow`/`thiserror` propagation.
- [X] T022 [P] Verify `make lint` and `make test-all` pass with all features enabled.