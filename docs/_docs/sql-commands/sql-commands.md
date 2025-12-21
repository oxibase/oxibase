---
layout: default
title: SQL Commands
parent: SQL Commands
nav_order: 1
---

# SQL Commands

This document provides a comprehensive reference to SQL commands supported by Oxibase.

## Data Manipulation Language (DML)

### SELECT

The SELECT statement retrieves data from one or more tables.

#### Basic Syntax

```sql
SELECT [DISTINCT] column1, column2, ...
FROM table_name
[WHERE condition]
[GROUP BY column1, ... | ROLLUP(column1, ...) | CUBE(column1, ...)]
[HAVING condition]
[ORDER BY column1 [ASC|DESC] [NULLS FIRST|NULLS LAST], ...]
[LIMIT count [OFFSET offset]]
```

#### Parameters

- **DISTINCT**: Removes duplicate rows from the result
- **column1, column2, ...**: Columns to retrieve; use `*` for all columns
- **table_name**: The table to query
- **WHERE condition**: Filter condition
- **GROUP BY**: Groups rows by specified columns
- **ROLLUP/CUBE**: Multi-dimensional aggregation (see [ROLLUP and CUBE](../sql-features/rollup-cube))
- **HAVING**: Filter applied to groups
- **ORDER BY**: Sorting of results (`NULLS FIRST` or `NULLS LAST` to control NULL placement)
- **LIMIT**: Maximum rows to return
- **OFFSET**: Number of rows to skip

#### Examples

```sql
-- Basic query
SELECT id, name, price FROM products;

-- Filtering
SELECT * FROM products WHERE price > 50.00 AND category = 'Electronics';

-- Sorting
SELECT * FROM products ORDER BY price DESC, name ASC;

-- Pagination
SELECT * FROM customers LIMIT 10 OFFSET 20;

-- Unique values
SELECT DISTINCT category FROM products;

-- Aggregation
SELECT category, AVG(price) AS avg_price, COUNT(*) as count
FROM products
GROUP BY category;

-- Filtering groups
SELECT category, COUNT(*) AS product_count
FROM products
GROUP BY category
HAVING COUNT(*) > 5;
```

#### JOIN Operations

Oxibase supports all standard JOIN types:

```sql
-- INNER JOIN
SELECT p.name, c.name AS category
FROM products p
INNER JOIN categories c ON p.category_id = c.id;

-- LEFT JOIN
SELECT c.name, o.id AS order_id
FROM customers c
LEFT JOIN orders o ON c.id = o.customer_id;

-- RIGHT JOIN
SELECT c.name, o.id AS order_id
FROM customers c
RIGHT JOIN orders o ON c.id = o.customer_id;

-- FULL OUTER JOIN
SELECT c.name, o.id
FROM customers c
FULL OUTER JOIN orders o ON c.id = o.customer_id;

-- CROSS JOIN
SELECT p.name, c.name
FROM products p
CROSS JOIN colors c;
```

See [JOIN Operations](../sql-features/join-operations) for detailed documentation.

#### Subqueries

Using subqueries in various clauses (both correlated and non-correlated):

```sql
-- Scalar subquery
SELECT name, price,
       (SELECT AVG(price) FROM products) as avg_price
FROM products;

-- IN subquery
SELECT * FROM customers
WHERE id IN (SELECT DISTINCT customer_id FROM orders);

-- EXISTS subquery (correlated)
SELECT * FROM customers c
WHERE EXISTS (SELECT 1 FROM orders o WHERE o.customer_id = c.id);

-- NOT IN subquery
SELECT * FROM products
WHERE id NOT IN (SELECT product_id FROM discontinued_items);

-- Correlated subquery in WHERE
SELECT * FROM employees e1
WHERE salary > (SELECT AVG(salary) FROM employees e2 WHERE e2.department = e1.department);

-- ANY/ALL subquery
SELECT * FROM products WHERE price > ALL (SELECT price FROM products WHERE category = 'Books');

-- Derived table (subquery in FROM)
SELECT * FROM (SELECT id, name FROM products WHERE price > 100) AS expensive;
```

See [Subqueries](../sql-features/subqueries) for detailed documentation.

