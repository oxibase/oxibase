# Feature Specification: Workspace Puppeteer Testing Plan

**Feature Branch**: `054-workspace-puppeteer-tests`  
**Created**: Tue Jun 23 2026  
**Status**: Draft  
**Input**: User description: "a testing plan for all the workspace features using puppeteer"

## User Scenarios & Testing *(mandatory)*

This feature defines a comprehensive end-to-end automated testing plan using Puppeteer to validate all interactive GUI functionalities of the Oxibase Workspace. The plan is designed to be fully testable and structured as prioritizable user journeys.

### User Story 1 - Interactive SQL Editor & Results Verification (Priority: P1)

As a database administrator, I want to verify that typing a SQL query in the editor, clicking "Execute", and viewing the results grid works seamlessly in a headless web browser.

**Why this priority**: Core developer productivity depends on the query editor; it is the primary interface of the workspace.

**Independent Test**: Can be verified by running the Puppeteer script against a running Oxibase server and asserting the presence of the resulting cells.

**Acceptance Scenarios**:
1. **Given** a running Oxibase server with seed data, **When** Puppeteer opens `http://localhost:8080/workspace/sql_editor`, types `SELECT * FROM system.logs;`, and clicks "Execute", **Then** the results pane displays a table containing rows and columns.
2. **Given** an invalid query, **When** executed, **Then** a visible error box with a descriptive database message is rendered.

---

### User Story 2 - Logs Explorer & Trace Timeline Verification (Priority: P1)

As a developer, I want to verify that the Logs Explorer and Trace Gantt chart correctly load telemetry data asynchronously via Fetch AJAX calls without crashing the UI.

**Why this priority**: Decoherence of logs or trace timeline dashboards breaks workspace monitoring.

**Independent Test**: Can be verified by a Puppeteer test script asserting AJAX-updated DOM elements.

**Acceptance Scenarios**:
1. **Given** active database trace and log entries, **When** Puppeteer navigates to `/workspace/observe/logs`, **Then** the severity summary widgets (ERROR, WARN, INFO, DEBUG) populate dynamically with correct non-zero integers.
2. **Given** a specific `trace_id` URL `/workspace/traces/{trace_id}`, **When** the page loads, **Then** a Gantt timeline chart with hierarchically structured tree nodes is built and inspectable.

---

### User Story 3 - Interactive Debugger Handshake & Breakpoints (Priority: P2)

As a developer, I want to verify that the browser-based workspace debugger successfully establishes a WebSocket session, sets breakpoints, runs a stored procedure via the Run modal, and steps through execution.

**Why this priority**: Testing the browser-based debugging loop is highly valuable for the PL/SQL and Rhai runtime developer experience.

**Independent Test**: Can be verified by running the Puppeteer test with mock procedure triggers.

**Acceptance Scenarios**:
1. **Given** a stored procedure in the database, **When** Puppeteer launches the debugger panel, **Then** the system establishes a WebSocket connection and enables the breakpoint gutters.
2. **Given** a breakpoint on line 1, **When** the procedure is invoked, **Then** the UI shows a "Paused" state, highlighting the active line and populating local variables.

---

### User Story 4 - Database Catalog & Schema Explorer (Priority: P2)

As a user, I want to navigate the Sidebar's Data section and explore custom schemas, tables, and views dynamically.

**Why this priority**: Validates primary navigation and route-context compilation across multiple tables.

**Independent Test**: Verified by Puppeteer simulating sidebar clicks and verifying data grid routing.

**Acceptance Scenarios**:
1. **Given** multiple schemas, **When** the sidebar is clicked, **Then** the table catalog loads, and clicking a table routes to `/workspace/data/{schema}/{table}` showing its rows.

### Edge Cases

- **Slow Network / Latency**: Puppeteer tests must support configurable wait/timeout parameters to avoid flaky assertions when fetching large telemetry tables.
- **Empty Datasets**: When there are no logs or traces, the dashboards must render friendly fallback messages rather than blank pages or infinite loading spinners.
- **Browser Disconnection**: Handlers must gracefully cleanup WebSockets or TCP ports if the browser or headless client disconnects abruptly.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The testing harness MUST support a self-contained JavaScript/TypeScript testing suite using the Puppeteer library.
- **FR-002**: The harness MUST support spawning the `oxibase` server in a separate background thread or utilizing an already active database server process.
- **FR-003**: The test runner MUST verify full page loads, form submissions, dynamic sidebar updates, and AJAX Fetch/RPC updates on all routes.
- **FR-004**: The suite MUST assert that the SQL editor runs and displays correct result counts and table structures in the browser DOM.
- **FR-005**: The suite MUST verify that the Trace Gantt chart correctly calculates width and position of timing blocks from AJAX JSON payloads.
- **FR-006**: The suite MUST support headless and headful execution modes via environment configuration (e.g. `HEADLESS=true`).
- **FR-007**: The harness MUST use a custom, self-contained vanilla Node.js script using native assertions to execute and coordinate Puppeteer tests, avoiding heavy external test runner dependencies.

### Key Entities

- **[Puppeteer Test Runner]**: Orchestrates browser automation, handles logins (if authentication is active), and performs UI assertions.
- **[Oxibase Serve Subcommand]**: The web server hosting the Workspace HTML/JS pages and API endpoints being tested.
- **[Workspace Web UI]**: The actual client-side single-page app containing editor elements, observation dashboards, and debugging frames.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The suite passes 100% of Puppeteer tests verifying navigation and AJAX loading across logs, traces, SQL editor, and data catalog.
- **SC-002**: Total test execution time is under 45 seconds for a complete local run.
- **SC-003**: No zombie browser or server processes are left running after test execution exits.

## Assumptions

- Assumes the local testing environment has Node.js (v18+) and Chromium/Chrome installed or download-enabled by Puppeteer.
- Assumes the database server binds to local loopback `127.0.0.1` and is accessible by the browser agent.
