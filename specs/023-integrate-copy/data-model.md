# Data Model: COPY FROM Integration

## Core Entities

The core data structures involved in executing `COPY FROM` mostly reside in the AST and Executor layers.

### `CopyStatement` (Parser AST)
Represents the parsed `COPY ... FROM ...` statement.
- `table_name`: `Identifier` - The target table.
- `columns`: `Vec<Identifier>` - Optional list of columns to populate.
- `file_path`: `String` - Path to the CSV or JSON file.
- `format`: `CopyFormat` - Enum indicating `Csv` or `Json`.
- `header`: `bool` - For CSV, indicates if the first row is a header.
- `delimiter`: `u8` - For CSV, the field delimiter (e.g., `,`).
- `null_string`: `Option<String>` - The string representation of `NULL`.

### `CopyFormat` (Parser AST)
- `Csv`
- `Json`

### `JsonArrayStripper` (Executor)
A memory-efficient (O(1)) stream reader adapter that transforms a JSON array byte stream into a stream of top-level JSON objects by dynamically replacing brackets `[` `]` and commas `,` with whitespace.

## Validation Rules & Constraints

During the copy process, the executor must validate:
- **Type Coercion**: Data from strings (CSV) or JSON values must be strictly coercible to the target column type.
- **Dimensionality**: For `VECTOR` columns, the number of parsed dimensions must match the schema exactly.
- **Constraints**: `CHECK` constraints on the table schema must be evaluated against the newly constructed `Row`.
- **Foreign Keys**: The executor must verify that parent records exist for any foreign key references in the new `Row`.

## State Transitions
- **Standalone Transaction**: The executor opens a new transaction `engine.begin_transaction()`.
- **Row Insertion**: For every parsed row that passes validation, `table.insert_discard(row)` is called.
- **Commit / Rollback**: If the end of the file is reached successfully, the transaction commits. If any parse error, type mismatch, or constraint violation occurs, the transaction is rolled back completely.
- **Cache Invalidation**: On successful commit with `rows_affected > 0`, the semantic and subquery caches for `table_name` are invalidated.