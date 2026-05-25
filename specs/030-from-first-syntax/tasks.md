# Tasks: FROM-First Syntax

**Input**: Design documents from `specs/030-from-first-syntax/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Project initialization and basic structure. No specific structural changes are required for this parser-only feature, but we establish the base parser files we will be editing.

- [x] T001 Verify project compiles (`cargo build`)

---

## Phase 2: Foundational 

**Purpose**: Blocking prerequisites that must be in place before the user stories can be fully realized.

- [x] T002 Implement `parse_from_first_statement` function scaffold in `src/parser/statements.rs`.
- [x] T003 Hook `parse_from_first_statement` into the main `parse_statement` dispatcher in `src/parser/statements.rs` when the `FROM` keyword is encountered.

---

## Phase 3: User Story 1 - FROM-First Syntax with a SELECT Clause (Priority: P1) 🎯 MVP

**Goal**: Users should be able to write SQL queries where the `FROM` clause appears before the `SELECT` clause. The parser should accept these and rewrite them into a standard `SelectStatement` AST node.

**Independent Test**: New tests in `src/parser/parser.rs` verifying `FROM table_name SELECT col1, col2` parses correctly into a `SelectStatement`.

### Tests for User Story 1 ⚠️

- [x] T010 [P] [US1] Add parser tests in `src/parser/parser.rs` to verify `FROM tbl SELECT a, b` parses equivalently to `SELECT a, b FROM tbl`.
- [x] T011 [P] [US1] Add parser tests in `src/parser/parser.rs` for subqueries and complex FROM clauses with SELECT, e.g. `FROM (VALUES(1)) t(a) SELECT a`.
- [x] T012 [P] [US1] Add parser tests in `src/parser/parser.rs` to verify that other clauses (`WHERE`, `ORDER BY`, `LIMIT`) can be parsed in "any order" alongside `SELECT` after `FROM`.
- [x] T013 [US1] Implement the core logic of `parse_from_first_statement` in `src/parser/statements.rs`. It must loop to parse the initial table expression, then dynamically dispatch to parse `SELECT`, `WHERE`, `ORDER BY`, etc., based on keywords.
- [x] T014 [US1] Ensure `parse_from_first_statement` properly constructs a `SelectStatement` struct and assigns the parsed clauses to their respective fields.
- [x] T015 [US1] Run `make test` to verify the new parser tests pass.

**Checkpoint**: At this point, User Story 1 should be fully functional, and explicit `FROM ... SELECT ...` queries should parse correctly.

---

## Phase 4: User Story 2 - FROM-First Syntax without a SELECT Clause (Priority: P2)

**Goal**: Users should be able to write SQL queries starting with a `FROM` clause and completely omitting the `SELECT` clause, implicitly resulting in a `SELECT *` projection.

**Independent Test**: New tests in `src/parser/parser.rs` verifying `FROM table_name;` parses correctly and implicitly adds `SELECT *`.

### Tests for User Story 2 ⚠️

- [x] T020 [P] [US2] Add parser tests in `src/parser/parser.rs` to verify `FROM tbl` without a `SELECT` parses equivalently to `SELECT * FROM tbl`.
- [x] T021 [P] [US2] Add parser tests in `src/parser/parser.rs` to verify `FROM tbl WHERE x > 1` parses equivalently to `SELECT * FROM tbl WHERE x > 1`.

### Implementation for User Story 2

- [x] T022 [US2] Modify `parse_from_first_statement` in `src/parser/statements.rs` so that if the clause parsing loop finishes without having parsed a `SELECT` clause (i.e. `columns` is empty), it defaults the `columns` field to `vec![Expression::Star(...)]`.
- [x] T023 [US2] Verify execution integration test. Add an integration test in `tests/` to actually execute a query like `FROM memory_table` to ensure end-to-end execution works seamlessly because of the AST rewrite.
- [x] T024 [US2] Run `make test` to verify the implicit `SELECT *` tests pass.

**Checkpoint**: At this point, User Story 2 should be fully functional, and queries omitting `SELECT` should work perfectly.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and code quality verification.

- [x] T030 [P] Ensure no `unwrap()` or `expect()` were added in `src/parser/statements.rs`.
- [x] T031 Verify `make lint` passes with no warnings.
- [x] T032 Verify `make license` passes to ensure headers are intact.
- [x] T033 Run `make test-all` to ensure no existing tests or other SQL dialects were broken by the new parser loop.
- [x] T034 Run `make coverage-check` to verify test coverage hasn't dropped.
