# Feature Specification: SQL Command Documentation & Railroad Diagrams

**Feature Branch**: `031-sql-railroad-diagrams`  
**Created**: 2026-05-25  
**Status**: Draft  
**Input**: User description: "i want to take the docs > references > sql commands and split it into subdirectories for DML, DDL, transaction ... and then one page per command, like select, insert ... and in each page add a railroad syntax diagram similar to duckdb"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently via `cargo nextest`
  - Deployed independently
-->

### User Story 1 - SQL Documentation Restructuring (Priority: P1)

As a database user, I want the SQL commands reference documentation organized logically into categories (DDL, DML, Transactions, etc.) with dedicated pages per command, so that I can easily find the specific command I need.

**Why this priority**: A well-structured documentation reference is foundational. Before adding complex diagrams, the base structure must be organized intuitively.

**Independent Test**: Can be fully tested by verifying the documentation directory structure and ensuring navigation links exist and resolve correctly to individual command pages.

**Acceptance Scenarios**:

1. **Given** the current flat `docs/references/sql_commands` structure, **When** the restructuring is applied, **Then** there should be subdirectories for `ddl`, `dml`, `transaction`, etc.
2. **Given** the new structure, **When** I navigate to `ddl`, **Then** I see individual markdown pages for `create_table`, `alter_table`, etc.

---

### User Story 2 - Railroad Syntax Diagrams (Priority: P2)

As a database user, I want to see a visual "railroad" syntax diagram on each SQL command documentation page, so that I can quickly and accurately understand the allowed syntax and structure for that command.

**Why this priority**: Visual aids significantly reduce the cognitive load for users learning the SQL dialect, directly improving developer experience.

**Independent Test**: Can be tested by navigating to a restructured command page (e.g., `SELECT`) and verifying that an SVG or HTML-based syntax diagram renders correctly and accurately reflects the command's grammar.

**Acceptance Scenarios**:

1. **Given** a documentation page for the `SELECT` command, **When** I view the page, **Then** a railroad syntax diagram depicting the `SELECT` syntax is visible at the top of the reference section.
2. **Given** the documentation generator, **When** a new command syntax is defined, **Then** the diagram generation tool produces an updated and accurate visual diagram for that command.

---

### Edge Cases

- What happens if a SQL command spans multiple categories (e.g., a utility command that feels like both DDL and admin)?
- How are complex, deeply nested SQL syntaxes (like nested `SELECT` within `INSERT`) represented visually without becoming unreadable?
- How is the diagram rendered if JavaScript is disabled in the user's browser (assuming a JS-based renderer like the DuckDB example)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Documentation MUST be organized into a categorical directory structure (e.g., DML, DDL, DQL, DCL, TCL).
- **FR-002**: Every supported SQL command MUST have its own dedicated documentation file within the appropriate category subdirectory.
- **FR-003**: The documentation generator or rendering pipeline MUST support the inclusion of railroad syntax diagrams.
- **FR-004**: Each SQL command documentation page MUST include a railroad syntax diagram representing its grammar.
- **FR-005**: The railroad diagram generator MUST generate SVGs dynamically at runtime on the client-side utilizing the DuckDB railroad-diagrams javascript library.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of existing documented SQL commands are migrated to the new category-based directory structure.
- **SC-002**: 100% of the individual SQL command pages include a corresponding railroad syntax diagram.
- **SC-003**: The new documentation structure builds successfully without broken internal links.
- **SC-004**: Users can navigate from the main SQL reference index to any individual command page in 2 clicks or fewer.

## Assumptions

- We are using a static site generator or documentation framework that supports embedding custom HTML/JS or SVGs.
- The SQL dialect's grammar is sufficiently documented internally to allow for accurate diagram creation.
