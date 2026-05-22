# Research: COPY FROM Integration

## Decisions & Findings

### Parsing Strategy for CSV
- **Decision**: Use the `csv` crate to parse CSV files.
- **Rationale**: It provides a robust, fast, and standard way to read CSVs in Rust, handling quoting, delimiters, and headers automatically.
- **Alternatives considered**: Writing a custom CSV parser. Rejected due to the complexity of RFC 4180 edge cases (escaped quotes, newlines in fields) which `csv` handles perfectly.

### Parsing Strategy for JSON
- **Decision**: Use `serde_json::Deserializer::from_reader` in stream mode, backed by a custom `JsonArrayStripper`.
- **Rationale**: JSON data can be large (larger than RAM). To maintain the memory efficiency required by Oxibase's zero-copyunikernel principles (O(1) memory), we need to stream it. `serde_json` supports streaming JSON Lines natively. For JSON arrays `[...]`, the `JsonArrayStripper` dynamically transforms the bracket/comma syntax into a stream of top-level objects, allowing a single streaming deserialization path for both formats.
- **Alternatives considered**: Loading the entire JSON file into memory with `serde_json::from_reader`. Rejected because it violates the memory efficiency requirement for bulk data operations.

### Execution Path
- **Decision**: Bypass the standard row-by-row SQL parser (`INSERT INTO ...`). Parse fields directly into Oxibase `Value` types.
- **Rationale**: SQL parsing per row is expensive. The fastest way to load data is to read it, coerce it to the target column type (e.g., `parse_field` for CSV which does direct `String -> i64` without intermediate String allocation if possible), and build the `Row` directly.
- **Alternatives considered**: Generating internal `InsertStatement` AST nodes and passing them to the executor. Rejected due to unnecessary parsing overhead and intermediate allocations.

### Transaction Management
- **Decision**: `COPY FROM` runs as a standalone, auto-commit transaction. It is rejected if an explicit transaction is active.
- **Rationale**: Bulk loads generate massive amounts of MVCC versions. Keeping them in an explicit transaction could overwhelm memory or rollback logs. Auto-commit ensures the load completes efficiently or rolls back cleanly as a single unit without interaction with other statements.
- **Alternatives considered**: Allowing `COPY` within `BEGIN ... COMMIT`. Rejected due to complexity and memory pressure for large files.