# Data Model

This feature does not introduce new database tables or modify the storage engine's physical data model. Instead, it bridges the gap between the logical data model (`oxibase::core::Value`) and the scripting backends (`Rhai`, `Python`, `PL/SQL`).

## Type Mappings

### Python Mappings

| Oxibase `Value` Type | Python Native Type | Direction |
| :--- | :--- | :--- |
| `Value::Json` | `dict` or `list` | Bidirectional |
| `Value::Timestamp` | `datetime.datetime` | Bidirectional |

### PL/SQL Mappings

| Oxibase `Value` Type | PL/SQL Native Type | Direction |
| :--- | :--- | :--- |
| `Value::Json` | `JSON` | Bidirectional |
| `Value::Timestamp` | `TIMESTAMP` | Bidirectional |
