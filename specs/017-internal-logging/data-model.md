# Data Model: Internal Logging System

## Entities

### `system.logs` Table

Stores intercepted high-severity tracing events.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique log ID |
| `timestamp` | TIMESTAMP | DEFAULT CURRENT_TIMESTAMP | When the event occurred |
| `level` | TEXT | NOT NULL | Log level (INFO, WARN, ERROR) |
| `target` | TEXT | NOT NULL | Module path / log target |
| `message` | TEXT | NOT NULL | The main log message |
| `json_fields` | TEXT | | Additional structured data serialized as JSON string |

### `LogEntry` Struct (In-Memory)

The data structure pushed across the bounded channel.

```rust
pub struct LogEntry {
    pub level: String,     // e.g., "ERROR"
    pub target: String,    // e.g., "oxibase::storage::engine"
    pub message: String,   // formatted message
    // Note: To keep things fast and allocation-low on the hot path, 
    // we format the fields immediately into the message or a pre-allocated string.
}
```
