---
layout: default
title: Schema Management in Oxibase
parent: SQL Commands
nav_order: 2
---

# Schema Management in Oxibase

This document covers Oxibase's schema management capabilities, including table creation, alteration, and handling of primary keys, indexes, and data types.

## Tables and Schemas

OxiBase provides standard SQL DDL (Data Definition Language) statements for managing database schemas, including tables, indexes, and user-defined functions.

### Creating Tables

Tables can be created using the standard `CREATE TABLE` syntax:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT,
    created_at TIMESTAMP,
    is_active BOOLEAN
);
```

### Table Constraints

When creating tables, you can specify the following constraints:

- **PRIMARY KEY** - Define a primary key constraint on one or more columns
- **NOT NULL** - Enforce that a column cannot contain NULL values

**Note**: For uniqueness constraints, use `CREATE UNIQUE INDEX` after table creation.

## Database Schemas

Oxibase supports database schemas as namespaces to organize database objects such as tables, views, and indexes. This allows for logical grouping and isolation of related database entities.

### Creating Schemas

Use `CREATE SCHEMA` to create a new database schema:

```sql
CREATE SCHEMA sales;
CREATE SCHEMA IF NOT EXISTS analytics;
```

The `IF NOT EXISTS` clause prevents an error if the schema already exists.

### Dropping Schemas

Use `DROP SCHEMA` to remove a database schema:

```sql
DROP SCHEMA sales;
DROP SCHEMA IF EXISTS analytics;
```

The `IF EXISTS` clause prevents an error if the schema does not exist. Note that a schema can only be dropped if it is empty (contains no objects).

### Using Schemas

Use `USE SCHEMA` to set the current schema for subsequent operations:

```sql
USE SCHEMA sales;

-- Create table in the current schema
CREATE TABLE customers (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

-- Query table in current schema
SELECT * FROM customers;

-- Reference table with explicit schema qualifier
SELECT * FROM sales.customers;
```

### Schema-Qualified Names

Tables and other objects can be referenced using schema-qualified names with the format `schema.object`:

```sql
-- Create table in specific schema
CREATE TABLE sales.orders (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER
);

-- Query across schemas
SELECT c.name, o.total
FROM sales.customers c
JOIN sales.orders o ON c.id = o.customer_id;
```

### Default Schema

For backward compatibility, all existing tables and objects belong to the default (unnamed) schema. When no schema is specified, operations default to the current schema, which starts as the default schema.

### Schema Concepts

- **Namespaces**: Schemas provide logical separation of database objects
- **Organization**: Group related tables, views, and indexes together
- **Qualification**: Use `schema.object` syntax for explicit object references
- **Current Schema**: The `USE SCHEMA` statement sets the active schema for unqualified references
- **Isolation**: Objects in different schemas are separate unless explicitly qualified

### Best Practices

- Use descriptive schema names that reflect their purpose (e.g., `sales`, `inventory`, `analytics`)
- Organize related objects into logical schemas
- Use schema-qualified names in scripts and applications for clarity
- Consider schema permissions for multi-user environments (future feature)

### Limitations

- Schema operations are DDL statements and follow transaction semantics
- The current schema setting may not persist across database connections
- Schema-qualified names are supported in most SQL statements but may have limitations in complex queries

### Altering Tables

Tables can be modified after creation using `ALTER TABLE` statements:

```sql
-- Add a new column
ALTER TABLE users ADD COLUMN last_login TIMESTAMP;

-- Drop a column
ALTER TABLE users DROP COLUMN is_active;

-- Rename a table
ALTER TABLE users RENAME TO system_users;
```

## Data Types

Oxibase supports the following data types:

### Numeric Types
- **INTEGER** - Signed integer number
- **FLOAT** - Floating-point number

### String Types
- **TEXT** - Variable-length character string

### Date and Time Types
- **TIMESTAMP** - Date and time

### Boolean Type
- **BOOLEAN** - True or false value

### Special Types
- **JSON** - JSON document

## Primary Keys

Primary keys uniquely identify rows in a table:

```sql
-- Single-column primary key
CREATE TABLE products (
    product_id INTEGER PRIMARY KEY,
    name TEXT
);

-- Composite primary key
CREATE TABLE order_items (
    order_id INTEGER,
    product_id INTEGER,
    quantity INTEGER,
    PRIMARY KEY (order_id, product_id)
);
```

## Indexes

Oxibase provides several index types for optimizing queries.

### Creating Indexes

```sql
-- Create a B-tree index
CREATE INDEX idx_user_email ON users (email);

-- Create a unique index
CREATE UNIQUE INDEX idx_unique_username ON users (username);

-- Create a multi-column index
CREATE INDEX idx_name_created ON products (name, created_at);
```

### Index Types

Oxibase supports multiple index implementations:

1. **B-tree Indexes** - For numeric and timestamp columns, supporting equality and range queries
2. **Hash Indexes** - For text and JSON columns, optimized for equality lookups
3. **Bitmap Indexes** - For boolean columns and low-cardinality data
4. **Multi-column Indexes** - For queries that filter on multiple columns together

### Dropping Indexes

```sql
DROP INDEX idx_user_email;
```

## User-Defined Functions

OxiBase supports creating custom functions using JavaScript/TypeScript:

### Creating Functions

```sql
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE BOA AS 'return arguments[0] + arguments[1];';
```

### Using Functions

```sql
SELECT add_numbers(5, 3) as sum;  -- Returns 8
```

See the [User-Defined Functions]({% link _docs/functions/user-defined-functions.md %}) documentation for detailed information about creating and using custom functions.

## Schema Information

Oxibase provides system tables and commands to query schema information:

### SHOW Commands

```sql
-- List all tables
SHOW TABLES;

-- Show table creation statement (includes structure)
SHOW CREATE TABLE users;

-- Show indexes for a table
SHOW INDEXES FROM users;
```

## Implementation Details

Under the hood, Oxibase's schema management is implemented with the following components:

- Table metadata is stored in a structured format that tracks column definitions, constraints, and indexes
- Schema changes are performed atomically, ensuring consistency
- The parser and executor collaborate to implement DDL operations
- Indexes are created in a non-blocking way when possible

## Best Practices

- Define primary keys for all tables to ensure row uniqueness
- Create indexes on columns frequently used in WHERE clauses and join conditions
- Use appropriate data types to optimize storage and query performance
- Consider using multi-column indexes for queries that filter on multiple columns
- Avoid excessive indexing, as it can impact write performance

## Limitations

- Certain ALTER TABLE operations may require significant processing time on large tables
- Currently, online schema changes for large tables may temporarily block writes
- There are limits on the number of columns and indexes per table for performance reasons