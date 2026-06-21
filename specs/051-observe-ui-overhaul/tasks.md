# Tasks: Observability Dashboard and UI Overhaul

**Input**: Design documents from `/specs/051-observe-ui-overhaul/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

---

## Phase 1: Setup

**Purpose**: Validate baseline build and test stability before making any UI/UX changes.

- [X] T001 Verify project compiles and existing test suite passes using `cargo nextest run`

---

## Phase 2: Foundational

**Purpose**: Bind essential backend routings and template assets that support both observe templates.

- [X] T002 Register Axum handler endpoints for `/workspace/observe/logs` and `/workspace/observe/traces` in `src/server/mod.rs`
- [X] T003 Bind registration statements for `workspace_observe_logs.html` and `workspace_observe_traces.html` in `src/bin/workspace/mod.rs`

---

## Phase 3: User Story 1 - Loki-style Logs Explorer

**Goal**: Expose a rich, search-and-level filterable log stream dashboard with infinite scrolling.

**Independent Test**: Can navigate to `/workspace/observe/logs` and search/filter log lists.

- [X] T004 [P] [US1] Create the Log Explorer Jinja template at `src/bin/workspace/templates/workspace_observe_logs.html` with filter controls, histogram bar, scroll loading, and collapsible details
- [X] T005 [US1] Implement dynamic parameterized SQL log filtration in `workspace_observe_logs` handler inside `src/server/handlers.rs`

---

## Phase 4: User Story 2 - Tempo-style Hierarchical Trace Timeline

**Goal**: Introduce grouped trace list summaries and collapsible Gantt tree timelines with detail side drawers.

**Independent Test**: Can open trace summaries at `/workspace/observe/traces` and inspect recursive span timelines.

- [X] T006 [P] [US2] Create the Trace summary list Jinja template at `src/bin/workspace/templates/workspace_observe_traces.html`
- [X] T007 [US2] Implement grouped-by transaction database query logic for `workspace_observe_traces` in `src/server/handlers.rs`
- [X] T008 [US2] Overhaul detail view `workspace_trace_view` handler to return complete span listings in `src/server/handlers.rs`
- [X] T009 [US2] Implement tree assembler and interactive Gantt collapsing UI in `src/bin/workspace/templates/workspace_trace_view.html`

---

## Phase 5: User Story 3 - Observability Control Center (Auto-Refresh)

**Goal**: Support real-time AJAX polling of traces/logs and sidebar navigation links.

**Independent Test**: Sidebar links open dashboards and auto-polling triggers partial updates periodically.

- [X] T010 [US3] Update observation side panel links in `src/bin/workspace/templates/workspace_sidebar_observe.html`
- [X] T011 [P] [US3] Add auto-refresh polling triggers utilizing Unpoly `up-poll` in `src/bin/workspace/templates/workspace_observe_logs.html`
- [X] T012 [P] [US3] Add auto-refresh polling triggers utilizing Unpoly `up-poll` in `src/bin/workspace/templates/workspace_observe_traces.html`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Comprehensive testing, linter compliance, and asset cleanup.

- [X] T013 Create integration tests for logs and traces routing inside `tests/procedure_plsql_tests.rs`
- [X] T014 Run formatting and clippy suite using `make lint` and fix any warnings

---

## Dependencies & Completion Order

```text
       [Phase 1: Setup]
              │
              ▼
    [Phase 2: Foundational]
         /         \
        ▼           ▼
     [US1]        [US2]
  (Logs Exp)   (Trace Gantt)
        \           /
         ▼         ▼
          [US3]
      (Auto-Refresh)
              │
              ▼
      [Phase 6: Polish]
```

---

## Parallel Execution Opportunities

- **T004 (Logs template)** and **T006 (Traces list template)** can be scaffolded concurrently since they are isolated HTML files with no overlapping bindings.
- **T011** and **T012** (applying `up-poll` to respective templates) are completely parallelizable across different directories.