#### Common Table Expressions (CTEs)

```sql
-- Simple CTE
WITH high_value_orders AS (
    SELECT * FROM orders WHERE total > 1000
)
SELECT * FROM high_value_orders;

-- Multiple CTEs
WITH
customer_totals AS (
    SELECT customer_id, SUM(total) as total_spent
    FROM orders
    GROUP BY customer_id
),
vip_customers AS (
    SELECT * FROM customer_totals WHERE total_spent > 10000
)
SELECT c.name, ct.total_spent
FROM customers c
JOIN vip_customers ct ON c.id = ct.customer_id;

-- Recursive CTE
WITH RECURSIVE numbers AS (
    SELECT 1 as n
    UNION ALL
    SELECT n + 1 FROM numbers WHERE n < 10
)
SELECT * FROM numbers;
```

See [Common Table Expressions](../sql-features/common-table-expressions) for detailed documentation.

#### Set Operations

```sql
-- UNION (removes duplicates)
SELECT name FROM customers
UNION
SELECT name FROM suppliers;

-- UNION ALL (keeps duplicates)
SELECT name FROM customers
UNION ALL
SELECT name FROM suppliers;

-- INTERSECT
SELECT id FROM table1
INTERSECT
SELECT id FROM table2;

-- EXCEPT
SELECT id FROM table1
EXCEPT
SELECT id FROM table2;
```

#### Temporal Queries (AS OF)

Query historical data at a specific point in time:

```sql
-- Query data as of a specific timestamp
SELECT * FROM orders AS OF TIMESTAMP '2024-01-15 10:30:00';

-- Query data as of current time
SELECT * FROM inventory AS OF TIMESTAMP NOW();
```

See [Temporal Queries](../sql-features/temporal-queries) for detailed documentation.

### INSERT

The INSERT statement adds new rows to a table.

#### Basic Syntax

```sql
-- Single row
INSERT INTO table_name [(column1, column2, ...)]
VALUES (value1, value2, ...)
[RETURNING *|column1, column2, ...];

-- Multiple rows
INSERT INTO table_name [(column1, column2, ...)]
VALUES
  (value1_1, value1_2, ...),
  (value2_1, value2_2, ...);

-- With ON DUPLICATE KEY UPDATE
INSERT INTO table_name [(column1, column2, ...)]
VALUES (value1, value2, ...)
ON DUPLICATE KEY UPDATE
  column1 = new_value1,
  column2 = new_value2;
```

#### Examples

```sql
-- Basic insertion
INSERT INTO customers (id, name, email)
VALUES (1, 'John Doe', 'john@example.com');

-- Multiple rows
INSERT INTO products (id, name, price) VALUES
(1, 'Laptop', 1200.00),
(2, 'Smartphone', 800.00),
(3, 'Tablet', 500.00);

-- With RETURNING clause
INSERT INTO users (name, email)
VALUES ('Alice', 'alice@example.com')
RETURNING id, name;

-- Upsert with ON DUPLICATE KEY UPDATE
INSERT INTO inventory (product_id, quantity)
VALUES (101, 50)
ON DUPLICATE KEY UPDATE
  quantity = quantity + 50;
```

See [ON DUPLICATE KEY UPDATE](../sql-features/on-duplicate-key-update) for detailed documentation.

### UPDATE

The UPDATE statement modifies existing data.

#### Basic Syntax

```sql
UPDATE table_name
SET column1 = value1, column2 = value2, ...
[WHERE condition]
[RETURNING *|column1, column2, ...];
```

#### Examples

```sql
-- Update single row
UPDATE customers
SET email = 'new.email@example.com'
WHERE id = 1;

-- Update multiple rows
UPDATE products
SET price = price * 1.1
WHERE category = 'Electronics';

-- Update all rows
UPDATE settings
SET last_updated = NOW();

-- With RETURNING clause
UPDATE accounts
SET balance = balance + 100
WHERE id = 1
RETURNING id, balance;

-- Update using subquery
UPDATE products
SET discount = 0.15
WHERE category IN (
    SELECT name FROM categories WHERE is_premium = true
);
```

### DELETE

