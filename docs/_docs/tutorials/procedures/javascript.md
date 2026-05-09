---
title: "Stored Procedures in JavaScript"
excerpt: "How to write stored procedures using the JavaScript (Boa) engine."
layout: default
parent: "Stored Procedures"
has_children: true
nav_order: 3
---

# Stored Procedures in JavaScript

Oxibase integrates the [Boa Engine](https://boajs.dev/), an experimental JavaScript lexer, parser, and compiler written in Rust. This allows you to write business logic using the ubiquitous ECMAScript syntax.

> **Feature Flag Required**: To use JavaScript procedures, Oxibase must be compiled with the `js` feature flag enabled.

## Basic Usage

When creating a JavaScript procedure, use `LANGUAGE js`. Procedure arguments (including `OUT` parameters) are injected into the global Javascript execution context as variables.

```sql
CREATE PROCEDURE multiply_js(a INT, b INT, OUT res INT) 
LANGUAGE js 
AS '
    res = a * b;
';
```

## Executing SQL Commands

JavaScript stored procedures have access to the main database engine via the global `oxibase` object. You can execute standard SQL queries natively.

The `oxibase.execute(query)` function returns the number of rows affected by the statement.

```sql
CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    message TEXT
);

CREATE PROCEDURE log_event_js(msg TEXT) 
LANGUAGE js 
AS '
    // We can use JS template literals for string interpolation
    let query = `INSERT INTO audit_logs (message) VALUES (''${msg}'')`;
    
    // Execute the query
    let rows_affected = oxibase.execute(query);
';
```

Call the procedure:
```sql
CALL log_event_js('Hello from JavaScript!');
```

If you query the `audit_logs` table, you will see the record has been inserted natively within the procedure's execution context.

Call the procedure:
```sql
CALL multiply_js(5, 4, 0);
```

**Result:**
| res |
| :--- |
| 20 |

## Working with Types

The JavaScript engine handles type translation seamlessly between Oxibase's native SQL types and JavaScript's runtime types:

- SQL `INTEGER` -> JS Number
- SQL `FLOAT` -> JS Number
- SQL `TEXT` -> JS String
- SQL `BOOLEAN` -> JS Boolean
- SQL `NULL` -> JS Null

Because JS numbers are internally floating-point values, Oxibase safely coerces integer values back to SQL `INTEGER` types if the resulting JS Number has no fractional part.

```sql
CREATE PROCEDURE generate_greeting(first_name TEXT, is_morning BOOLEAN, OUT greeting TEXT)
LANGUAGE js
AS '
    if (is_morning) {
        greeting = "Good morning, " + first_name + "!";
    } else {
        greeting = `Hello there, ${first_name}!`;
    }
';
```

Execution:
```sql
CALL generate_greeting('Alice', true, '');
```

**Result:**
| greeting |
| :--- |
| "Good morning, Alice!" |
