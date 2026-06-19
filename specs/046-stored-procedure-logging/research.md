# Research & Technical Decisions: Formal Stored Procedure Logging

## 1. Thread-Safe Dispatch via Tracing Ecosystem

- **Decision**: All procedurally-generated logs will be dispatched through standard Rust `tracing` macro events, specifically using a common `log_message` helper in `src/common/logging.rs`.
- **Rationale**:
  - Automatically correlates with any active OpenTelemetry trace/spans without passing down span references.
  - Leverages the robust, pre-existing asynchronous flusher thread (`start_log_flusher`) to write to `system.logs`, preserving memory safety, thread isolation, and non-blocking characteristics.
  - Keeps the scripting engine code decoupled from database engine transaction internals.
- **Alternatives considered**:
  - *Direct database insert inside the backend query runner*: Rejected because performing synchronous, un-buffered inserts from user execution context would degrade performance, bypass tracing context, and introduce lock-contention risks.

## 2. PL/SQL Keyword `LOG` Syntax Integration

- **Decision**: Extend the PL/SQL parser to support a native statement keyword `LOG <level_identifier>, <expression>;`.
- **Rationale**:
  - Provides a clean, elegant, SQL-standard-like procedural instruction.
  - Clear separation from `PRINT` or `RAISE NOTICE`, which serve user/output capture streams.
- **Alternatives considered**:
  - *Using built-in function call e.g., `oxibase_log()` in PL/SQL*: Rejected because PL/SQL does not natively support external namespace package lookups like `oxibase.log()`, making a native AST statement keyword much cleaner and more natural for PL/SQL developers.

## 3. Removing system.logs Whitelist in `dml.rs`

- **Decision**: Fully delete the temporary `system.logs` whitelist check in `src/executor/dml.rs`.
- **Rationale**:
  - Secures the database system catalog namespace against arbitrary direct modification.
  - Reverts the temporary workaround cleanly to ensure proper production-grade standards.
- **Alternatives considered**:
  - *Keep the whitelist as fallback*: Rejected because keeping the whitelist leaves a security gap where any user can arbitrarily insert, update, or delete system log rows, violating database integrity.
