# Research & Decisions: Foreign Key Constraints

## Parsing and AST
- **Decision**: Update the existing `CREATE TABLE` and `ALTER TABLE` parsers to recognize foreign key syntax. The AST will need a new `TableConstraint::ForeignKey` variant or similar.
- **Rationale**: standard SQL syntax requires parsing `FOREIGN KEY (col) REFERENCES other_table(other_col) [ON DELETE action] [ON UPDATE action]`.
- **Alternatives considered**: None, this is standard SQL.

## Schema Metadata
- **Decision**: The `Schema` or `Table` struct must store a list of foreign keys indicating the local column, the referenced table, the referenced column, and the referential actions.
- **Rationale**: The executor needs fast access to this metadata to validate inserts/updates and to trigger actions.
- **Alternatives considered**: Storing constraints in a separate system table. We will likely do both (in-memory for speed, system table for persistence).

## Constraint Validation (Immediate)
- **Decision**: Hook into the `storage` engine's `insert` and `update` methods. Before writing the new row, perform a lookup in the referenced table to ensure the foreign key value exists.
- **Rationale**: Immediate validation is required by the spec. Performing it in the storage layer ensures it's checked regardless of how the executor formulates the plan.
- **Alternatives considered**: Checking in the executor. Doing it in the storage layer is safer and centralizes the logic.

## Referential Actions (CASCADE, SET NULL)
- **Decision**: When an `update` or `delete` occurs on a *referenced* table, the storage engine must identify all *referencing* tables. It must then recursively apply the configured action (`CASCADE` -> delete/update child rows, `SET NULL` -> update child rows to null) or block the operation if `RESTRICT`.
- **Rationale**: This is the standard behavior for referential actions.
- **Alternatives considered**: Returning an error and requiring the user to manually manage it. The spec explicitly requires supporting `CASCADE` and `SET NULL`.

## Avoiding External Dependencies
- **Decision**: Implement the lookup and cascading logic using existing internal index and scan mechanisms. No new crates will be added for this feature.
- **Rationale**: Adheres to user constraints.

## Testing Strategy
- **Decision**: Add unit tests in the parser for the new syntax. Add integration tests in the `tests/` directory verifying `INSERT` rejections, `DELETE CASCADE` behavior, and `ALTER TABLE` validation.
- **Rationale**: Ensures comprehensive coverage and satisfies the requirement that all lines of code have a testing plan.