The DELETE statement removes rows from a table.

#### Basic Syntax

```sql
DELETE FROM table_name
[WHERE condition]
[RETURNING *|column1, column2, ...];
```

#### Examples

```sql
-- Delete single row
DELETE FROM customers WHERE id = 1;

-- Delete multiple rows
DELETE FROM orders WHERE order_date < '2023-01-01';

-- Delete all rows
DELETE FROM temporary_logs;

-- With RETURNING clause
DELETE FROM users WHERE inactive = true
RETURNING id, name;

-- Delete using subquery
DELETE FROM orders
WHERE customer_id IN (
    SELECT id FROM customers WHERE status = 'inactive'
);
```

### TRUNCATE

The TRUNCATE statement removes all rows from a table efficiently.

#### Basic Syntax

```sql
TRUNCATE TABLE table_name;
```

#### Example

```sql
-- Remove all rows (faster than DELETE)
TRUNCATE TABLE logs;
```

Note: TRUNCATE is faster than DELETE for removing all rows because it doesn't log individual row deletions.

## Data Definition Language (DDL)

### CREATE TABLE

Creates a new table.

#### Basic Syntax

```sql
CREATE TABLE [IF NOT EXISTS] table_name (
    column_name data_type [constraints...],
    column_name data_type [constraints...],
    ...
);
```

#### Data Types

| Type | Description |
|------|-------------|
| INTEGER | 64-bit signed integer |
| FLOAT | 64-bit floating point |
| TEXT | Variable-length string |
| BOOLEAN | true/false |
| TIMESTAMP | Date and time |
| JSON | JSON data |

#### Column Constraints

| Constraint | Description |
|------------|-------------|
| PRIMARY KEY | Unique identifier, cannot be NULL |
| NOT NULL | Column cannot contain NULL values |
| AUTO_INCREMENT | Automatically generates sequential values |

#### Examples

```sql
-- Basic table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT,
    age INTEGER,
    created_at TIMESTAMP
);

-- With AUTO_INCREMENT
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    title TEXT NOT NULL,
    content TEXT,
    author_id INTEGER
);

-- With IF NOT EXISTS
CREATE TABLE IF NOT EXISTS products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price FLOAT NOT NULL
);
```

### CREATE TABLE AS SELECT

Creates a new table from the result of a SELECT query.

#### Basic Syntax

```sql
CREATE TABLE table_name AS SELECT ...;
```

#### Example

```sql
-- Create table from query result
CREATE TABLE active_users AS
SELECT id, name, email
FROM users
WHERE last_login > '2024-01-01';

-- Create summary table
CREATE TABLE daily_sales AS
SELECT DATE_TRUNC('day', order_date) as day,
       SUM(amount) as total
FROM orders
GROUP BY DATE_TRUNC('day', order_date);
```

### ALTER TABLE

Modifies an existing table.

#### Basic Syntax

```sql
ALTER TABLE table_name operation;
```

#### Operations

```sql
-- Add a column
ALTER TABLE users ADD COLUMN last_login TIMESTAMP;

-- Drop a column
ALTER TABLE users DROP COLUMN age;

-- Rename a column
ALTER TABLE users RENAME COLUMN username TO user_name;

-- Rename table
ALTER TABLE users RENAME TO customers;
```

### DROP TABLE

Removes a table and all its data.

#### Basic Syntax

```sql
DROP TABLE [IF EXISTS] table_name;
```

#### Examples

```sql
DROP TABLE temporary_data;
DROP TABLE IF EXISTS old_logs;
```

### CREATE VIEW

Creates a virtual table based on a SELECT statement.

#### Basic Syntax

```sql
CREATE VIEW view_name AS SELECT ...;
```

#### Examples

```sql
-- Simple view
CREATE VIEW active_products AS
SELECT * FROM products WHERE in_stock = true;

-- View with joins
CREATE VIEW order_details AS
SELECT o.id, c.name as customer, p.name as product, o.quantity
FROM orders o
JOIN customers c ON o.customer_id = c.id
JOIN products p ON o.product_id = p.id;

-- Query the view
SELECT * FROM active_products WHERE price > 100;
```

### DROP VIEW

Removes a view.

