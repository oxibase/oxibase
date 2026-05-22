# Generate Series Table-Valued Function Research

## Decision: AST Representation
- **Decision**: Introduce `FunctionTableSource` in `src/parser/ast.rs`.
- **Rationale**: `generate_series` acts as a table source in the `FROM` clause. It needs an AST representation that captures the function name, arguments, and optional alias.
- **Alternatives considered**: Treating it as a scalar function call, but that wouldn't cleanly integrate with `FROM` clause parsing where table sources are expected. Note: we will *also* support it as a scalar function returning an array, matching `stoolap`.

## Decision: Parsing Logic
- **Decision**: Implement `parse_function_table_source` in `src/parser/statements.rs`.
- **Rationale**: Parses the function name and arguments like a regular function, but returns an `Expression::FunctionTableSource`.
- **Alternatives considered**: Reusing scalar function parsing, but table-valued functions allow aliases in `FROM`.

## Decision: Execution Engine Integration
- **Decision**: Implement `execute_tvf_source` in `src/executor/query.rs` and the TVF logic in `src/functions/tvf.rs` (or similar).
- **Rationale**: The executor needs to yield rows directly from the function iterator rather than reading from storage. 
- **Alternatives considered**: Creating an in-memory table first, but that violates the Zero-Copy Unikernel Efficiency principle for large ranges. An iterator-based execution is preferred.

## Decision: Core Data Structure & Types
- **Decision**: Iterate over values using an iterator yielding `Row` objects iteratively. Support Integers, Floats, Dates, and Timestamps.
- **Rationale**: Prevents massive memory allocations for large series limits. Adheres to the Constitution's memory efficiency mandate. The specific data types match the `stoolap` implementation that we are porting.