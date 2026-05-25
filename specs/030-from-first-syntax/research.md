# Research: FROM-First Syntax

## AST Representation
- **Decision**: Reuse the existing `SelectStatement` AST node to represent FROM-first queries.
- **Rationale**: The specification states that `FROM tbl SELECT ...` is semantically identical to `SELECT ... FROM tbl` (FR-004). By rewriting or mapping the FROM-first syntax directly into a `SelectStatement` with `columns` defaulting to `Expression::Star` (when SELECT is omitted), we avoid modifying the logical planner, optimizer, and execution engine entirely.
- **Alternatives considered**: Creating a new `FromFirstStatement` AST node. This was rejected because it would require cascading changes throughout the entire optimizer and executor to handle the new node type, duplicating the logic of `SelectStatement`.

## Parsing Strategy for "Any Order" Clauses
- **Decision**: Implement a flexible clause parsing loop for `FROM`-first statements. When a statement begins with `FROM`, we will consume the table expression and then enter a loop that looks for keywords (`SELECT`, `WHERE`, `GROUP BY`, `ORDER BY`, `LIMIT`, `HAVING`, etc.) and parses them in whatever order they appear.
- **Rationale**: The user clarified that clauses should be allowed in "Any order" after the initial `FROM`. The current `parse_select_statement` is strictly ordered (SELECT -> FROM -> WHERE -> GROUP BY ...). Building a new `parse_from_first_statement` function that builds up a `SelectStatement` handles this without breaking standard SQL parsing.
- **Alternatives considered**: Modifying `parse_select_statement` to support any order. This was rejected as it could make the standard `SELECT` parser too lenient and incorrectly accept malformed standard SQL queries (e.g., `SELECT * WHERE x FROM tbl`).

## Default Projection
- **Decision**: When parsing a `FROM`-first statement, if no `SELECT` clause is encountered before the end of the statement, automatically populate the `columns` field of the resulting `SelectStatement` with a single `Expression::Star`.
- **Rationale**: This fulfills the requirement that omitting `SELECT` is functionally equivalent to `SELECT *`.
