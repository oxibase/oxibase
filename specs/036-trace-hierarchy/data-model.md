# Data Model & State Changes: Trace Hierarchy

This feature doesn't introduce any persistent data model changes, database schema alterations, or state machine transitions. 

It purely enhances the internal tracing context during runtime execution.

### Key Conceptual Entities

1. **Root API Span (`db.execute` / `db.query`)**:
   - Represents the public API boundary.
   - **Attributes**: `sql` (truncated string).
   - **Lifespan**: From method entry to method return.

2. **Network Root Span (`network.request`)**:
   - Represents an HTTP/RPC request hitting the server.
   - **Attributes**: `method`, `path`.
   - **Context**: Can inherit distributed trace IDs from headers (`traceparent`).

3. **Background Job Span (`job.execute`)**:
   - Represents a scheduled task running internally.
   - **Attributes**: `job_id`, `job_name`.
