# Data Model: Fix System Tables Creation

This feature enforces explicit schema creation for the following internal tables:

1. **`system.functions`**: Enforces `id INTEGER PRIMARY KEY AUTO_INCREMENT` and `UNIQUE(schema, name)`.
2. **`system.procedures`**: Enforces `id INTEGER PRIMARY KEY AUTO_INCREMENT` and `UNIQUE(schema, name)`.
3. **`system.triggers`**: Enforces `id INTEGER PRIMARY KEY AUTO_INCREMENT` and `UNIQUE(table_schema, table_name, name)`.
4. **`system.table_stats`**: Enforces `id INTEGER PRIMARY KEY AUTO_INCREMENT` and `table_name TEXT NOT NULL UNIQUE`.
5. **`system.column_stats`**: Enforces `id INTEGER PRIMARY KEY AUTO_INCREMENT`.