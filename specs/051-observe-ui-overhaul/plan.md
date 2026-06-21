# Implementation Plan: observe-ui-overhaul

**Branch**: `051-observe-ui-overhaul` | **Date**: 2026-06-20 | **Spec**: [specs/051-observe-ui-overhaul/spec.md](./spec.md)
**Input**: Feature specification from `/specs/051-observe-ui-overhaul/spec.md`

## Summary

This plan outlines the visual, architectural, and routing overhaul to introduce state-of-the-art APM features for Logs & Traces in Oxibase Workspace:
1. **Loki-style Logs Explorer:** Fully filterable (level, keyword, trace ID), with infinite-scroll loading and clickable trace correlation badges.
2. **Tempo-style Trace summary list:** Groups individual database spans by `trace_id` to represent unique transaction runs with error indicators.
3. **Collapsible Gantt Tree Timeline:** Assembles a precise parent-child nested tree hierarchy inside the trace detail view with details drawer and deep correlation links.
4. **Auto-Refresh Toggle:** Unpoly-powered AJAX polling to support real-time workload monitoring.

---

## Technical Context

**Language/Version**: Rust 1.85+, HTML5, JavaScript (ES6), Jinja templates  
**Primary UI Frameworks**: DaisyUI 5, Tailwind CSS 4, Unpoly 3.14  
**Testing**: `cargo nextest` / `make test`  
**Target Platform**: Embedded Monolithic Workspace

---

## Constitution Check

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, HTML templates compile directly into the system binary via `include_str!`).
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (Yes, read-only queries are run on standard database endpoints).
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations? (Yes, templates and pagination handle memory footprints elegantly on the client-side).
- [x] **Safe Rust**: Are errors properly propagated? (Yes, query handlers handle and serialize errors gracefully).
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Yes).

---

## Project Structure

### Source Code Files Affected
```text
src/
├── bin/
│   └── workspace/
│       ├── templates/
│       │   ├── workspace_observe_logs.html      # NEW: Loki-style logs explorer
│       │   ├── workspace_observe_traces.html    # NEW: Tempo-style trace list
│       │   ├── workspace_sidebar_observe.html   # Update links to use observe routes
│       │   └── workspace_trace_view.html        # Update to collapsible Gantt timeline tree
│       └── mod.rs                               # Register routes/templates to database
└── server/
    ├── mod.rs                                   # Bind new axum handlers
    └── handlers.rs                              # Implement observe_logs & observe_traces handlers
```

---

## Phased Implementation Details

### Phase 0: Research & Verification
- Design secure dynamic SQL generation with parameterized bindings for filter combinations.
- Prototype the JS-based tree assembling algorithm for recursive span rendering.
- Documented in [research.md](./research.md).

### Phase 1: Design & Contracts
- Design UI entities, filter payloads, and rendering data structures.
- Defined in [data-model.md](./data-model.md) and [quickstart.md](./quickstart.md).

### Phase 2: Implementation & Code Integration
- **Step 1:** Create `workspace_observe_logs.html` containing search filter widgets, histogram top-bar, scroll loader, and correlation badges.
- **Step 2:** Create `workspace_observe_traces.html` containing summarized trace transaction rows, filtering fields, and auto-refresh polling triggers.
- **Step 3:** Overhaul `workspace_trace_view.html` to replace the flat span grid with a nested tree Gantt chart, interactive slide-drawer, and pre-filtered logs correlation links.
- **Step 4:** Update routes and side-panel links in `workspace_sidebar_observe.html`.
- **Step 5:** Register the new assets and setup routes (`/workspace/observe/logs`, `/workspace/observe/traces`) in `src/bin/workspace/mod.rs`.
- **Step 6:** Register Axum handlers in `src/server/mod.rs`.
- **Step 7:** Implement database querying handlers `workspace_observe_logs` and `workspace_observe_traces` inside `src/server/handlers.rs`.

### Phase 3: Verification & Integration Testing
- Create comprehensive integration tests:
  - `test_observe_logs_filtering` (verifies levels and text search SQL safety)
  - `test_observe_traces_grouping` (verifies trace aggregation queries)
- Verify compiling and linter suite with `make lint`.
