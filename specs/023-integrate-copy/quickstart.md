# Quickstart: COPY FROM Integration

## Using `COPY FROM` in Oxibase

This feature introduces a highly optimized fast-path for loading bulk data (CSV or JSON) into Oxibase, bypassing standard SQL parsing per-row.

### Loading CSV Data

```sql
-- Create a destination table
CREATE TABLE employees (id INT PRIMARY KEY, name VARCHAR, department VARCHAR);

-- Load data from a CSV file (assuming 'employees.csv' exists)
COPY employees FROM 'employees.csv' WITH (FORMAT csv, HEADER true);
```

### Loading JSON Data

The JSON importer automatically handles both JSON arrays and JSON Lines (JSONL) formats with O(1) memory footprint.

```json
// data.json
[
  {"id": 1, "name": "Alice", "role": "Engineer"},
  {"id": 2, "name": "Bob", "role": "Designer"}
]
```

```sql
-- The keys in the JSON automatically map to column names
COPY employees FROM 'data.json' WITH (FORMAT json);
```

### Advanced Usage

You can map specific columns or handle custom NULL strings.

```sql
COPY users (id, email) FROM 'users.csv' WITH (FORMAT csv, NULL 'N/A', DELIMITER '|');
```