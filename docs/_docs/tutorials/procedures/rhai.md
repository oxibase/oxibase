---
title: "Stored Procedures in Rhai"
layout: default
parent: "Stored Procedures"
has_children: true
nav_order: 2
---

# Stored Procedures in Rhai

[Rhai](https://rhai.rs/) is a fast, embedded scripting language designed specifically for Rust. It is the default procedural language in Oxibase and is always available without requiring any additional compilation feature flags.

Rhai's syntax is very similar to Rust and C-like languages, making it an intuitive choice for logic encapsulation.

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

## Executing SQL Commands

Rhai stored procedures have access to the main database engine via the global `oxibase` object. You can execute standard SQL queries (like `INSERT`, `UPDATE`, `DELETE`) natively.

The `oxibase::execute(query)` function returns the number of rows affected by the statement.

```sql
CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    message TEXT
);

CREATE PROCEDURE log_event(msg TEXT) 
LANGUAGE rhai 
AS '
    // We can concatenate strings using Rhai syntax
    let query = "INSERT INTO audit_logs (message) VALUES (''" + msg + "'')";
    
    // Execute the query
    let rows_affected = oxibase::execute(query);
    
    if rows_affected > 0 {
        // success
    }
';
```

Call the procedure:
```sql
CALL log_event('Hello from Rhai!');
```

If you query the `audit_logs` table, you will see the record has been inserted natively within the procedure's execution context.

```sql
SELECT * FROM audit_logs;
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
