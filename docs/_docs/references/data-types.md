---
layout: default
title: Data Types in Oxibase
parent: References
nav_order: 3
---

# Data Types in Oxibase
{: .no_toc}

This document provides information about the data types supported in Oxibase, based on evidence from test files and implementations.

---

#### Table of Contents
{: .no-toc}

1. TOC
{:toc}

---

## Supported Data Types

Oxibase supports the following data types:

### INTEGER

64-bit signed integer values:

```sql
-- Column definition
CREATE TABLE example (
    id INTEGER PRIMARY KEY,
    count INTEGER,
    large_number INTEGER
);

-- Example values
INSERT INTO example VALUES (1, 42, 9223372036854775807);  -- Max int64
INSERT INTO example VALUES (2, -100, -9223372036854775808);  -- Min int64
```

Features:
- Full range of 64-bit integer values
- Support for PRIMARY KEY constraint
- Auto-increment support

### FLOAT

64-bit floating-point numbers:

```sql
-- Column definition
CREATE TABLE example (
    id INTEGER PRIMARY KEY,
    price FLOAT,
    temperature FLOAT
);

-- Example values
INSERT INTO example VALUES (1, 99.99, -273.15);
INSERT INTO example VALUES (2, 3.14159265359, 1.7976931348623157e+308);  -- Max float64
```

Features:
- Full range of 64-bit floating-point values
- Support for scientific notation

### TEXT

UTF-8 encoded string values:

```sql
-- Column definition
CREATE TABLE example (
    id INTEGER PRIMARY KEY,
    name TEXT,
    description TEXT
);

-- Example values
INSERT INTO example VALUES (1, 'Simple text', 'This is a longer description');
INSERT INTO example VALUES (2, 'Unicode: こんにちは', 'Special chars: !@#$%^&*()');
```

Features:
- UTF-8 encoding
- No practical length limit (constrained by available memory)
- Support for quotes and special characters

### BOOLEAN

Boolean true/false values:

```sql
-- Column definition
CREATE TABLE example (
    id INTEGER PRIMARY KEY,
    is_active BOOLEAN,
    is_deleted BOOLEAN
);

-- Example values
INSERT INTO example VALUES (1, true, false);
INSERT INTO example VALUES (2, FALSE, TRUE);  -- Case-insensitive
```

Features:
- Case-insensitive `TRUE`/`FALSE` literals
- Conversion to/from integers (1 = true, 0 = false)

### TIMESTAMP

Date and time values:

```sql
-- Column definition
CREATE TABLE example (
    id INTEGER PRIMARY KEY,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

-- Example values
INSERT INTO example VALUES (1, '2023-01-01 12:00:00', '2023-01-02T15:30:45');
INSERT INTO example VALUES (2, CURRENT_TIMESTAMP, NULL);
```

Features:
- ISO 8601 compatible format
- Support for date and time components
- `NOW()` and `CURRENT_TIMESTAMP` functions for current time
- Date and time functions (`DATE_TRUNC()`, `TIME_TRUNC()`) as shown in tests

### JSON

Oxibase implements a dedicated JSON data type:

```sql
CREATE TABLE products (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  attributes JSON
);
```

The JSON data type in Oxibase supports:

- **Objects** - Collection of key-value pairs: `{"name": "value", "name2": "value2"}`
- **Arrays** - Ordered collection of values: `[1, 2, 3, "text", true]`
- **Nested structures** - Complex combinations of objects and arrays
- **Primitive values** - Numbers, strings, booleans, and null
- **NULL constraints** - `NOT NULL` constraints can be applied to JSON columns

#### JSON Validation

Oxibase validates JSON syntax during insertion:

```sql
-- Valid JSON will be accepted
INSERT INTO products (id, name, attributes) VALUES (4, 'Valid', '{"brand":"Example"}');

-- Invalid JSON will be rejected
INSERT INTO products (id, name, attributes) VALUES (5, 'Invalid', '{brand:"Example"}');
-- Error: Invalid JSON format
```

Oxibase validates these examples of properly formatted JSON:

```
{"name":"John","age":30}
[1,2,3,4]
{"user":{"name":"John","age":30}}
[{"name":"John"},{"name":"Jane"}]
[]
{}
{"":""}
```

