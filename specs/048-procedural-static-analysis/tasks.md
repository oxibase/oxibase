# Tasks: AST-in-AST Static Analysis for Related Objects Detection

**Input**: Design documents from `/specs/048-procedural-static-analysis/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Unit and integration test tasks are included to verify full coverage and robustness of dependency extraction.

**Organization**: Tasks are grouped by setup, foundation, user story, and polish phases to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- `src/parser/` for native SQL visitor and extractor
- `src/functions/` for scripting AST walkers and compiler/analyzers
- `src/api/` for public database structs and API methods
- `tests/` for integration tests

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic registration

- [X] T001 Register `visitor` submodule in `src/parser/mod.rs` and export its contents
- [X] T002 Register `analyzer` submodule in `src/functions/mod.rs`
- [X] T003 Define public `RelatedObject` struct in `src/api/database.rs` and re-export in `src/api/mod.rs` and `src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Goal**: Complete the native SQL AST visitor and dependency extractor.

**Independent Test**: Unit tested within `src/parser/visitor.rs`.

- [X] T004 Implement `Visitor` trait in `src/parser/visitor.rs` for `Statement` and `Expression` nodes
- [X] T005 Implement `walk_statement` and `walk_expression` helper functions in `src/parser/visitor.rs` to recursively traverse the AST
- [X] T006 Implement `DependencyExtractor` struct implementing `Visitor` to collect tables, procedures, and functions in `src/parser/visitor.rs`
- [X] T007 Add unit tests for `DependencyExtractor` in `src/parser/visitor.rs` verifying collection of tables/procedures from SELECT, INSERT, UPDATE, DELETE, TRUNCATE, and CALL statements

---

## Phase 3: User Story 1 - Rhai Procedural Static Analysis (Priority: P1) 🎯 MVP

**Goal**: Statically analyze Rhai scripts to find `oxibase` execution queries and extract their dependencies.

**Independent Test**: Tested in `src/functions/analyzer.rs` with Rhai-specific tests.

### Implementation for User Story 1
- [X] T008 [P] [US1] Implement Rhai AST walking functions `walk_rhai_stmt` and `walk_rhai_expr` in `src/functions/analyzer.rs` to extract literal SQL queries
- [X] T009 [US1] Integrate Rhai query compilation and analysis logic in `analyze_script` in `src/functions/analyzer.rs` using `rhai::Engine::compile`
- [X] T010 [US1] Support dynamic query detection for Rhai (FR-006) to append a `"Dynamic"` object marker in `src/functions/analyzer.rs`
- [X] T011 [US1] Add unit tests for Rhai script static analysis in `src/functions/analyzer.rs`

---

## Phase 4: User Story 2 - Python Procedural Static Analysis (Priority: P2)

**Goal**: Statically analyze Python scripts (via rustpython parser) to find `oxibase` queries and extract their dependencies.

**Independent Test**: Tested in `src/functions/analyzer.rs` under `#[cfg(feature = "python")]`.

### Implementation for User Story 2
- [X] T012 [P] [US2] Implement Python AST walking functions `walk_python_stmt` and `walk_python_expr` under `#[cfg(feature = "python")]` in `src/functions/analyzer.rs`
- [X] T013 [US2] Integrate Python script parsing and analysis logic in `analyze_script` in `src/functions/analyzer.rs` using `rustpython_vm::compiler::parser::parse`
- [X] T014 [US2] Support dynamic query detection for Python (FR-006) to append a `"Dynamic"` object marker in `src/functions/analyzer.rs`
- [X] T015 [US2] Add unit tests for Python script static analysis under `#[cfg(feature = "python")]` in `src/functions/analyzer.rs`

---

## Phase 5: User Story 3 - PL/SQL and SQL Procedural Static Analysis (Priority: P1)

**Goal**: Directly parse SQL and PL/SQL procedures and walk their entire AST using `DependencyExtractor`.

**Independent Test**: Tested in `src/functions/analyzer.rs` with PL/SQL/SQL tests.

### Implementation for User Story 3
- [X] T016 [P] [US3] Implement PL/SQL / raw SQL direct parsing and visitor execution in `analyze_script` in `src/functions/analyzer.rs` by directly parsing script via `parse_sql`
- [X] T017 [US3] Add unit tests for PL/SQL/SQL script static analysis in `src/functions/analyzer.rs`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Public API integration, end-to-end integration testing, and project compliance.

- [X] T018 Implement public `Database::analyze_script` method in `src/api/database.rs` routing to the analyzer orchestrator
- [X] T019 Add integration tests in `tests/` verifying `Database::analyze_script` against actual scripts across Rhai, Python, and PL/SQL
- [X] T020 Verify `make license` passes across all source files and run `./scripts/fix_copyrights.sh` if needed
- [X] T021 Run formatting and full project clippy suite `make lint`
- [X] T022 Run full test suite with all features enabled `make test-all`

---

## Dependencies & Completion Order

```text
       [Phase 1: Setup]
               │
               ▼
   [Phase 2: Foundational]
               │
               ▼
[Phase 3: Rhai Static Analysis (US1)]
         /         \
        ▼           ▼
[Phase 4 (US2)]   [Phase 5 (US3)]
        \           /
         ▼         ▼
    [Phase 6: Polish]
```

## Parallel Execution Examples

The user stories run completely in parallel since they reside in separate modules and functions:
- **Stream A (Rhai)**: `T008` -> `T009` -> `T010` -> `T011`
- **Stream B (Python)**: `T012` -> `T013` -> `T014` -> `T015`
- **Stream C (PL/SQL)**: `T016` -> `T017`

## Implementation Strategy

1. **MVP First**: Establish Rhai static analysis support first as the foundational proof-of-concept.
2. **Incremental Delivery**: Roll out Python support and PL/SQL parsing support once the Rhai MVP is verified.
3. **Cross-backend testing**: Validate thread-safety and feature flags as part of Phase 6 Polish.
