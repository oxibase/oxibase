# Tasks: SQL Command Documentation & Railroad Diagrams

**Input**: Design documents from `/specs/031-sql-railroad-diagrams/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Tests for this feature involve verifying the documentation builds and the visual elements render correctly.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Analyze existing documentation structure and prepare for migration.

- [x] T001 Find the current markdown files for SQL commands in the `docs/` repository
- [x] T002 Identify the static site generator used in `docs/` (e.g., Jekyll, MkDocs) by looking for configuration files like `mkdocs.yml` or `_config.yml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the base directory structure for the new documentation organization.

- [x] T003 Create directories `docs/_docs/references/sql-commands/ddl`
- [x] T004 Create directories `docs/_docs/references/sql-commands/dml`
- [x] T005 Create directories `docs/_docs/references/sql-commands/dql`
- [x] T006 Create directories `docs/_docs/references/sql-commands/dcl`
- [x] T007 Create directories `docs/_docs/references/sql-commands/tcl`
- [x] T008 Create directory `docs/_docs/references/sql-commands/pragma`

---

## Phase 3: User Story 1 - SQL Documentation Restructuring (Priority: P1) 🎯 MVP

**Goal**: Organize the SQL commands reference documentation into categories (DDL, DML, Transactions, etc.) with dedicated pages per command.

**Independent Test**: Build the documentation site locally and verify the navigation structure.

### Implementation for User Story 1

- [x] T009 [US1] Move/create `CREATE TABLE` documentation to `docs/_docs/references/sql-commands/ddl/create_table.md`
- [x] T010 [P] [US1] Move/create `ALTER TABLE` documentation to `docs/_docs/references/sql-commands/ddl/alter_table.md`
- [x] T011 [P] [US1] Move/create `DROP TABLE` documentation to `docs/_docs/references/sql-commands/ddl/drop_table.md`
- [x] T012 [P] [US1] Move/create `INSERT` documentation to `docs/_docs/references/sql-commands/dml/insert.md`
- [x] T013 [P] [US1] Move/create `UPDATE` documentation to `docs/_docs/references/sql-commands/dml/update.md`
- [x] T014 [P] [US1] Move/create `DELETE` documentation to `docs/_docs/references/sql-commands/dml/delete.md`
- [x] T015 [P] [US1] Move/create `SELECT` documentation to `docs/_docs/references/sql-commands/dql/select.md`
- [x] T016 [P] [US1] Move/create `BEGIN`, `COMMIT`, `ROLLBACK` documentation to `docs/_docs/references/sql-commands/tcl/transaction.md`
- [x] T017 [US1] Update documentation navigation configuration (e.g., in `mkdocs.yml`) to reflect the new `sql-commands` structure

**Checkpoint**: At this point, the documentation should be restructured, buildable, and navigable without broken links.

---

## Phase 4: User Story 2 - Railroad Syntax Diagrams (Priority: P2)

**Goal**: Add a visual "railroad" syntax diagram on each SQL command documentation page using the DuckDB javascript library.

**Independent Test**: Load a command page (e.g., `SELECT`) in a browser and verify the SVG renders correctly.

### Implementation for User Story 2

- [x] T018 [US2] Copy or create the core `railroad-diagrams.js` (and CSS if any) into a static assets directory like `docs/assets/js/railroad.js`
- [x] T019 [US2] Update the documentation layout/template to include the `railroad.js` script on SQL command pages
- [x] T020 [US2] Implement the Javascript diagram generator for `SELECT` syntax and embed it in `docs/_docs/references/sql-commands/dql/select.md`
- [x] T021 [P] [US2] Implement the Javascript diagram generator for `CREATE TABLE` syntax and embed it in `docs/_docs/references/sql-commands/ddl/create_table.md`
- [x] T022 [P] [US2] Implement the Javascript diagram generator for `INSERT` syntax and embed it in `docs/_docs/references/sql-commands/dml/insert.md`
- [x] T023 [P] [US2] Implement diagram generators for the remaining SQL commands moved in US1

**Checkpoint**: At this point, Railroad diagrams should appear on all migrated SQL command pages.

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Ensure links are valid and presentation is clean.

- [x] T024 Search for and fix any broken links pointing to old SQL documentation paths across all `docs/**/*.md` files.
- [x] T025 Verify the railroad diagrams look visually correct (colors, spacing, font sizes) within the documentation theme.