#### Basic Syntax

```sql
DROP VIEW [IF EXISTS] view_name;
```

#### Example

```sql
DROP VIEW active_products;
DROP VIEW IF EXISTS old_report;
```

### CREATE FUNCTION

Creates a user-defined function using JavaScript/TypeScript.

#### Basic Syntax

```sql
CREATE FUNCTION [IF NOT EXISTS] function_name (
    param1 data_type,
    param2 data_type,
    ...
)
RETURNS return_type
LANGUAGE DENO AS 'JavaScript code';
```

#### Parameters

- **function_name**: Name of the function
- **param1, param2, ...**: Parameter names and their data types
- **return_type**: Return value data type
- **LANGUAGE DENO**: Specifies JavaScript/TypeScript implementation
- **AS 'code'**: The function implementation code

#### Supported Data Types

- `INTEGER` - 64-bit signed integers
- `FLOAT` - 64-bit floating-point numbers
- `TEXT` - UTF-8 text strings
- `BOOLEAN` - True/false values
- `TIMESTAMP` - Date and time values
- `JSON` - JSON documents

#### Examples

```sql
-- Simple arithmetic function
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE DENO AS 'return arguments[0] + arguments[1];';

-- String manipulation
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE DENO AS 'return `Hello, ${arguments[0]}!`;';

-- Date processing
CREATE FUNCTION format_date(ts TIMESTAMP)
RETURNS TEXT
LANGUAGE DENO AS '
    const date = new Date(arguments[0]);
    return date.toLocaleDateString();
';

-- JSON processing
CREATE FUNCTION extract_field(json_doc JSON, field TEXT)
RETURNS TEXT
LANGUAGE DENO AS '
    const doc = JSON.parse(arguments[0]);
    return doc[arguments[1]] || null;
';
```

See [User-Defined Functions](../functions/user-defined-functions) for detailed documentation.

### CREATE INDEX

Creates an index on table columns for faster queries.

#### Basic Syntax

```sql
CREATE [UNIQUE] INDEX [IF NOT EXISTS] index_name
ON table_name (column_name [, column_name...]);
```

#### Index Type Selection

Oxibase automatically selects the optimal index type based on column data type:

| Data Type | Index Type | Best For |
|-----------|------------|----------|
| INTEGER, FLOAT, TIMESTAMP | B-tree | Range queries, equality, sorting |
| TEXT, JSON | Hash | Equality lookups, IN clauses |
| BOOLEAN | Bitmap | Low-cardinality columns |

#### Examples

```sql
-- Single-column index
CREATE INDEX idx_user_email ON users (email);

-- Multi-column index
CREATE INDEX idx_order_customer_date ON orders (customer_id, order_date);

-- Unique index
CREATE UNIQUE INDEX idx_unique_email ON users (email);

-- With IF NOT EXISTS
CREATE INDEX IF NOT EXISTS idx_name ON products (name);
```

See [Indexing](../architecture/indexing) for detailed documentation.

### DROP INDEX

Removes an index from a table.

#### Basic Syntax

```sql
DROP INDEX [IF EXISTS] index_name ON table_name;
```

#### Example

```sql
DROP INDEX idx_user_email ON users;
DROP INDEX IF EXISTS idx_old ON products;
```

## Transaction Control

### BEGIN TRANSACTION

Starts a new transaction.

```sql
BEGIN TRANSACTION;
-- or simply
BEGIN;
```

### COMMIT

Commits the current transaction, making all changes permanent.

```sql
COMMIT;
```

### ROLLBACK

Rolls back the current transaction, discarding all changes.

```sql
ROLLBACK;
```

### SAVEPOINT

Creates a savepoint within a transaction for partial rollback.

```sql
-- Create a savepoint
SAVEPOINT savepoint_name;

-- Rollback to a savepoint
ROLLBACK TO SAVEPOINT savepoint_name;

-- Release a savepoint
RELEASE SAVEPOINT savepoint_name;
```

#### Example

