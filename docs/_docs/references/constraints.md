---
layout: default
title: Constraints
parent: References
nav_order: 4
---

# Constraints
{: .no_toc}

This document provides information about the constraints supported in Oxibase, based on evidence from test files and implementations.

---

#### Table of Contents
{: .no_toc}

1. TOC
{:toc}

---

Oxibase supports several column constraints:

### PRIMARY KEY

Uniquely identifies each row in a table:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT
);

-- With AUTO_INCREMENT
CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    product TEXT
);
```

### NOT NULL

Ensures a column cannot contain NULL values:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL
);
```

### UNIQUE

Ensures all values in a column are distinct:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT UNIQUE,
    username TEXT UNIQUE
);

-- Duplicate values will be rejected
INSERT INTO users VALUES (1, 'alice@test.com', 'alice');
INSERT INTO users VALUES (2, 'alice@test.com', 'bob');  -- Error: unique constraint failed
```

### DEFAULT

Specifies a default value when none is provided:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT DEFAULT 'Unknown',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert without specifying defaulted columns
INSERT INTO users (id) VALUES (1);
-- Result: id=1, name='Unknown', active=true, created_at=<current time>
```

Supported default values:
- Literal values: `'text'`, `123`, `3.14`, `true`, `false`
- `NULL`
- `CURRENT_TIMESTAMP` or `NOW()` for timestamps

### CHECK

Validates that values satisfy a condition (column-level constraint):

```sql
CREATE TABLE employees (
    id INTEGER PRIMARY KEY,
    age INTEGER CHECK(age >= 18 AND age <= 120),
    salary FLOAT CHECK(salary > 0),
    status TEXT CHECK(status IN ('active', 'inactive', 'pending'))
);

-- Valid insert
INSERT INTO employees VALUES (1, 25, 50000, 'active');

-- Invalid insert - fails CHECK constraint
INSERT INTO employees VALUES (2, -5, 50000, 'active');
-- Error: CHECK constraint failed for column age: (age >= 18 AND age <= 120)
```

Note: CHECK must be specified as a column constraint (inline with column definition), not as a table-level constraint.
