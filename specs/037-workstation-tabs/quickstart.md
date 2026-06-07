# Quickstart: Workstation Sidebar Tabs

This feature adds a tabbed interface to the left sidebar of the Oxibase workspace, allowing users to navigate between Data, Compute, and Observe domains.

## Development Setup

No special setup is required beyond the standard Oxibase build process.

## How to Test

1. Build and run the `oxibase` binary with the CLI feature enabled (or run the workspace server directly if applicable).
2. Open the workstation UI in a web browser (typically `http://localhost:8080/workspace`).
3. Verify the left sidebar now contains three tabs: "Compute", "Data", and "Observe".
4. Click the "Data" tab. The existing tree of schemas and tables should be displayed.
5. Click the "Compute" tab. A list of compute-related options should appear, including "Query Console", "Functions", "Procedures", "Triggers", and "Crons". Clicking "Query Console" should open the SQL editor.
6. Click the "Observe" tab. Options for "Traces" and "Logs" should appear.

## Implementation Notes

- **UI Framework:** The workspace uses DaisyUI and Unpoly. The tabs should be implemented using DaisyUI's tab components (`tabs tabs-boxed` or similar).
- **Navigation:** Unpoly's `up-target` and `up-nav` should be used to load tab contents asynchronously without a full page reload.
- **Templates:** The layout changes will primarily involve `src/bin/workspace/templates/workspace_layout.html` to add the tab structure, and the creation of new template files for the content of the Compute and Observe tabs.
