# Implementation Plan: HTML/JS DAP Debugger Frontend

**Branch**: `040-html-js-dap-debugger` | **Date**: 2026-06-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/040-html-js-dap-debugger/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

This feature adds a web-based debugger frontend to the Oxibase workspace. It embeds CodeMirror 6 for source code viewing and breakpoint management, uses a custom vanilla JavaScript DAP client to communicate over WebSockets with standard DAP headers, and renders variables and execution state using native HTML elements, preserving state across Unpoly fragment navigations. The debugging experience is conditionally available only when a procedure or function has been selected from the workstation list.

## Technical Context

**Language/Version**: HTML, CSS (Tailwind/DaisyUI), JavaScript (Vanilla ES6+), Rust 1.85+ (Backend WS endpoint)
**Primary Dependencies**: CodeMirror 6 (via CDN or bundled), Unpoly (existing)
**Testing**: Unit tests for `DAPClient` (if applicable), manual E2E via browser.
**Target Platform**: Embedded Monolithic DB Web Workspace (Linux, macOS, Windows)
**Performance Goals**: Minimal overhead; DAP client and CodeMirror should not block the main thread; WebSocket communication should be efficient.
**Constraints**: 
- Must use vanilla JS for the DAP Client (no heavy frameworks like React/Vue).
- Must integrate seamlessly with Unpoly (`up-keep`).
- Backend WebSocket endpoint must serve strict DAP payloads including HTTP-like `Content-Length` headers.
- The debugger UI (CodeMirror, toolbars, variable views) MUST be conditionally rendered ONLY when a procedure or function is selected in the workstation list.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, the WS endpoint is served directly by the existing workspace binary).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (N/A for frontend UI, but backend pauses will hold locks as expected for debugging).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (Yes, frontend avoids heavy virtual DOMs; backend streams WS efficiently).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, `axum` or equivalent WS handlers will map errors correctly).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes, backend WS tests; frontend UI mostly manual or browser-based integration tests).

## Project Structure

### Documentation (this feature)

```text
specs/040-html-js-dap-debugger/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── dap-websocket.md # WebSocket contract
└── tasks.md             # Phase 2 output (future)
```

### Source Code (repository root)

```text
src/
└── bin/
    └── workspace/
        ├── static/
        │   └── js/
        │       └── dap-client.js        # New: Vanilla JS DAP Client
        ├── templates/
        │   ├── workspace_sql_editor.html # Modified: Add CodeMirror and debug toolbar
        │   └── workspace_sidebar_compute.html # Modified: Add variables tree and call stack
        └── routes/                      # Modified: Add WS endpoint for `/workspace/dap-ws`
```

**Structure Decision**: This feature primarily impacts the web workspace frontend (`src/bin/workspace/templates/` and `src/bin/workspace/static/`) and adds a single WebSocket route to the workspace backend.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None      | N/A        | N/A                                 |