# Tasks: HTML/JS DAP Debugger Frontend

**Input**: Design documents from `/specs/040-html-js-dap-debugger/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: MUST include corresponding `cargo nextest` integration or unit tests for any new feature. 

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Initialize DAP websocket route placeholder in `src/bin/workspace/routes/mod.rs` (or equivalent routing file)
- [x] T002 Create empty `dap-client.js` in `src/bin/workspace/static/js/dap-client.js`
- [x] T003 Verify workspace server compiles and runs

---

## Phase 2: Foundational (Backend DAP WebSocket Bridge)

**Purpose**: Create the backend WebSocket endpoint that serves strict DAP payloads (including `Content-Length` headers). This is a prerequisite for all frontend interactions.

- [x] T010 Implement `/workspace/dap-ws` endpoint in `src/bin/workspace/routes/` to accept WebSocket connections.
- [x] T011 Implement stream parsing/framing in Rust to inject standard DAP HTTP-like `Content-Length` headers for outgoing messages to the browser.
- [x] T012 Implement stream parsing/framing in Rust to parse incoming WebSocket string messages containing `Content-Length` headers from the browser and pass the JSON to the internal `DebugController`.

---

## Phase 3: User Story 1 - Connect and View Code (Priority: P1) 🎯 MVP

**Goal**: A database developer working in the web workspace wants to view their PL/SQL, Rhai or Python procedure code and attach to an active debug session ONLY when a specific procedure or function has been selected from the workstation list.

**Independent Test**: Connect to the workspace, select a procedure, and verify the CodeMirror editor mounts and the WebSocket connects successfully.

### Implementation for User Story 1

- [x] T020 [P] [US1] Update `src/bin/workspace/templates/workspace_sql_editor.html` to conditionally load CodeMirror 6 instead of a `<textarea>` ONLY when a procedure/function is selected.
- [x] T021 [US1] Implement the `DAPClient` class in `src/bin/workspace/static/js/dap-client.js` to establish a WebSocket connection.
- [x] T022 [US1] Implement string buffer parser in `DAPClient` to read `Content-Length` headers and extract the JSON payloads.
- [x] T023 [US1] Initialize `DAPClient` connection when the CodeMirror editor is mounted for a selected procedure.
- [x] T024 [US1] Update workspace layout to include `[up-keep]` attribute around the debugger workspace container to persist state across Unpoly navigations.

**Checkpoint**: At this point, opening a procedure should render the code in CodeMirror and establish a stable background WebSocket connection parsing headers correctly.

---

## Phase 4: User Story 2 - Manage Breakpoints (Priority: P2)

**Goal**: A database developer wants to set and remove breakpoints directly in the web UI so they can specify where execution should pause.

**Independent Test**: Click the editor margin and verify the red dot appears and the `setBreakpoints` DAP request is transmitted.

### Implementation for User Story 2

- [x] T030 [P] [US1] Add `sendRequest(command, args)` method returning a Promise to `DAPClient` in `src/bin/workspace/static/js/dap-client.js`.
- [x] T031 [US1] Configure CodeMirror 6 gutter extension in `workspace_sql_editor.html` to display breakpoint indicators (red dots).
- [x] T032 [US1] Add click listener to CodeMirror gutter to toggle breakpoint state for the specific line.
- [x] T033 [US1] Map gutter clicks to trigger `dapClient.sendRequest('setBreakpoints', ...)` formatting the payload according to the DAP spec.

**Checkpoint**: Users can visually toggle breakpoints, sending correct DAP requests over the WebSocket.

---

## Phase 5: User Story 3 - Pause Execution and Inspect State (Priority: P3)

**Goal**: A database developer wants to step through paused code and inspect the current variables so they can understand the internal state of their procedure.

**Independent Test**: Backend `stopped` event triggers UI update, controls enable, and variables fetch.

### Implementation for User Story 3

- [x] T040 [P] [US3] Add `on(event, callback)` event emitter logic to `DAPClient` in `src/bin/workspace/static/js/dap-client.js`.
- [x] T041 [US3] Create debug toolbar UI in `workspace_sql_editor.html` with Continue, Step Over, and Stop buttons (disabled by default).
- [x] T042 [US3] Listen for the `stopped` event in the frontend; upon receipt, enable the debug toolbar buttons and highlight the stopped line in CodeMirror.
- [x] T043 [US3] Implement the sequential request chain on pause: request `threads` -> `stackTrace` -> `scopes` -> `variables`.
- [x] T044 [US3] Render fetched variables in `workspace_sql_editor.html` using native `<details>` and `<summary>` tags for a collapsible tree in a docked view below the editor.
- [x] T045 [US3] Wire debug toolbar buttons to send DAP `continue`, `next`, and `disconnect` requests.
- [ ] T046 [US3] Add Call Stack panel UI adjacent to Variables in the docked layout to show the current `stackTrace` response.
- [ ] T047 [US3] Add Interactive Debug Console/REPL panel UI allowing users to input expressions.
- [ ] T048 [US3] Wire Debug Console input to send DAP `evaluate` requests and display the result string.

**Checkpoint**: The full debug layout is functional; pausing execution populates the variables tree, call stack, and enables console evaluation.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T050 [P] Ensure CSS styles for debug components use existing DaisyUI/Tailwind classes.
- [x] T051 Verify Unpoly fragment updates do not duplicate WebSocket connections or leak memory.
- [x] T052 Verify `unwrap()` and `expect()` are not used inappropriately in the Rust WebSocket handler.
- [x] T053 Run `make lint` and `make license` to ensure compliance.