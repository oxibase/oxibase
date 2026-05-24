# Feature Specification: Replace Tailwind with DaisyUI in Workspace App

**Feature Branch**: `029-metadata-api-workspace`  
**Created**: 2026-05-24  
**Status**: Draft  
**Input**: User description: "change tailwind for daisyUI"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - DaisyUI Integration (Priority: P1)

As a developer, I want to use DaisyUI component classes instead of raw Tailwind utility classes so that the HTML templates are cleaner and easier to maintain.

**Why this priority**: Using semantic components drastically improves readability of Minijinja templates and standardizes the UI look and feel.

**Independent Test**: Load the workspace at `http://localhost:8080/workspace` and visually confirm that the layout, sidebar, tables, and buttons render correctly using DaisyUI styles.

**Acceptance Scenarios**:

1. **Given** the workspace HTML templates, **When** they are served, **Then** they include the DaisyUI CDN link and Tailwind browser script.
2. **Given** the Workspace app is loaded, **When** I view the UI components (buttons, inputs, tables, layout), **Then** they use DaisyUI classes (e.g., `btn`, `input`, `table`) instead of verbose Tailwind utility strings.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Replace raw TailwindCSS CDN link with the DaisyUI v5 CDN link and Tailwind browser script in `workspace_layout.html`.
- **FR-002**: Refactor existing HTML templates (`workspace_sidebar.html`, `workspace_sql_editor.html`, `workspace_sql_results.html`, `workspace_table_create.html`, `workspace_data_grid.html`) to use DaisyUI component classes instead of raw Tailwind utility classes.
- **FR-003**: Maintain existing Unpoly (`up-*`) attributes and template logic (`{% %}`) intact.

### Key Entities

- **HTML Templates**: The `.html` files in `src/bin/workspace/templates/` containing the UI structure.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All `.html` files in the workspace templates directory use DaisyUI classes.
- **SC-002**: The application visually renders correctly with DaisyUI styling.

## Assumptions

- The frontend will rely on CDNs for DaisyUI (Tailwind v4 browser script) and Unpoly.
- No changes to backend Rust code or API logic are required for this pure UI migration.