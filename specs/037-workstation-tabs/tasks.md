# Implementation Tasks: Workstation Sidebar Tabs

**Feature**: Workstation Sidebar Tabs
**Branch**: `037-workstation-tabs`

## Phase 1: Setup

*Project initialization and architecture scaffolding*

- [ ] T001 Verify existing Unpoly/DaisyUI setup in `src/bin/workspace/templates/workspace_layout.html`
- [ ] T002 Identify routing mechanism in `src/bin/workspace/` (axum routes or similar)

## Phase 2: Foundational

*Core architecture required by all stories*

- [ ] T003 Update `src/bin/workspace/templates/workspace_layout.html` to add the DaisyUI tab container in the sidebar.
- [ ] T004 Rename `#schema-tree` in layout to `#sidebar-content` for broader semantics.

## Phase 3: User Story 1 (US1) - Data Section Navigation

*Goal: Preserve existing schema tree functionality within the new "Data" tab.*
*Test: Clicking "Data" tab renders the schema tree.*

- [ ] T005 [P] [US1] Create backend route handler for `/workspace/sidebar/data` in `src/bin/workspace/` (moving existing `/workspace/sidebar` logic if needed).
- [ ] T006 [P] [US1] Rename/refactor `workspace_sidebar.html` to `workspace_sidebar_data.html` if necessary, or ensure it correctly renders just the data tree list.
- [ ] T007 [US1] Wire up the "Data" tab link in `workspace_layout.html` to use `up-target="#sidebar-content"` and point to `/workspace/sidebar/data`.

## Phase 4: User Story 2 (US2) - Compute Section Navigation

*Goal: Add a "Compute" tab with links to functions, procedures, etc.*
*Test: Clicking "Compute" tab lists compute resources.*

- [ ] T008 [P] [US2] Create template `src/bin/workspace/templates/workspace_sidebar_compute.html` with links for Query Console, Functions, Procedures, Triggers, Crons.
- [ ] T009 [P] [US2] Create backend route handler for `/workspace/sidebar/compute` in `src/bin/workspace/` to serve the new template.
- [ ] T010 [US2] Wire up the "Compute" tab link in `workspace_layout.html` to point to `/workspace/sidebar/compute`.
- [ ] T011 [US2] Ensure "Query Console" link inside the compute tab correctly targets the main content area (`up-target="[up-main='content']"`).

## Phase 5: User Story 3 (US3) - Observe Section Navigation

*Goal: Add an "Observe" tab with links to traces and logs from the system schema.*
*Test: Clicking "Observe" tab lists observability resources.*

- [ ] T012 [P] [US3] Create template `src/bin/workspace/templates/workspace_sidebar_observe.html` with links for Traces and Logs.
- [ ] T013 [P] [US3] Create backend route handler for `/workspace/sidebar/observe` in `src/bin/workspace/` to serve the new template.
- [ ] T014 [US3] Wire up the "Observe" tab link in `workspace_layout.html` to point to `/workspace/sidebar/observe`.

## Phase 6: Polish

*Cross-cutting concerns, cleanup, and final verification*

- [ ] T015 Ensure default tab loading logic (e.g., loading Data tab on initial visit) works correctly with Unpoly in `workspace_layout.html`.
- [ ] T016 Verify styling and active states of tabs using DaisyUI classes.

## Dependencies

- **US1** depends on **Foundational**.
- **US2** depends on **Foundational**.
- **US3** depends on **Foundational**.
- US1, US2, and US3 can be developed in parallel once Foundational is complete.

## Parallel Execution Examples

- Dev A builds US1 (Data route and tab wiring).
- Dev B builds US2 (Compute route, template, and tab wiring).
- Dev C builds US3 (Observe route, template, and tab wiring).

## Implementation Strategy

1. **Foundational MVP**: Add the visual tabs to the layout.
2. **US1 (Data MVP)**: Re-wire the existing schema tree to work inside the new "Data" tab mechanism. This ensures no regression.
3. **US2 & US3**: Add the static (or simple dynamic) templates and routes for the remaining tabs.
