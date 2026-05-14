# Feature Specification: SQL Sequences

## 1. Feature Description
Native support for standard SQL Sequence objects to provide explicit, transaction-safe auto-incrementing capabilities. This feature introduces the necessary DDL commands (`CREATE`, `ALTER`, `DROP`) and sequence manipulation functions (`NEXTVAL`, `CURRVAL`, `SETVAL`), while correctly populating the existing `information_schema.sequences` view.

## 2. Business Value & Goals
- **Data Integrity:** Provides a reliable and concurrent-safe mechanism for generating unique identifiers.
- **Standards Compliance:** Aligns Oxibase with standard SQL behavior, making it easier for users migrating from other relational databases.
- **Usability:** Streamlines the creation of auto-incrementing primary keys.

## 3. User Scenarios & Use Cases
- **Scenario 1:** A database administrator creates a sequence `user_id_seq` starting at 1000 to be used as primary keys for a new `users` table.
- **Scenario 2:** A developer inserts a new record and uses `NEXTVAL('user_id_seq')` to assign a unique ID. They immediately call `CURRVAL('user_id_seq')` to retrieve that generated ID for use in a dependent table insert.
- **Scenario 3:** During a database migration or import, a user runs `SETVAL('user_id_seq', 5000)` to ensure subsequent inserts don't collide with imported legacy IDs.
- **Scenario 4:** An analytics tool queries `information_schema.sequences` to inspect all active sequences, their current values, and increments.

## 4. Functional Requirements
- **FR1 (DDL Support):** The SQL parser and executor must support `CREATE SEQUENCE [name] [options]`, `ALTER SEQUENCE [name] [options]`, and `DROP SEQUENCE [name]`.
- **FR2 (Sequence Options):** Sequence definitions must support standard options including `START WITH`, `INCREMENT BY`, `MINVALUE`, `MAXVALUE`, and `CYCLE`.
- **FR3 (Sequence Functions):** The execution engine must implement built-in functions:
  - `NEXTVAL('seq_name')`: Increments the sequence and returns the new value.
  - `CURRVAL('seq_name')`: Returns the value most recently obtained by `NEXTVAL` in the current session.
  - `SETVAL('seq_name', value, [is_called])`: Sets the sequence's current value.
- **FR4 (Concurrency):** Calls to `NEXTVAL` must be thread-safe and avoid transaction rollbacks from creating gaps, ensuring high concurrency without blocking.
- **FR5 (Catalog Integration):** Sequence metadata and state must be persisted in the catalog.
- **FR6 (Information Schema):** The `information_schema.sequences` table must correctly read from the catalog to expose the sequence definitions.

## 5. Non-Functional Requirements & Constraints
- **Performance:** Sequence incrementation must be extremely fast, utilizing atomic operations to minimize locking overhead.
- **Durability:** The current value of the sequence must be reliably journaled/persisted so that values are not reused after a crash or restart.
- **Session Isolation:** `CURRVAL` must be strictly scoped to the current user session. It should fail if `NEXTVAL` hasn't been called in the current session.

## 6. Success Criteria
- All standard DDL sequence commands parse and execute successfully.
- Calling `NEXTVAL()` from 100 concurrent connections yields exactly 100 unique, sequential IDs with zero duplicates.
- `SELECT * FROM information_schema.sequences` returns accurate metadata matching the created objects.