---
title: "Stored Procedures in Rhai"
permalink: /docs/sql-features/procedures/rhai/
excerpt: "How to write stored procedures using the Rhai scripting language."
layout: doc
---

# Stored Procedures in Rhai

[Rhai](https://rhai.rs/) is a fast, embedded scripting language designed specifically for Rust. It is the default procedural language in Oxibase and is always available without requiring any additional compilation feature flags.

Rhai's syntax is very similar to Rust and JavaScript, making it an intuitive choice for logic encapsulation.

## Basic Usage

When creating a procedure, set the `LANGUAGE` to `rhai`. 

```sql
CREATE PROCEDURE update_status(is_active BOOLEAN, OUT result_msg TEXT) 
LANGUAGE rhai 
AS '
    if is_active {
        result_msg = "The user is now active";
    } else {
        result_msg = "The user is suspended";
    }
';
```

Call the procedure, passing an initial placeholder value for the `OUT` parameter:
```sql
CALL update_status(true, "");
```

**Result:**
| result_msg |
| :--- |
| "The user is now active" |

## Mutating INOUT Parameters

`INOUT` parameters can be read from and written to. The initial value is accessible inside the script, and whatever value it holds at the end of the execution is returned.

```sql
CREATE PROCEDURE apply_tax(price FLOAT, tax_rate FLOAT, INOUT total FLOAT) 
LANGUAGE rhai
AS '
    // Read the initial INOUT value (total)
    let base = total + price;
    
    // Mutate the INOUT value
    total = base + (base * tax_rate);
';
```

Execution:
```sql
CALL apply_tax(100.0, 0.20, 50.0);
```

**Result:**
| total |
| :--- |
| 180.0 |

## Types and Conversion

Rhai supports dynamic typing, but it seamlessly binds to Oxibase's native SQL types (Integer, Float, Boolean, Text).

To explicitly cast variables within Rhai, Oxibase exposes several built-in conversion functions you can use inside your scripts:
- `to_string(value)` -> Text
- `to_int(value)` -> Integer
- `to_float(value)` -> Float

```sql
CREATE PROCEDURE format_id(id INT, prefix TEXT, OUT formatted TEXT)
LANGUAGE rhai
AS '
    formatted = prefix + "-" + to_string(id);
';
```
