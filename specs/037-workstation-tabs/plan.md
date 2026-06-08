# Implementation Plan: Workstation Sidebar Tabs

**Branch**: `037-workstation-tabs` | **Date**: 2026-06-07 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/037-workstation-tabs/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

This feature replaces the single-purpose schema tree sidebar in the workstation with a tabbed interface separating Data, Compute, and Observe domains. The Observe tab will fetch its traces and logs data from the tables located in the `system` schema. The UI changes will be implemented using HTML templates and Unpoly for dynamic content loading.

## Technical Context

**Language/Version**: HTML/Tera (templates), Rust 1.85+ (backend routes)
**Primary Dependencies**: unpoly, daisyui, tailwindcss, axum (or similar for serving)
**Testing**: cargo nextest (via `make test` / `make test-all`)
**Target Platform**: Web Browser (Workstation UI)
**Performance Goals**: Fast UI updates without full page reloads.
**Constraints**: Must integrate with the existing Unpoly/DaisyUI architecture. No new external JS frameworks.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Mainframe Monolith**: Does this maintain the embedded/monolith architecture? (Yes, UI is served directly)
- [x] **ACID & MVCC**: Does this change respect multi-version concurrency control and strict data integrity? (N/A, UI only)
- [x] **Memory Efficiency**: Does this avoid unnecessary allocations (e.g., `Vec` clones)? (N/A, UI only)
- [x] **Safe Rust**: Are errors properly propagated? (Yes, backend route handlers will use standard error propagation)
- [x] **Tests First**: Will this be covered by tests that can be verified via `cargo nextest`? (Integration tests for new UI routes should be added)

## Project Structure

### Documentation (this feature)

```text
specs/037-workstation-tabs/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/bin/workspace/
├── templates/
│   ├── workspace_layout.html   # Needs update to add tab structure
│   ├── workspace_sidebar.html  # To be renamed or adapted for "Data" tab
│   ├── workspace_compute.html  # NEW: Content for Compute tab
│   └── workspace_observe.html  # NEW: Content for Observe tab
└── routes/                     # (or similar) Needs new endpoints for the tabs
```

**Structure Decision**: This feature primarily impacts the `src/bin/workspace/templates` directory for UI changes and the corresponding route handlers in `src/bin/workspace/` that serve these templates.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
