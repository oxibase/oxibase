# Research: Workstation Sidebar Tabs

## Context
The goal is to implement a tabbed interface in the left sidebar of the `workspace` UI. The tabs should be "Compute", "Data", and "Observe".
- "Data" should show the existing tree of tables.
- "Compute" should show query console, functions, procedures, triggers, crons.
- "Observe" should show traces and logs.

## Findings
- The UI is currently built using HTML templates located in `src/bin/workspace/templates/`.
- `workspace_layout.html` defines the overall layout, including the sidebar container.
- `workspace_sidebar.html` renders the content of the sidebar, which is currently just the table tree.
- The UI uses `unpoly` for dynamic page updates without full reloads (`up-target`, `up-nav`).
- The backend for these routes is likely in `src/bin/workspace/` (probably in an `api` or `routes` module).

## Design Decisions
1.  **Layout Updates:** Modify `workspace_layout.html` to include a tab bar at the top of the sidebar.
2.  **Tab Navigation:** Use Unpoly to load the content of each tab dynamically into the sidebar content area (`#schema-tree` which should probably be renamed to `#sidebar-content`).
3.  **Backend Routes:** Ensure routes exist for `/workspace/sidebar/data`, `/workspace/sidebar/compute`, and `/workspace/sidebar/observe` that return the HTML for the respective tab contents.

## Alternatives Considered
-   **Client-side routing (React/Vue):** Rejected as it goes against the existing Unpoly/HTML template architecture.
-   **Single page loading all tabs:** Rejected as it might slow down initial load time if the compute/observe data is large or slow to fetch. Dynamic loading via Unpoly is better.
