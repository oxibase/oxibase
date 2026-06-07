# Feature Specification: Telemetry Ring Buffer Table

**Feature Branch**: `035-telemetry-ring-buffer`  
**Created**: 2026-06-07
**Status**: Draft  
**Input**: User description: "Research and diagnose performance bottlenecks in the database's internal telemetry system (tracing, logging, and metrics)"

## Clarifications

### Session 2026-06-07
- Q: Should we include user-facing SQL syntax (`CREATE UNLOGGED TABLE`) in this feature, or build the internal ring buffer for telemetry first and expose to users later? → A: Build the internal ring buffer for telemetry first; expose to users in a separate feature.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Non-blocking Telemetry Insertion (Priority: P1)

As a database operator, I want internal telemetry (traces, logs, metrics) to be inserted without contending for MVCC locks, transactional overhead, or the WAL, so that user query performance is not degraded.

**Why this priority**: Restoring zero-copy unikernel efficiency and unblocking the hot path is the core goal of this optimization.

**Independent Test**: Can be tested via a benchmark or integration test that emits a high volume of telemetry and verifies that transaction throughput and WAL write latency remain stable compared to when telemetry is disabled.

**Acceptance Scenarios**:

1. **Given** a high-throughput workload, **When** telemetry is enabled, **Then** the database inserts telemetry records without writing them to the main WAL.
2. **Given** telemetry ingestion, **When** a span or log is recorded, **Then** the `SystemRingBufferTable` stores it without checking MVCC visibility or allocating transactional memory.

---

### User Story 2 - Memory-Bounded Telemetry Storage (Priority: P1)

As a database operator, I want the internal system tables to retain a fixed amount of recent telemetry, so that memory usage remains strictly bounded and predictable.

**Why this priority**: Preventing unbounded memory growth is essential for stability. The ring buffer must cap the number of records.

**Independent Test**: Can be tested via an integration test that inserts records exceeding the capacity and verifies that older records are evicted (count remains at capacity).

**Acceptance Scenarios**:

1. **Given** a `SystemRingBufferTable` with a capacity of 100,000, **When** the 100,001st telemetry record is inserted, **Then** the oldest record is evicted, and the total row count remains 100,000.
2. **Given** a query `SELECT COUNT(*) FROM system.traces`, **When** the ring buffer is saturated, **Then** it returns the configured capacity.

---

### User Story 3 - Telemetry Querying via Standard SQL (Priority: P2)

As a database administrator, I want to query the system tables (`system.logs`, `system.traces`, `system.metrics`) using standard SQL `SELECT` queries to diagnose issues.

**Why this priority**: Operators still need to access the telemetry data to understand system behavior.

**Independent Test**: Can be tested via integration tests that execute `SELECT * FROM system.logs WHERE level = 'ERROR'` and verify correct results.

**Acceptance Scenarios**:

1. **Given** records in the `SystemRingBufferTable`, **When** a user executes a `SELECT` query, **Then** the table returns an iterator over the ring buffer contents.
2. **Given** a query with a `WHERE` clause, **When** executed against the telemetry tables, **Then** it correctly filters the ring buffer rows.

### Edge Cases

- What happens if the background flusher threads fall behind? (Channel saturation)
- How does the system handle schema changes to system tables? (Should be disallowed or restricted).
- How are telemetry records reconstructed into standard `Row` format for SQL querying?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `SystemRingBufferTable` implementation of the `Table` trait.
- **FR-002**: `SystemRingBufferTable` MUST store rows in a memory-bounded structure (e.g., `RwLock<VecDeque<Row>>`).
- **FR-003**: `SystemRingBufferTable.insert()` MUST NOT interact with the `VersionStore` or `TransactionRegistry`.
- **FR-004**: `SystemRingBufferTable.get_pending_versions()` MUST return an empty list to bypass the WAL.
- **FR-005**: Engine MUST instantiate `SystemRingBufferTable` instead of a standard MVCC table for tables matching `system.*` (or specifically logs, metrics, traces).
- **FR-006**: Telemetry channels MUST support high-throughput, low-contention ingestion (e.g., passing unformatted primitives to defer JSON serialization).
- **FR-007**: Telemetry ingestion MUST NOT perform blocking `send` operations on the hot path.

### Key Entities

- **`SystemRingBufferTable`**: A new struct implementing the `Table` trait, acting as a non-MVCC, non-durable storage backend for system tables.
- **`Row`**: The standard representation of a database row, which the ring buffer will store.
- **`SpanEvent`, `LogEntry`, `MetricEvent`**: Intermediate structs used to pass telemetry data from the hot path to the flusher threads.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Telemetry insertion overhead is reduced to <1% of total transaction execution time under load.
- **SC-002**: The `system.*` tables cap memory usage at the configured ring buffer capacity.
- **SC-003**: No WAL writes are generated by internal telemetry insertions.
- **SC-004**: Passes all new and existing `make test` suites, specifically ensuring telemetry queries still function.

## Assumptions

- It is acceptable to lose the most recent telemetry records in the event of a hard crash (no WAL durability).
- Telemetry tables do not require indexes, update, or delete capabilities (append and select only).
- The schema for system tables is fixed and well-known.