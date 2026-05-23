# Data Model: generate_series

## AST Entity
- **Name**: `FunctionTableSource`
- **Fields**: 
  - `token`: Token (for error reporting)
  - `function`: Identifier (the function name, e.g. "generate_series")
  - `arguments`: Vec<Expression>
  - `alias`: Option<Identifier>
  - `column_aliases`: Option<Vec<Identifier>>

## Execution Entity
- **Name**: `TableFunctionExecutor` (Logical / Physical)
- **State**: Tracks current value, stop value, and step value. Must handle Integers, Floats, Dates, and Timestamps.
- **Output**: Streams `Row` values iteratively via an iterator or loop on execution.

## Scalar Function Entity
- **Name**: `GenerateSeriesScalarFunction`
- **Registration**: Registered via `src/functions/registry.rs`.
- **Output**: Returns a JSON array (or array type) when used as a scalar function (e.g. `SELECT generate_series(1, 5)`).