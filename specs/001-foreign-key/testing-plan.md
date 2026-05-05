# Testing Plan: Foreign Key Constraints

## Unit Tests
- `src/parser/parser.rs`: Verify AST for `FOREIGN KEY ... REFERENCES ... ON DELETE ... ON UPDATE ...`
- `src/parser/parser.rs`: Verify `ALTER TABLE ADD CONSTRAINT` parsing

## Integration Tests
- `tests/integration/ddl.rs`: `CREATE TABLE` and `ALTER TABLE` adding foreign keys.
- `tests/integration/fk_insert.rs`: Successful insert, failed insert with non-existent FK, and NULL FK insert.
- `tests/integration/fk_actions.rs`: Update/delete with RESTRICT, CASCADE, and SET NULL actions.
- `tests/integration/fk_edge_cases.rs`: Self-referencing tables.

## Benchmarks
- `benches/fk_insert.rs`: Verify `< 15%` overhead on inserts compared to no-FK tables.