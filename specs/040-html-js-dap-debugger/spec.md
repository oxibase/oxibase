# Feature Specification: HTML/JS DAP Debugger Frontend

**Feature Branch**: `040-html-js-dap-debugger`  
**Created**: 2026-06-14  
**Status**: Draft  
**Input**: User description: "html-js dap debugger"

### Clarifications

#### Session 2026-06-14
- Q: Which languages should be supported in the debugger? → A: PL/SQL, Rhai, and Python.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Connect and View Code (Priority: P1)

A database developer working in the web workspace wants to view their PL/SQL, Rhai or Python procedure code and attach to an active debug session so they can prepare to debug their logic.

**Why this priority**: Establishing the connection, rendering the code editor, and subscribing to the DAP WebSocket are the foundational steps required before any debugging actions can occur.

**Independent Test**: Can be tested by opening the workspace UI, navigating to the compute/SQL view, and ensuring the Monaco Editor initializes and successfully connects a WebSocket to the Oxibase backend without immediate errors.

**Acceptance Scenarios**:

1. **Given** the workspace UI is loaded, **When** the developer opens a PL/SQL procedure, **Then** the procedure code is rendered in a Monaco Editor instance.
2. **Given** the code editor is visible, **When** the workspace initializes, **Then** a background WebSocket connection is established to the Oxibase DAP endpoint (`/workspace/dap-ws`).

---

### User Story 2 - Manage Breakpoints (Priority: P2)

A database developer wants to set and remove breakpoints in their procedure/function code in pl/SQL, Rhai or Python, directly in the web UI so they can specify where execution should pause.

**Why this priority**: Breakpoints are the primary mechanism for controlling debug flow. Without them, users cannot stop execution at specific points of interest.

**Independent Test**: Can be tested by clicking the editor margin to set a breakpoint, verifying the UI shows a visual indicator, and confirming the `setBreakpoints` DAP request is sent over the WebSocket.

**Acceptance Scenarios**:

1. **Given** a procedure is open in the editor, **When** the user clicks the editor margin on line 10, **Then** a red breakpoint indicator appears, AND a DAP `setBreakpoints` request is sent to the backend.
2. **Given** a breakpoint exists on line 10, **When** the user clicks the indicator again, **Then** the indicator is removed, AND an updated DAP `setBreakpoints` request is sent.

---

### User Story 3 - Pause Execution and Inspect State (Priority: P3)

A database developer wants to step through paused code and inspect the current variables so they can understand the internal state of their procedure.

**Why this priority**: State inspection is the core value proposition of a debugger. This builds upon the connection (P1) and breakpoints (P2).

**Independent Test**: Can be tested by triggering a `stopped` event from the backend, verifying the UI enters a "paused" state, and confirming the variables panel populates with data fetched via DAP requests.

**Acceptance Scenarios**:

1. **Given** an active debug session with a set breakpoint, **When** the backend execution hits the breakpoint and sends a `stopped` event, **Then** the UI highlights the current line in the editor AND enables the debugging controls (Step Over, Continue).
2. **Given** the execution is paused, **When** the UI receives the stopped state, **Then** it automatically requests `threads`, `stackTrace`, `scopes`, and `variables` via DAP, AND displays the local variables in a collapsible tree in the sidebar.
3. **Given** the execution is paused, **When** the user clicks "Step Over", **Then** a DAP `next` request is sent, the UI temporarily disables controls, and awaits the next `stopped` event.

### Edge Cases

- What happens if the WebSocket connection drops during an active debug session?
- How does the UI behave if a procedure is modified while a debug session is paused?
- How does the UI handle very large objects or deep variable hierarchies (e.g., lazy loading vs. fetching all)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The UI MUST embed CodeMirror editor to display procedure code and support breakpoint gutter interactions.
- **FR-002**: The frontend MUST include a standalone, vanilla JavaScript DAP Client Library (`dap-client.js`) to handle JSON-RPC message framing over WebSockets.
- **FR-003**: The DAP Client MUST expose a Promise-based API for outgoing requests and an Event Emitter pattern for incoming events.
- **FR-004**: The workspace layout MUST persist the debugger state and WebSocket connection during Unpoly fragment navigations (e.g., using `[up-keep]`).
- **FR-005**: The UI MUST render debugging controls (Continue, Step Over, Stop) that are enabled only when the session is paused.
- **FR-006**: The UI MUST display a variables tree using native HTML elements (e.g., `<details>`, `<summary>`) populated from DAP variable responses.
- **FR-007**: The backend Oxibase workspace server MUST expose a WebSocket endpoint (`/workspace/dap-ws`) that bridges JSON messages to the internal `DebugController`.

### Key Entities

- **DAP Client**: The vanilla JS class managing the WebSocket lifecycle and JSON-RPC mapping.
- **CodeMirror Editor Instance**: The visual component rendering the source code and breakpoint indicators.
- **Variables Tree**: The DOM structure representing the current execution scope's state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The DAP Client library is written in vanilla JavaScript without requiring a heavy frontend framework build step (e.g., no React/Vue required for the core client).
- **SC-002**: A user can successfully set a breakpoint, trigger a procedure execution, and see the UI pause at the correct line.
- **SC-003**: Navigating between "Compute" and "Data" tabs in the workspace does not disconnect the active debugging WebSocket.
- **SC-004**: The variables panel accurately reflects the local environment state of the paused PL/SQL, Rhai or Python procedure.

## Assumptions

- The Oxibase backend already implements (or is implementing via `039-dap-plsql`) the internal `DebugController` capable of receiving DAP requests and pausing execution.
- Modern browser WebSocket support is assumed.
- The UI will utilize the existing DaisyUI/Tailwind CSS setup for styling the debugger components.
