# API Contract: Trigger Context API (oxibase.ctx)

This contract defines the public namespace injected into trigger scripting environments (Python, JS, Rhai).

## `oxibase.ctx`

The context object guarantees the availability of the following structures:

### `oxibase.ctx.old`
Represents the state of the row *before* the DML operation (UPDATE or DELETE). 
- **Type**: Dictionary/Object mapping column names to scalar values.
- **Nullability**: Will be undefined/null during `INSERT` triggers.
- **Mutability**: Strictly read-only.

### `oxibase.ctx.new`
Represents the state of the row *during or after* the DML operation (INSERT or UPDATE).
- **Type**: Dictionary/Object mapping column names to scalar values.
- **Nullability**: Will be undefined/null during `DELETE` triggers.
- **Mutability**: 
  - **BEFORE Triggers**: Mutable. Changes made to properties within this object will be intercepted by the engine and written directly to the database layer.
  - **AFTER Triggers**: Read-only. Values reflect the data that has already been persisted to disk.