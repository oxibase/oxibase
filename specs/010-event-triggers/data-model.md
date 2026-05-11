# Phase 1: Data Model

## SQL AST Entities (`src/parser/ast.rs`)

### CreateTriggerStatement
Represents the AST node produced when parsing a `CREATE TRIGGER` query.

*   `trigger_name`: `String`
*   `timing`: `TriggerTiming` (Enum: `Before`, `After`)
*   `event`: `TriggerEvent` (Enum: `Insert`, `Update`, `Delete`)
*   `table_name`: `TableName`
*   `for_each_row`: `bool` (Always true initially, but kept for ANSI compatibility)
*   `language`: `String` (e.g., "rhai", "js", "python")
*   `body`: `String`

### DropTriggerStatement
*   `trigger_name`: `String`
*   `table_name`: `TableName`
*   `if_exists`: `bool`

## Storage Entities (`src/storage/triggers.rs`)

### `_sys_triggers` Table Schema
Maintains the catalog definition of active triggers.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | `INTEGER` | `PRIMARY KEY AUTO_INCREMENT` | Internal ID |
| `schema` | `TEXT` | `NULL` | Database schema name |
| `name` | `TEXT` | `NOT NULL` | Trigger name |
| `table_name` | `TEXT` | `NOT NULL` | Target table |
| `timing` | `TEXT` | `NOT NULL` | BEFORE / AFTER |
| `event` | `TEXT` | `NOT NULL` | INSERT / UPDATE / DELETE |
| `for_each_row` | `BOOLEAN` | `NOT NULL` | Row vs Statement level |
| `language` | `TEXT` | `NOT NULL` | Backend script type |
| `code` | `TEXT` | `NOT NULL` | The raw script body |

## Memory Architecture (`src/executor/triggers.rs`)

### TriggerRegistry
An in-memory cache attached to the `Executor` or `Engine` to allow `O(1)` trigger lookups during DML operations without querying `_sys_triggers`.
*   A `HashMap<TableName, Vec<TriggerDefinition>>`

### Trigger Context (Thread Locals)
Located in `src/functions/backends/triggers.rs` (or similar). These thread-locals are set immediately before executing a trigger and explicitly cleared after.
```rust
thread_local! {
    pub static CURRENT_NEW_ROW: RefCell<Option<*mut Row>> = RefCell::new(None);
    pub static CURRENT_OLD_ROW: RefCell<Option<*const Row>> = RefCell::new(None);
    pub static CURRENT_SCHEMA: RefCell<Option<*const Schema>> = RefCell::new(None);
}
```