```sql
BEGIN TRANSACTION;

INSERT INTO accounts (id, balance) VALUES (1, 1000);
SAVEPOINT after_insert;

UPDATE accounts SET balance = 500 WHERE id = 1;
-- Oops, wrong update
ROLLBACK TO SAVEPOINT after_insert;

-- Continue with correct update
UPDATE accounts SET balance = 900 WHERE id = 1;
COMMIT;
```

See [Savepoints](../sql-features/savepoints) for detailed documentation.

## Query Analysis

### EXPLAIN

Shows the query execution plan.

```sql
EXPLAIN SELECT * FROM users WHERE id = 1;
```

Output:
```
plan
----
SELECT
  Columns: *
  -> PK Lookup on users
       id = 1
```

### EXPLAIN ANALYZE

Shows the execution plan with actual runtime statistics.

```sql
EXPLAIN ANALYZE SELECT * FROM products WHERE price > 100;
```

Output:
```
plan
----
SELECT (actual time=1.2ms, rows=150)
  Columns: *
  -> Seq Scan on products (actual rows=150)
       Filter: (price > 100)
```

See [EXPLAIN](../sql-features/explain) for detailed documentation.

### ANALYZE

Collects statistics for the query optimizer.

```sql
-- Analyze a specific table
ANALYZE table_name;
```

Statistics are used by the cost-based optimizer to choose efficient query plans.

## Utility Commands

### SHOW TABLES

Lists all tables in the database.

```sql
SHOW TABLES;
```

### SHOW INDEXES

Lists all indexes for a table.

```sql
SHOW INDEXES FROM table_name;
```

### SHOW CREATE TABLE

Shows the CREATE TABLE statement for a table.

```sql
SHOW CREATE TABLE table_name;
```

### SHOW FUNCTIONS

Lists all available SQL functions (scalar, aggregate, and window functions).

#### Basic Syntax

```sql
SHOW FUNCTIONS;
SHOW FUNCTION;  -- Also accepted
```

#### Example

```sql
SHOW FUNCTIONS;
```

Output:
```
name       type
---------  ---------
UPPER      SCALAR
COUNT      AGGREGATE
ROW_NUMBER WINDOW
...
```

### INFORMATION_SCHEMA

Oxibase provides standard SQL metadata access through `information_schema` virtual tables. These tables behave like regular SQL tables and support full query capabilities including WHERE, ORDER BY, LIMIT, and joins.

#### Available Tables

| Table | Description |
|-------|-------------|
| `information_schema.tables` | Lists all tables and views |
| `information_schema.columns` | Column metadata for all tables |
| `information_schema.functions` | Available SQL functions |
| `information_schema.views` | View definitions |
| `information_schema.statistics` | Index information |
| `information_schema.sequences` | Sequence objects (empty in current version) |

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

Reserved for future sequence support. Currently returns no rows.

**Example:**
```sql
SELECT * FROM information_schema.sequences;
-- Returns empty result set
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

### PRAGMA

Sets or gets configuration options.

```sql
-- Set a value
PRAGMA name = value;

-- Get current value
PRAGMA name;
```

#### Supported PRAGMAs

| PRAGMA | Description | Default |
|--------|-------------|---------|
| sync_mode | WAL sync mode (0=None, 1=Normal, 2=Full) | 1 |
| snapshot_interval | Snapshot interval in seconds | 300 |
| keep_snapshots | Number of snapshots to retain | 3 |
| wal_flush_trigger | Operations before WAL flush | 1000 |
| create_snapshot | Manually create a snapshot | - |

See [PRAGMA Commands](pragma-commands) for detailed documentation.

## Parameter Binding

Use `$N` positional placeholders for parameter binding:

```sql
SELECT * FROM users WHERE id = $1;
INSERT INTO products (name, price) VALUES ($1, $2);
UPDATE orders SET status = $1 WHERE id = $2;
```

See [Parameter Binding](../sql-features/parameter-binding) for detailed documentation.

## Notes

1. **Transactions**: Oxibase provides MVCC-based transactions for concurrent operations
2. **NULL Handling**: Follows standard SQL NULL semantics; use IS NULL or IS NOT NULL for testing
3. **Type Conversion**: Explicit CAST is recommended for clarity
4. **Case Sensitivity**: SQL keywords are case-insensitive; identifiers are case-sensitive
