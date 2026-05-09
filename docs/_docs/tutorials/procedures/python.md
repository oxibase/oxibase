---
title: "Stored Procedures in Python"
permalink: /docs/tutorials/procedures/python/
excerpt: "How to write stored procedures using the Python (RustPython) engine."
layout: doc
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