And these examples of invalid JSON:

```
{name:"John"}        -- Missing quotes around property name
{"name":"John"       -- Missing closing brace
{"name":"John",}     -- Trailing comma
{"name":John}        -- Missing quotes around string value
{name}               -- Invalid format
[1,2,3,}             -- Mismatched brackets
```

Features:
- Support for JSON objects and arrays
- Nested structures
- Validation of JSON syntax on insert
- Basic equality comparison
- JSON functions available (see [Scalar Functions]({% link _docs/references/functions/scalar-functions.md %}))


## NULL Values

Oxibase fully supports NULL values:

```sql
-- Column definition with nullable columns
CREATE TABLE example (
    id INTEGER PRIMARY KEY,
    name TEXT,       -- Implicitly nullable
    value INTEGER,   -- Implicitly nullable
    required TEXT NOT NULL  -- Explicitly non-nullable
);

-- Example values with NULL
INSERT INTO example (id, name, value, required) VALUES (1, NULL, NULL, 'Required');
```

Features:
- Any column can be NULL unless specifically marked as NOT NULL
- NULL handling in indexes
- IS NULL and IS NOT NULL operators
- NULL propagation in expressions
- NULL is distinct from any value, including another NULL
- More details in the dedicated [NULL Handling]({% link _docs/sql-features/null-handling.md %}) documentation

## Type Conversions

Oxibase supports type casting between compatible types:

```sql
-- Explicit CAST
SELECT CAST(42 AS TEXT);
SELECT CAST('42' AS INTEGER);
SELECT CAST('2023-01-01' AS TIMESTAMP);

-- Implicit conversion
SELECT '42' + 1;  -- Converts '42' to INTEGER
```

More details on type conversions can be found in the dedicated [CAST Operations]({% link _docs/sql-features/cast-operations.md %}) documentation.

## Examples

### Basic Types

```sql
-- Create table with all basic types
CREATE TABLE data_types_test (
    id INTEGER PRIMARY KEY,
    int_val INTEGER,
    float_val FLOAT,
    text_val TEXT,
    bool_val BOOLEAN,
    timestamp_val TIMESTAMP
);

-- Insert test values
INSERT INTO data_types_test VALUES (
    1,                    -- INTEGER
    42,                   -- INTEGER
    3.14,                 -- FLOAT
    'Hello, world!',      -- TEXT
    TRUE,                 -- BOOLEAN
    '2023-01-01 12:00:00' -- TIMESTAMP
);
```

### Timestamp Operations

```sql
-- Create table for timestamp testing
CREATE TABLE timestamp_test (
    id INTEGER PRIMARY KEY,
    event_time TIMESTAMP
);

-- Insert timestamps in different formats
INSERT INTO timestamp_test VALUES (1, '2023-05-15 14:30:45');
INSERT INTO timestamp_test VALUES (2, '2023-05-15T14:30:45');
INSERT INTO timestamp_test VALUES (3, '2023-05-15');

-- Query with time functions
SELECT id, DATE_TRUNC('day', event_time) FROM timestamp_test;
```

### JSON Data

```sql
-- Create table with JSON column
CREATE TABLE json_test (
    id INTEGER PRIMARY KEY,
    data JSON
);

-- Insert different JSON structures
INSERT INTO json_test VALUES (1, '{"name":"John","age":30}');
INSERT INTO json_test VALUES (2, '[1,2,3,4]');
INSERT INTO json_test VALUES (3, '{"user":{"name":"John","age":30}}');
```

## Data Type Storage and Performance

Based on implementation details in the code:

- INTEGER and BOOLEAN types are stored efficiently with native Rust types
- TEXT strings use UTF-8 encoding for maximum compatibility
- TIMESTAMP values are stored as Unix time with nanosecond precision
- JSON values are validated on insert but stored as string representation
- All data types support specialized compression based on data patterns

## Best Practices

- Use the most appropriate data type for your data
- Use INTEGER for IDs and counters
- Use BOOLEAN for true/false flags rather than INTEGER
- Use JSON only for genuinely structured/schemaless data
- Consider type-specific optimizations in WHERE clauses
- Use TIMESTAMP for date and time values rather than storing as TEXT
