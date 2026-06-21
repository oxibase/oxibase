# Feature Specification: Observability Dashboard and UI Overhaul (Logs & Traces)

**Feature Branch**: `051-observe-ui-overhaul`  
**Created**: June 20, 2026  
**Status**: Draft  
**Input**: User description: "add to 0.6 a better tracing and logging UI/UX ? something more similar to grafana or other UIs.. as the actual experience is poor for debugging due to the pretty limiting list of 100 lines, no filter, no hierchy visualization etc.. (1. no dark theme, 2. auto-refresh, 3. infinite scroll)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Integrated Loki-style Logs Explorer (Priority: P1)

Developers and operators need a centralized dashboard to stream, filter, and inspect database system logs without being restricted by flat database table structures or hardcoded limits.

**Why this priority**: Logs are the primary entry point for error identification and troubleshooting. Adding live filtering and infinite scroll makes diagnostics painless.

**Independent Test**: Verified via UI integration routing tests and manual validation in Workspace.

**Acceptance Scenarios**:
1. **Given** the Workspace logs view (`/workspace/observe/logs`), **When** selecting log levels (e.g., `ERROR`), **Then** only logs with that severity must be listed in descending order of timestamp.
2. **Given** the search bar, **When** entering keywords (e.g., `"SELECT"`), **Then** results must filter matching text inside message/target fields in real time using safe parameterized database queries.
3. **Given** a long log stream, **When** scrolling to the bottom, **Then** additional log records must dynamically load (infinite scroll) rather than stopping at a hardcoded 100-line limit.
4. **Given** a log entry associated with a transaction, **When** it contains a non-empty `trace_id`, **Then** a "🔍 View Trace" badge must appear, linking directly to the specific trace timeline view.

---

### User Story 2 - Tempo-style Hierarchical Trace Timeline (Priority: P1)

Database developers need a clear visual breakdown of procedural executions and transaction spans showing nesting hierarchy, start-offsets, durations, and bottlenecks.

**Why this priority**: Flat lists of spans are impossible to decode for deeply nested query calls and procedure triggers. Visualizing execution nesting is vital.

**Independent Test**: Verified via trace-grouping tests and Gantt tree hierarchy validation.

**Acceptance Scenarios**:
1. **Given** the Trace summary list (`/workspace/observe/traces`), **When** rendering, **Then** it must group individual spans by `trace_id` to present unique transaction executions (showing root operation name, duration, span count, and timestamp) sorted by newest first.
2. **Given** a trace contains a span with status code `ERROR`, **When** displaying the list or detail view, **Then** a prominent red indicator must flag the entire trace as failed.
3. **Given** a trace timeline detail view (`/workspace/traces/{trace_id}`), **When** loaded, **Then** spans must be ordered in a recursive tree based on parent-child ID relations with collapsible indentation tree nodes.
4. **Given** any span is clicked inside the Gantt chart, **When** selected, **Then** an interactive side drawer must slide out containing exact duration, start/end timestamps, fully formatted JSON attributes, and a quick link to filter associated logs for that specific span.

---

### User Story 3 - Observability Control Center (Priority: P2)

Operators need a central control point to toggle active polling for live diagnostic streaming of traces and logs.

**Why this priority**: Simplifies monitoring active workloads or debugging concurrent procedures without manual refresh cycles.

**Independent Test**: Verified via dynamic template polling tests.

**Acceptance Scenarios**:
1. **Given** the logs or traces dashboard, **When** activating the "Auto-Refresh" toggle, **Then** the page contents must periodically reload automatically at a short configurable interval (e.g., every 5 seconds) using Unpoly-based AJAX polling.

---

### Edge Cases

- **Orphaned Spans**: If a trace contains spans whose `parent_span_id` does not refer to any span in the trace, those spans must fall back to being treated as root spans (rendered at depth 0) rather than failing the tree rendering.
- **Trace logs without traces**: If logs contain a `trace_id` that was pruned or is no longer present in `system.traces`, clicking "View Trace" should degrade gracefully, informing the user that the trace metrics are not available.
- **Malformed Attribute/JSON Payload**: If a span's attributes or a log's `json_fields` contains invalid JSON or nulls, the UI must fallback to displaying raw text or a placeholder rather than breaking the UI thread.
- **Empty Datasets**: If `system.logs` or `system.traces` contains zero records, the UI must display a beautiful, empty-state placeholder with helpful setup guidance.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support dedicated GET routes for `/workspace/observe/logs` and `/workspace/observe/traces` returning rich interactive Dashboards.
- **FR-002**: The Log Explorer MUST query `system.logs` using secure parameterized filters: `level`, `search` (matching message or target), `trace_id`, and a dynamically requested `limit`.
- **FR-003**: The Log Explorer MUST implement infinite scroll loading, using sequential page/limit increments or offset fetches when the user reaches the bottom.
- **FR-004**: The Trace Summary list MUST execute trace-level aggregations (`GROUP BY trace_id`) to show distinct transactions, summarising total duration, span counts, overall error state, and root span name.
- **FR-005**: The Trace Detail View MUST implement a recursive tree-generation algorithm to visually represent span hierarchies with expand/collapse nodes.
- **FR-006**: Spans in the Gantt timeline MUST render relative timing offsets and widths calculated dynamically against the total trace duration.
- **FR-007**: The Trace Detail View MUST support an interactive sidebar drawer showing pretty-printed key-value attributes and events JSON.
- **FR-008**: The Observe dashboards MUST support a user-configurable "Auto-Refresh" toggle using `up-poll` or equivalent AJAX-polling triggers.

### Key Entities

- **[system.logs]**: System table containing server and procedure log lines with trace correlation.
- **[system.traces]**: System table containing OpenTelemetry spans.
- **[LogsExplorer]**: Custom Jinja template rendering rich log controls, level filter badges, live search, and collapsible JSON context drawer.
- **[TracesExplorer]**: Custom Jinja template rendering the aggregated list of traces and transaction stats.
- **[TraceTimelineTree]**: Front-end recursive span visualizer generating the collapsing Gantt layout and side details drawer.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of all compile and integration tests pass successfully.
- **SC-002**: Real-time log filtration completes query execution and render within 100 milliseconds for datasets up to 10,000 logs.
- **SC-003**: Gantt hierarchical trees render accurate nestings up to 10 levels deep without UI freeze or recursion limits.
- **SC-004**: Clicking trace-to-log and log-to-trace links transitions states with full target filters applied in under 150 milliseconds.
- **SC-005**: `make lint` returns zero clippy or format warnings.

---

## Assumptions

- We assume standard timestamps are available and indexed for efficient ordering.
- We assume DaisyUI 5 and Tailwind CSS 4 classes are sufficient to style highly responsive APM widgets.
- We assume Unpoly's DOM morphing is utilized to ensure seamless partial page refreshes for auto-polling.
