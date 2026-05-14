---
title: "Stored Procedures in Python"
layout: default
parent: "Stored Procedures"
has_children: true
nav_order: 4
---

# Stored Procedures in Python

Oxibase integrates [RustPython](https://rustpython.github.io/), a Python 3 interpreter written in Rust. This enables developers and data scientists to write database logic using standard Python 3 syntax.

> **Feature Flag Required**: To use Python procedures, Oxibase must be compiled with the `python` feature flag enabled.

## Basic Usage

When defining a Python procedure, use `LANGUAGE python`. Since Python relies on indentation, Oxibase automatically wraps your code string into an executable context. However, it's best practice to keep your indentation clean.

Arguments (including `OUT` and `INOUT` parameters) are available in the script's global scope.

```sql
CREATE PROCEDURE concat_py(a TEXT, b TEXT, OUT res TEXT) 
LANGUAGE python 
AS '
res = a + " " + b
';
```

## Executing SQL Commands

Python stored procedures have access to the main database engine via the `oxibase` module. You can execute standard SQL queries natively.

To use the module, simply `import oxibase`. The `oxibase.execute(query)` function returns the number of rows affected by the statement.

```sql
CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    message TEXT
);

CREATE PROCEDURE log_event_py(msg TEXT) 
LANGUAGE python 
AS '
import oxibase

# We can use Python f-strings or format for string interpolation
query = f"INSERT INTO audit_logs (message) VALUES (''{msg}'')"

# Execute the query
rows_affected = oxibase.execute(query)
';
```

Call the procedure:
```sql
CALL log_event_py('Hello from Python!');
```

If you query the `audit_logs` table, you will see the record has been inserted natively within the procedure's execution context.

```sql
SELECT * FROM audit_logs;
```

Call the procedure:
```sql
CALL concat_py('hello', 'world', '');
```

**Result:**
| res |
| :--- |
| "hello world" |

## Complex Logic

You can use standard Python control flow, string manipulation, and list comprehensions.

```sql
CREATE PROCEDURE calculate_stats(val1 FLOAT, val2 FLOAT, OUT max_val FLOAT, OUT is_equal BOOLEAN)
LANGUAGE python
AS '
my_list = [val1, val2]
max_val = max(my_list)
is_equal = val1 == val2
';
```

Execution:
```sql
CALL calculate_stats(15.5, 42.0, 0.0, false);
```

**Result:**
| max_val | is_equal |
| :--- | :--- |
| 42.0 | false |

## Type Mapping

Python types are automatically converted back and forth:
- `int` <-> SQL `INTEGER`
- `float` <-> SQL `FLOAT`
- `str` <-> SQL `TEXT`
- `bool` <-> SQL `BOOLEAN`
- `None` <-> SQL `NULL`
