# Data Model: Workstation Sidebar Tabs

The primary data structures involved in this feature are related to the UI layout and routing, rather than core database storage. However, the UI needs to be aware of the different logical sections.

## Logical Entities

### Sidebar Navigation State
- **Active Tab**: Represents the currently selected tab in the sidebar (Data, Compute, or Observe). This state is managed via URL paths and Unpoly's DOM updating mechanism.

### Tab Contents
- **Data Tab Content**: Requires fetching schema and table metadata. (Already exists).
- **Compute Tab Content**: Requires logical grouping of compute resources:
    - Query Console (Link to existing route)
    - Functions (List of registered user-defined functions)
    - Procedures (List of registered stored procedures)
    - Triggers (List of database triggers)
    - Crons (List of scheduled jobs)
- **Observe Tab Content**: Requires logical grouping of observability tools:
    - Traces (Fetched from `system` schema observability tables)
    - Logs (Fetched from `system` schema observability tables)

## Backend Requirements
The backend must provide routes to render the HTML for each tab's content:
- `/workspace/sidebar/data` (Existing `/workspace/sidebar` can be adapted or moved here)
- `/workspace/sidebar/compute`
- `/workspace/sidebar/observe`
