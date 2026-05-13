# Tasks: Scripting Backend Context Refactor (oxibase.ctx)

**Input**: Design documents from `specs/012-scripting-context/`
**Prerequisites**: plan.md, spec.md, contracts/, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- `src/functions/backends/` for all scripting backend modification

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Update tests and tutorials to map to the new target syntax

- [ ] T001 Update `docs/_docs/tutorials/triggers.md` tutorial with the new `oxibase.ctx.old` and `oxibase.ctx.new` syntax across all languages.
- [ ] T002 Update tutorial integration test `tests/tutorial_triggers_test.rs` to use the new `oxibase.ctx` syntax.

---

## Phase 2: User Story 1 - Python Context Refactor (Priority: P1) 🎯 MVP

**Goal**: Encapsulate `OLD` and `NEW` inside `oxibase.ctx.old` and `oxibase.ctx.new` for Python scripts.

**Independent Test**: `tests/triggers_test.rs` (Python triggers block)

### Implementation for User Story 1

- [ ] T003 [P] [US1] Update `src/functions/backends/python.rs` to inject a `ctx` Python dictionary containing `old` and `new` into the `oxibase` module, rather than putting `OLD` and `NEW` directly into the scope.
- [ ] T004 [P] [US1] Update `src/functions/backends/python.rs` `extract_new_row_dict` logic to read the updated row state from `vm.sys_modules["oxibase"].ctx.new`.
- [ ] T005 [P] [US1] Update all Python trigger tests in `tests/triggers_test.rs` to use the `oxibase.ctx` syntax.
- [ ] T006 [US1] Run `cargo nextest run --features python --test triggers_test` and fix any Python trigger execution failures.

**Checkpoint**: Python triggers can correctly run context properties nested inside the module.

---

## Phase 3: User Story 2 - JavaScript Context Refactor (Priority: P1)

**Goal**: Encapsulate `OLD` and `NEW` inside `oxibase.ctx.old` and `oxibase.ctx.new` for JS (Boa) scripts.

**Independent Test**: `tests/triggers_test.rs` (JS triggers block)

### Implementation for User Story 2

- [ ] T007 [P] [US2] Update `src/functions/backends/boa.rs` to create a `ctx` property inside the global `oxibase` object containing `old` and `new` JS objects. Remove the global `OLD`/`NEW` properties.
- [ ] T008 [P] [US2] Update `src/functions/backends/boa.rs` `extract_new_row_json` logic to fetch the updated state from `context.global_object().get("oxibase").get("ctx").get("new")`.
- [ ] T009 [P] [US2] Update all JavaScript trigger tests in `tests/triggers_test.rs` to use the `oxibase.ctx` syntax.
- [ ] T010 [US2] Run `cargo nextest run --features js --test triggers_test` and fix any JS trigger execution failures.

**Checkpoint**: JS triggers can correctly execute under the nested global properties.

---

## Phase 4: User Story 3 - Rhai Context Refactor (Priority: P2)

**Goal**: Encapsulate `OLD` and `NEW` inside `oxibase.ctx.old` and `oxibase.ctx.new` for Rhai scripts.

**Independent Test**: `tests/triggers_test.rs` (Rhai triggers block)

### Implementation for User Story 3

- [ ] T011 [P] [US3] Update `src/functions/backends/rhai.rs` to map `NewRowProxy` and `OldRowProxy` into an `oxibase` -> `ctx` Map structure within the evaluation scope.
- [ ] T012 [P] [US3] Update all Rhai trigger tests in `tests/triggers_test.rs` to use the `oxibase.ctx` syntax.
- [ ] T013 [US3] Run `cargo nextest run --test triggers_test` and fix any Rhai trigger execution failures.

**Checkpoint**: Rhai triggers correctly bind the row proxies into a dynamic object.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: System-wide regression verification

- [ ] T014 Run `cargo nextest run --features js,python` to verify all integrations operate without breaking existing systems.
- [ ] T015 Run `make lint` and fix any unused variables or imports caused by removing the global `OLD` / `NEW` injection logic.
