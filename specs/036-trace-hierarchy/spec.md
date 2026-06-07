# Feature Specification: Trace Hierarchy Grouping

**Feature Branch**: `036-trace-hierarchy`  
**Created**: 2026-06-07  
**Status**: Draft  
**Input**: User description: "Instrument the public API boundaries and entry points of the database to ensure all internal spans are properly parented and grouped into cohesive trace hierarchies."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Database Query Execution Tracing (Priority: P1)

As a database operator or developer, I want all internal engine operations (like parsing, planning, execution) that happen during a query to be grouped under a single root trace representing that query, so I can understand the full lifecycle and performance bottleneck of an individual database query.

**Why this priority**: Core execution paths are the most frequently invoked and vital parts of the system. Getting tracing right here provides the most immediate observability benefit.

**Independent Test**: Can be fully tested by a new integration test in `tests/` verifying that when a `Database::execute` or `Database::query` is called, the internal spans (e.g., executor spans) have a non-null `parent_span_id` that links back to the root execution span.

**Acceptance Scenarios**:

1. **Given** an initialized database instance, **When** a user calls `Database::execute` with a SQL string, **Then** a root span named `db.execute` is created, it contains the SQL string as an attribute, and all internal engine spans share this span as their parent or ancestor.
2. **Given** an initialized database instance, **When** a user calls `Database::query` with a SQL string, **Then** a root span named `db.query` is created, and all internal engine spans share this span as their parent or ancestor.

---

### User Story 2 - Background Job Tracing (Priority: P2)

As a database operator, I want any internal background jobs (like scheduled tasks or maintenance) to emit traces with a clear root span, so I can distinguish background work from user queries and analyze their performance.

**Why this priority**: Background tasks can cause performance anomalies. Grouping their traces helps isolate their impact from active user queries.

**Independent Test**: Can be tested by triggering a background job (if applicable in the architecture) and asserting that the resulting spans have a root span named `job.execute` with the relevant `job_id`.

**Acceptance Scenarios**:

1. **Given** the database scheduler is active, **When** an internal background job is executed, **Then** a root span named `job.execute` is created containing the `job_id`, and all internal spans spawned by the job share this as their parent.

---

### User Story 3 - Network RPC/SQL Endpoint Tracing (Priority: P3)

As a system administrator running the database as a server, I want network requests handling remote client queries to act as the absolute root span, and to respect incoming OpenTelemetry trace context headers, so I can achieve distributed tracing across my application and the database.

**Why this priority**: While critical for distributed environments, establishing the internal API trace boundaries (P1) is a prerequisite.

**Independent Test**: Can be tested by sending a network request with OpenTelemetry headers to the server endpoint and verifying that the database execution trace is parented to the provided trace context.

**Acceptance Scenarios**:

1. **Given** the database is running as a server, **When** a remote client submits a query over the network without trace headers, **Then** an overarching `network.request` span is created and acts as the parent for `db.query`/`db.execute`.
2. **Given** the database is running as a server, **When** a remote client submits a query with OpenTelemetry `traceparent` headers, **Then** the `network.request` span (or `db.query` span) correctly links to the provided remote parent trace.

### Edge Cases

- What happens if the SQL string is extremely large (e.g., a massive `INSERT` statement)? The raw SQL attribute on the trace should be truncated or scrubbed to avoid blowing up telemetry payload sizes.
- How does the system handle tracing during database initialization or shutdown where typical boundaries might not apply?
- If a query fails early (e.g., during parsing), does the root span still correctly capture the failure and duration?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST wrap `Database::execute`, `Database::query`, and `Database::execute_with_params` methods (in `src/api/database.rs`) with spans named `db.execute` or `db.query`.
- **FR-002**: The root API spans MUST record the SQL query string as a span attribute, truncating it if it exceeds a reasonable length.
- **FR-003**: The root API spans MUST be explicitly entered (`.entered()`) or constructed such that all synchronous nested function calls (e.g., parsing, execution) automatically inherit them as parent spans.
- **FR-004**: Network request handlers (if present in `src/server/` or similar) MUST establish a `network.request` root span.
- **FR-005**: Network request handlers MUST extract OpenTelemetry trace context headers (e.g., `traceparent`) and link the internal traces to the remote parent if provided.
- **FR-006**: Background job schedulers MUST wrap individual job executions in a root span named `job.execute` with an associated `job_id` attribute.

### Key Entities *(include if feature involves data)*

- **Root API Span**: The top-level tracing span instantiated at the public interface boundary (e.g., `db.query`).
- **Internal Engine Span**: Existing tracing spans within the parser, optimizer, and executor that currently lack a parent.
- **Trace Context**: The OpenTelemetry standard headers (`traceparent`, `tracestate`) used for distributed tracing across network boundaries.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of internal engine operations executed via public APIs have a non-null `parent_span_id` linking to the respective API boundary span.
- **SC-002**: A new automated test passes, verifying the parent-child relationship of spans for a standard query execution.
- **SC-003**: SQL strings recorded in span attributes do not exceed a predefined maximum length (e.g., 1024 characters), preventing telemetry bloat.
- **SC-004**: No performance regressions > 2% overhead on standard query execution benchmarks due to the new trace instantiations.

## Assumptions

- The underlying tracing infrastructure (non-blocking, zero-copy ring buffers) is already functional and correctly captures span data.
- The project uses the `tracing` crate for instrumentation.
- Network endpoints and background jobs exist as described in the context; if not, those specific requirements will be scoped out during implementation planning.
