# Tasks: Workspace Puppeteer Testing Plan

This document contains the step-by-step task checklist required to implement the browser automated testing suite for Oxibase Workspace.

## Implementation Strategy

We follow an incremental delivery approach, completing and verifying each user story phase independently. The SQL editor and telemetry explorer tests are prioritized as foundational MVP tasks, followed by debugger WebSocket testing and schema navigation.

---

## Task Checklist

### Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Install Node.js dependencies in the workspace and initialize Puppeteer via `npm install puppeteer --no-save`
- [x] T002 Create test runner script entrypoint and imports boilerplate in `scripts/test-workspace.js`

---

### Phase 2: Foundational

**Purpose**: Core process lifecycle orchestration

- [x] T003 Implement process orchestration in `scripts/test-workspace.js` to spawn the local `oxibase serve` server, await port `8080` binding, and register robust SIGINT/SIGKILL cleanups

---

### Phase 3: User Story 1 - Interactive SQL Editor & Results Verification (Priority: P1) 🎯 MVP

**Goal**: Validate query typing, execution, and rendering of rows/errors in the editor.

* **Independent Test Criteria**: A standard SELECT query input successfully renders corresponding tabular cells in the DOM.

- [x] T004 [P] [US1] Add query typing and submission block in `scripts/test-workspace.js` targeting the `/workspace/sql_editor` textarea
- [x] T005 [P] [US1] Add assertions in `scripts/test-workspace.js` to verify `#query-results` contains expected output table rows
- [x] T006 [P] [US1] Add error rendering assertions in `scripts/test-workspace.js` for malformed SQL inputs

---

### Phase 4: User Story 2 - Logs Explorer & Trace Timeline Verification (Priority: P1)

**Goal**: Validate async loading of logs severity metrics and trace timelines.

* **Independent Test Criteria**: Dynamic metrics and tree Gantt logs/traces populate asynchronously in the browser DOM.

- [x] T007 [P] [US2] Add metrics explorer assertions in `scripts/test-workspace.js` targeting `/workspace/observe/logs` severity counters
- [x] T008 [P] [US2] Add timeline Gantt assertions in `scripts/test-workspace.js` targeting traceรายละเอียด routes `/workspace/traces/{trace_id}`

---

### Phase 5: User Story 3 - Interactive Debugger Handshake & Breakpoints (Priority: P2)

**Goal**: Validate debugger WebSocket handshakes and breakpoint highlighters.

* **Independent Test Criteria**: Debugger establishes a connection, hits line breakpoints, and populates locals sidebar.

- [x] T009 [P] [US3] Add WebSocket session assertion block in `scripts/test-workspace.js` on routing to `/workspace/debugger`
- [x] T010 [P] [US3] Add breakpoint toggling and line highlight assertions in `scripts/test-workspace.js`

---

### Phase 6: User Story 4 - Database Catalog & Schema Explorer (Priority: P2)

**Goal**: Validate Sidebar Data lists catalog and dynamic routing grids.

* **Independent Test Criteria**: Navigation clicks correctly route to detailed table schemas.

- [x] T011 [P] [US4] Add sidebar list navigation clicks and routing assertions in `scripts/test-workspace.js`

---

### Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Overall validation, lints, and resource leaks checks

- [x] T012 Verify complete automated suite execution via `node scripts/test-workspace.js` finishes successfully under 45 seconds
- [x] T013 Verify that zero node, browser, or database server processes are leaked after exit

---

## Dependencies & Completion Order

```text
Phase 1: Setup -> Phase 2: Foundational -> Phase 3: User Story 1 (P1)
                                        -> Phase 4: User Story 2 (P1)
                                        -> Phase 5: User Story 3 (P2)
                                        -> Phase 6: User Story 4 (P2) -> Phase 7: Polish
```

## Parallel Execution Examples

- Task `T004` (US1 editor checks) and `T007` (US2 metrics checks) have zero mutual dependencies and can be run concurrently within different asynchronous browser contexts.
- Task `T009` (US3 WS handshake) and `T011` (US4 sidebar clicks) can be developed/executed in parallel inside separate tests.
