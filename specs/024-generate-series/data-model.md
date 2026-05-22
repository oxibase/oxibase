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
- **State**: Tracks current value, stop value, and step value.
- **Output**: Streams `Row` values iteratively via an iterator or loop on execution.