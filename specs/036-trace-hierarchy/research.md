# Research Findings: Trace Hierarchy

## Decision 1: Database API Tracing
- **Decision**: Instrument the core entry points in `src/api/database.rs` (`execute`, `query`, `execute_with_timeout`, `query_with_timeout`, `execute_named`, `query_named`) with overarching spans: `tracing::info_span!("db.execute", sql = %truncated_sql)` or `db.query`. Use the `.entered()` guard to parent synchronous child spans seamlessly.
- **Rationale**: The `Database` API is the primary application boundary. Establishing a root span here encapsulates parsing, planning, and execution under a single traceable operation.
- **Alternatives considered**: Passing a manual context object down the stack (rejected: too intrusive, doesn't utilize `tracing`'s ambient scope).

## Decision 2: Network Server Endpoints
- **Decision**: In `src/server/handlers.rs`, extract OpenTelemetry `traceparent` contexts from incoming request headers using `opentelemetry::global::get_text_map_propagator(|propagator| ...)`. Create a root `network.request` span and assign it the extracted parent context if present.
- **Rationale**: Bridges distributed systems with the database's internal trace hierarchy, making it possible to observe end-to-end request latency.
- **Alternatives considered**: Using Axum's built-in `tower_http::trace::TraceLayer` exclusively (rejected: we need explicit extraction and propagation tailored to our engine's internal tracing).

## Decision 3: Background Job Schedulers
- **Decision**: Wrap the execution in `src/executor/scheduler.rs` (`JobScheduler::execute_job`) with an overarching span `tracing::info_span!("job.execute", job_id = job_id, job_name = name).entered()`.
- **Rationale**: Ensures cron tasks or system-level maintenance operations don't mix their execution spans with user query traffic.
- **Alternatives considered**: No distinct span for jobs (rejected: makes identifying background overhead difficult).

## Decision 4: SQL String Truncation
- **Decision**: Truncate SQL queries attached to spans to a maximum of 1024 characters.
- **Rationale**: Prevents telemetry payloads from ballooning due to massive `INSERT` or batched statements.
- **Alternatives considered**: Hashing the query string (rejected: makes debugging specific errors impossible without a reverse mapping table).
