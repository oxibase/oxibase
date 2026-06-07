# Feature Specification: Workstation Sidebar Tabs

**Feature Branch**: `037-workstation-tabs`
**Created**: 2026-06-07
**Status**: Draft  
**Input**: User description: "in the workstation, i want in the left sidebar to have tree tabs to navigate to three sections: compute, data and observe. The exisiting tree of tables should be shown when data is selected. when compute is selected, the query console, functions, procedures triggers, crons should appear and in the last tab, observe, i want ot have an observability similar to the one in the image, with the traces and logs"

## Clarifications
### Session 2026-06-07
- Q: Data source for observability tab? → A: The observability tab should use the observability tables in the system schema

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Data Section Navigation (Priority: P1)

As a workstation user, I want to navigate to the "Data" tab in the left sidebar so that I can view the existing tree of tables as it currently functions.

**Why this priority**: Essential to preserve the existing functionality of exploring tables and data structures before adding new domains.

**Independent Test**: The UI should show a sidebar with a "Data" tab. Clicking it should render the existing tree view of schemas and tables.

**Acceptance Scenarios**:

1. **Given** I am in the workstation interface, **When** I click the "Data" tab in the left sidebar, **Then** I should see the tree of tables.
2. **Given** the workstation loads, **When** initialized, **Then** one of the tabs (e.g., Data) should be selected by default, displaying its content.

---

### User Story 2 - Compute Section Navigation (Priority: P1)

As a workstation user, I want to navigate to the "Compute" tab in the left sidebar so that I can access query console, functions, procedures, triggers, and crons.

**Why this priority**: Organizes active computational and scripting elements into a dedicated area, improving discoverability.

**Independent Test**: Clicking the "Compute" tab should update the sidebar to list "Query Console", "Functions", "Procedures", "Triggers", and "Crons".

**Acceptance Scenarios**:

1. **Given** I am in the workstation interface, **When** I click the "Compute" tab in the left sidebar, **Then** I should see navigation items for the query console, functions, procedures, triggers, and crons.
2. **Given** I am in the Compute section, **When** I click on "Query Console", **Then** the main view should update to display the query editor.

---

### User Story 3 - Observe Section Navigation (Priority: P2)

As a workstation user, I want to navigate to the "Observe" tab in the left sidebar so that I can access observability features including traces and logs.

**Why this priority**: Introduces a new domain for monitoring and debugging system behavior, referencing the provided visual example of a trace/timeline UI.

**Independent Test**: Clicking the "Observe" tab should display observability features.

**Acceptance Scenarios**:

1. **Given** I am in the workstation interface, **When** I click the "Observe" tab in the left sidebar, **Then** I should see options or a view for traces and logs.
2. **Given** I navigate to a trace in the Observe section, **When** it loads, **Then** it displays a timeline/span view similar to the provided reference image.

### Edge Cases

- What happens if the backend fails to load data for the currently selected tab?
- How does the UI behave on smaller screens or when the sidebar is collapsed?
- Does the system remember the last selected tab upon page reload?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The workstation left sidebar MUST include three primary navigation tabs: "Compute", "Data", and "Observe".
- **FR-002**: Selecting the "Data" tab MUST display the existing tree view of tables and schemas.
- **FR-003**: Selecting the "Compute" tab MUST display a navigational structure containing "Query Console", "Functions", "Procedures", "Triggers", and "Crons".
- **FR-004**: Selecting the "Observe" tab MUST display an observability interface including "Traces" and "Logs".
- **FR-005**: The Observe tab's trace interface MUST include a timeline/span visualization similar to standard tracing tools (as shown in the provided reference image).
- **FR-006**: The observability data (traces, logs) MUST be queried from the observability tables within the `system` schema.

### Key Entities

- **Sidebar Tab**: Represents the top-level navigation category in the workstation (Compute, Data, Observe).
- **Navigation Item**: Represents a specific resource or tool within a selected tab (e.g., "Tables" under Data, "Functions" under Compute).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can switch between the three tabs (Compute, Data, Observe) without page reloads.
- **SC-002**: Existing table tree functionality is fully preserved within the Data tab.
- **SC-003**: The Compute tab successfully routes to the query console and other compute-related views.
- **SC-004**: The Observe tab successfully renders a trace timeline visualization for system observability.

## Assumptions

- The underlying backend APIs to fetch tables, functions, procedures, triggers, crons, traces, and logs either already exist or will be developed concurrently.
- The reference image implies a trace timeline component needs to be integrated or built.
- The default selected tab on load will be either "Data" (to match current default behavior) or "Compute".
