# Feature Specification: Implement System Metrics

**Feature Branch**: `033-implement-metrics`  
**Created**: 2026-06-06
**Status**: Draft  
**Input**: User description: "implement the metrics"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Emit and Persist Internal Metrics (Priority: P1)

As a database operator, I need the database system to automatically collect and persist internal performance metrics (like query counts, transaction durations) into the `system.metrics` table so that I can monitor the health and performance of the database.

**Why this priority**: Without a functional collection and persistence mechanism, the `system.metrics` table remains empty and useless for observability, which is a core feature for an autonomous database.

**Independent Test**: Can be fully tested by a new integration test running a query and then querying `system.metrics` to verify that rows have been inserted representing the query execution.

**Acceptance Scenarios**:

1. **Given** a running database instance, **When** a user executes a `SELECT 1` query, **Then** at least one metric related to query execution (e.g., query counter) is recorded in the `system.metrics` table.
2. **Given** a high load of metrics being emitted, **When** the background flusher processes them, **Then** the metrics are efficiently batched and inserted into the `system.metrics` table without blocking the main execution threads.

---

### User Story 2 - Query System Metrics (Priority: P2)

As a user or database operator, I need to be able to query the `system.metrics` table using standard SQL to aggregate and analyze the internal performance metrics over time.

**Why this priority**: Once metrics are collected, the primary way to consume them is via SQL queries.

**Independent Test**: Can be tested by inserting mock metrics and executing aggregation queries (e.g., `SELECT COUNT(*), metric_type FROM system.metrics GROUP BY metric_type`) and verifying the results.

**Acceptance Scenarios**:

1. **Given** the `system.metrics` table contains historical metric data, **When** a user executes `SELECT metric_type, value FROM system.metrics WHERE name = 'queries_total'`, **Then** the correct metric values are returned.

### Edge Cases

- What happens if the background flusher thread encounters an error while inserting into `system.metrics` (e.g., storage full or MVCC conflict)?
- How does the system prevent infinite recursion where the act of inserting metrics generates more metrics to be inserted? (The system must skip tracing/metrics collection for operations performed by the telemetry flusher itself).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST implement a metrics collection mechanism (e.g., via the `opentelemetry` crate or a custom layer) that captures internal events like query executions, errors, and system resource usage.
- **FR-002**: The system MUST implement a background flusher thread that receives metric events from execution threads via a channel and batches them.
- **FR-003**: The background flusher MUST insert the batched metric events into the `system.metrics` MVCC table.
- **FR-004**: The system MUST prevent the metrics flusher thread from generating recursive metrics during its own database insert operations.
- **FR-005**: The metrics data structure MUST align with the existing `CREATE_METRICS_SQL` schema (`name`, `description`, `unit`, `metric_type`, `value`, `attributes`, `timestamp`).

### Key Entities

- **[MetricEvent]**: The in-memory representation of a metric reading (counter, gauge, histogram value) before persistence.
- **[MetricsFlusher]**: The background component responsible for batching and persisting `MetricEvent`s.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Queries against `system.metrics` return non-empty, accurate data reflecting recent database activity.
- **SC-002**: The overhead of metric collection and persistence is negligible, causing < 5% performance regression in benchmark suites.
- **SC-003**: Passes `make lint` without warnings and no new `unwrap()` calls introduced in library code.
- **SC-004**: Passes all new and existing `make test` suites, particularly new integration tests verifying `system.metrics` populates correctly.

## Assumptions

- The metrics system will heavily reuse or mimic the pattern established by the tracing system (`SystemTraceLayer` and `start_trace_flusher`) to ensure consistency and prevent infinite loops via thread-local flags.
- The `opentelemetry` crate (already a dependency) is the preferred foundation for metric definitions, or a custom tracing layer will be adapted to handle metrics.
