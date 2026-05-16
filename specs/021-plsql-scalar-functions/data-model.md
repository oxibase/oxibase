# Data Model: PL/SQL Scalar Functions

This feature does not introduce new persistent database entities or tables. It primarily modifies the Abstract Syntax Tree (AST) representations and internal runtime signals.

## Entities

### `PlSqlStatement::Return` (AST Node)
- **Fields**:
  - `token`: The `Token` corresponding to the `RETURN` keyword.
  - `expression`: An `Option<Expression>` that evaluates to the returned value. `None` if it's a bare return.
- **Relationships**: Part of the `PlSqlStatement` enum, representing a return statement inside a PL/SQL block.

### `ExecutionStatus` (Interpreter Signal)
- **State Transitions**:
  - `Continue`: Normal execution flow within a block.
  - `Return(Option<Value>)`: Emitted when a `RETURN` statement is executed. Bubbles up through the interpreter's call stack to immediately halt execution of the function and yield the optional value.
