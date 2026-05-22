# Feature Specification: integrate-copy

**Feature Branch**: `023-integrate-copy`  
**Created**: 2026-05-22  
**Status**: Draft  
**Input**: User description: "integrate the copy"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bulk Loading Data from CSV (Priority: P1)

Users need to quickly import large volumes of structured data from CSV files directly into the database without the overhead of row-by-row `INSERT` statements. This ensures fast bootstrapping of database tables and efficient data migration.

**Why this priority**: Core functionality of bulk data ingress is critical for performance and user experience when populating new databases or refreshing analytical datasets.

**Independent Test**: Can be fully tested by creating a test CSV file, creating a table, running the `COPY FROM` command, and verifying the row count and data integrity using `SELECT`.

**Acceptance Scenarios**:

1. **Given** an existing table `users` and a valid CSV file with a header row, **When** the user executes `COPY users FROM 'data.csv' WITH (FORMAT csv, HEADER true)`, **Then** all rows from the CSV are inserted into the `users` table efficiently.
2. **Given** an existing table and a CSV file, **When** the CSV contains malformed data or type mismatches for the target columns, **Then** the `COPY` operation fails with a clear type conversion error message and the transaction is rolled back.
3. **Given** an existing table with a `CHECK` constraint, **When** the user attempts to `COPY` data that violates this constraint, **Then** the operation is rejected and an error is returned.

---

### User Story 2 - Bulk Loading Data from JSON (Priority: P1)

Users need the ability to bulk ingest data formatted as JSON (either JSON arrays or JSON Lines) efficiently. This is vital for importing data from document-oriented systems or web APIs.

**Why this priority**: JSON is a ubiquitous data exchange format. Fast loading is essential for modern data workflows alongside CSV.

**Independent Test**: Can be tested independently by providing JSON and JSONL files, running the `COPY` command, and verifying insertion success and constraint enforcement via queries.

**Acceptance Scenarios**:

1. **Given** a table `events` and a file containing a JSON array of objects, **When** the user executes `COPY events FROM 'events.json' WITH (FORMAT json)`, **Then** the JSON objects are parsed and inserted into the table, mapping JSON keys to column names case-insensitively.
2. **Given** a table and a file in JSON Lines format, **When** the user executes the `COPY ... FORMAT json` command, **Then** the data is ingested correctly without requiring manual format conversion.
3. **Given** a table with vector columns, **When** a `COPY` operation imports vector data (e.g., embeddings) from a JSON file, **Then** the vector dimensions are validated against the schema before insertion.

---

### User Story 3 - Selective Column Ingestion (Priority: P2)

Users may have data files containing more columns than needed or where columns are in a different order than the target table schema. They need to specify which columns to populate during the bulk load.

**Why this priority**: Real-world data files rarely perfectly match the destination table schema. Supporting column selection makes the feature significantly more robust and usable.

**Independent Test**: Can be tested by providing a CSV/JSON file with extra fields and a `COPY` statement specifying a subset of columns, then verifying only those columns are populated and others receive defaults.

**Acceptance Scenarios**:

1. **Given** a table with columns `(id, name, age)` and a CSV, **When** the user executes `COPY users (id, name) FROM 'data.csv'`, **Then** only `id` and `name` are populated from the file, and `age` receives its default value or `NULL`.
2. **Given** a JSON file containing keys not present in the table schema, **When** a `COPY` operation without explicit columns is run, **Then** the extra keys are silently ignored and the valid keys are ingested.

---

### Edge Cases

- What happens when a user attempts a `COPY FROM` within an explicit transaction (e.g., `BEGIN; COPY ...`)? (Should be rejected to ensure atomicity of the bulk load itself without complex rollback states).
- How does the system handle missing fields in a CSV or missing keys in JSON when the target column has a `DEFAULT` expression vs. no default?
- How are string values matching the explicit `NULL` string parameter handled during type coercion?
- How are Foreign Key constraints validated during the bulk load process?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Engine MUST parse and execute the `COPY table_name [(column_list)] FROM 'file_path' [WITH (options)]` statement.
- **FR-002**: Engine MUST support `FORMAT csv` and `FORMAT json` options.
- **FR-003**: System MUST execute the `COPY` operation as a standalone auto-commit transaction and reject it if an explicit transaction is active.
- **FR-004**: System MUST perform direct parsing of fields/values to native internal types, bypassing standard per-row SQL AST parsing for performance.
- **FR-005**: System MUST enforce all existing table constraints during ingestion, including `CHECK` constraints, vector dimensionality, and Foreign Keys.
- **FR-006**: System MUST invalidate semantic and subquery caches for the affected table upon successful ingestion.
- **FR-007**: For JSON ingestion, System MUST support stream processing of both JSON arrays and JSON Lines with O(1) object memory footprint.

### Key Entities

- **CopyStatement**: AST Node representing the parsed `COPY` command.
- **CopyFormat**: Enum distinguishing between CSV and JSON formats.
- **JsonArrayStripper**: Utility stream reader that dynamically strips array brackets and commas to feed the JSON deserializer efficiently.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The database can successfully ingest data from CSV and JSON files using the `COPY` command.
- **SC-002**: The implementation successfully processes files larger than available RAM by using streaming memory-efficient readers (O(1) object memory for JSON, row-by-row for CSV).
- **SC-003**: Passes all new integration tests specific to the `COPY` functionality and does not break existing test suites (`make test-all`).
- **SC-004**: Code adheres to project standards, passing `make lint` with no warnings, avoiding new `unwrap()` calls, and proper error propagation.

## Assumptions

- The underlying file system where the database process runs has read access to the specified `file_path`.
- The storage engine supports bulk inserts or fast sequential row inserts efficiently.
- `csv` crate will be added as a dependency for CSV parsing.
