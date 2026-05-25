---
layout: default
title: INFORMATION_SCHEMA
parent: Utility Commands
grand_parent: SQL Commands
---

# INFORMATION_SCHEMA

Oxibase provides standard SQL metadata access through `information_schema` virtual tables. These tables behave like regular SQL tables and support full query capabilities including WHERE, ORDER BY, LIMIT, and joins.

#### Available Tables

| Table | Description |
|-------|-------------|
| `information_schema.tables` | Lists all tables and views |
| `information_schema.columns` | Column metadata for all tables |
| `information_schema.functions` | Available SQL functions |
| `information_schema.views` | View definitions |
| `information_schema.statistics` | Index information |
| `information_schema.sequences` | Sequence objects |
| `system.cron` | Configured job schedules |
| `system.cron_runs` | Execution history of job schedules |

#### information_schema.tables

Lists all tables and views in the database.

**Columns:**
- `table_catalog`: Always "def"
- `table_schema`: NULL (single schema database)
- `table_name`: Name of the table or view
- `table_type`: "BASE TABLE" for tables, "VIEW" for views

**Example:**
```sql
-- List all tables and views
SELECT * FROM information_schema.tables;

-- Find all views
SELECT table_name FROM information_schema.tables
WHERE table_type = 'VIEW';
```

#### information_schema.columns

Provides detailed column information for all tables.

**Columns:**
- `table_catalog`: Always "def"
- `table_schema`: NULL (single schema database)
- `table_name`: Name of the table
- `column_name`: Name of the column
- `ordinal_position`: Position in table (1-based)
- `column_default`: Default value expression
- `is_nullable`: "YES" or "NO"
- `data_type`: Data type (Integer, Text, Float, Boolean, Timestamp, Json)
- `character_maximum_length`: Max length for TEXT (65535)
- `numeric_precision`: Precision for numeric types
- `numeric_scale`: Scale for numeric types

**Example:**
```sql
-- Get column details for a table
SELECT column_name, data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_name = 'users'
ORDER BY ordinal_position;

-- Find all nullable columns
SELECT table_name, column_name
FROM information_schema.columns
WHERE is_nullable = 'YES';
```

#### information_schema.functions

Lists all available SQL functions.

**Columns:**
- `function_catalog`: Always "def"
- `function_schema`: NULL (single schema database)
- `function_name`: Name of the function
- `function_type`: "SCALAR", "AGGREGATE", or "WINDOW"
- `data_type`: Return type of the function
- `is_deterministic`: Always "true" (all built-in functions are deterministic)

**Example:**
```sql
-- List all scalar functions
SELECT function_name, data_type
FROM information_schema.functions
WHERE function_type = 'SCALAR'
ORDER BY function_name;

-- Find functions by return type
SELECT function_name, function_type
FROM information_schema.functions
WHERE data_type = 'INTEGER';
```

#### information_schema.views

Shows view definitions.

**Columns:**
- `table_catalog`: Always "def"
- `table_schema`: NULL (single schema database)
- `table_name`: Name of the view
- `view_definition`: The CREATE VIEW statement

**Example:**
```sql
-- Get all view definitions
SELECT table_name, view_definition
FROM information_schema.views;
```

#### information_schema.statistics

Provides index information for all tables.

**Columns:**
- `table_catalog`: Always "def"
- `table_schema`: NULL (single schema database)
- `table_name`: Name of the table
- `index_name`: Name of the index
- `seq_in_index`: Position in index (1-based)
- `column_name`: Column name (shows multiple rows for multi-column indexes)
- `non_unique`: "true" for non-unique indexes, "false" for unique
- `index_type`: Index type (BTREE, HASH, BITMAP)

**Example:**
```sql
-- List all indexes
SELECT table_name, index_name, column_name, non_unique
FROM information_schema.statistics
ORDER BY table_name, index_name, seq_in_index;

-- Find indexes on a specific table
SELECT index_name, column_name
FROM information_schema.statistics
WHERE table_name = 'users';
```

#### information_schema.sequences

Lists all sequences defined in the database.

**Columns:**
- `sequence_catalog`: Always "def"
- `sequence_schema`: NULL (single schema database)
- `sequence_name`: Name of the sequence
- `data_type`: The data type of the sequence (e.g. INTEGER)
- `start_value`: Starting value
- `minimum_value`: Minimum value
- `maximum_value`: Maximum value
- `increment`: Increment by value
- `cycle_option`: "YES" if cycles, "NO" otherwise

**Example:**
```sql
-- List all sequences
SELECT sequence_name, start_value, increment
FROM information_schema.sequences;
```

#### system.cron and system.cron_runs

Oxibase stores job scheduler configurations and their execution logs in the `system` schema.

**Example:**
```sql
-- View all active schedules
SELECT * FROM system.cron WHERE active = true;

-- View recent execution failures
SELECT * FROM system.cron_runs WHERE status = 'failed' ORDER BY execution_time DESC LIMIT 10;
```

#### Advanced Examples

**Find tables with specific column patterns:**
```sql
-- Tables with auto-increment columns
SELECT t.table_name, c.column_name
FROM information_schema.tables t
JOIN information_schema.columns c ON t.table_name = c.table_name
WHERE c.column_default LIKE '%AUTO_INCREMENT%';

-- Tables without primary keys
SELECT table_name
FROM information_schema.tables
WHERE table_name NOT IN (
    SELECT table_name
    FROM information_schema.columns
    WHERE column_default LIKE '%PRIMARY KEY%'
);
```

**Database schema documentation:**
```sql
-- Complete table schema with all details
SELECT
    c.table_name,
    c.column_name,
    c.data_type,
    c.is_nullable,
    c.column_default,
    CASE WHEN c.column_default LIKE '%PRIMARY KEY%' THEN 'YES' ELSE 'NO' END as is_primary_key,
    s.index_name,
    s.non_unique
FROM information_schema.columns c
LEFT JOIN information_schema.statistics s ON c.table_name = s.table_name AND c.column_name = s.column_name
WHERE c.table_name = 'users'
ORDER BY c.table_name, c.ordinal_position;
```

**Function discovery:**
```sql
-- Find all date/time functions
SELECT function_name, data_type
FROM information_schema.functions
WHERE function_name LIKE '%DATE%' OR function_name LIKE '%TIME%'
ORDER BY function_name;

-- Count functions by type
SELECT function_type, COUNT(*) as count
FROM information_schema.functions
GROUP BY function_type;
```
