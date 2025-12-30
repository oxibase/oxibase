---
layout: default
title: "User-Defined Functions"
parent: Functions
nav_order: 4
---

# User-Defined Functions

OxiBase supports user-defined functions written in JavaScript and TypeScript that execute using the Deno runtime. This allows you to extend the database with custom logic while maintaining security and performance.

## Overview

User-defined functions (UDFs) enable you to create custom scalar functions that can be called from SQL queries. These functions run in a secure, isolated JavaScript environment with no access to the file system, network, or system resources by default.

## Functions vs Stored Procedures

OxiBase currently supports **user-defined functions** but not **stored procedures**. Understanding the difference is important for choosing the right tool for your database logic.

### Comparison at a Glance

| Feature | Function (`CREATE FUNCTION`) | Procedure (`CREATE PROCEDURE`) |
| --- | --- | --- |
| **Return Value** | **Must** return exactly one value (scalar). | Can return zero, one, or multiple values. |
| **Usage in SQL** | Can be used in `SELECT`, `WHERE`, and `JOIN`. | Must be called using `EXECUTE` or `CALL`. |
| **Data Modification** | Cannot perform DML (Insert, Update, Delete). | Can perform any DML operations. |
| **Transactions** | **No** transaction control allowed. | Supports `COMMIT`, `ROLLBACK`, and `SAVEPOINT`. |
| **Parameters** | Generally only **Input** parameters. | Supports **Input**, **Output**, and **In-Out**. |
| **Error Handling** | Limited (JavaScript exceptions only). | Full support for error handling constructs. |

### Key Differences

#### Integration with Queries

Functions are "pluggable" into your SQL statements and can be used just like built-in functions:

```sql
-- Function usage in queries
SELECT calculate_tax(price) FROM products;
SELECT * FROM users WHERE is_adult(age);
```

Procedures cannot be used directly in queries and must be called separately:

```sql
-- Procedure usage (when implemented)
EXECUTE update_inventory;
CALL process_monthly_report;
```

#### Side Effects and DML

Functions are restricted to be "side-effect free" and cannot change database state:

```sql
-- ✅ Valid function - read-only calculation
CREATE FUNCTION calculate_tax(price INTEGER) RETURNS INTEGER
LANGUAGE DENO AS 'return arguments[0] * 0.08;';
```

Procedures are designed for actions that modify data:

```sql
-- ❌ Invalid in functions - would be valid in procedures (when implemented)
-- CREATE PROCEDURE update_prices()
-- AS BEGIN
--     UPDATE products SET price = price * 1.1;
-- END;
```

#### When to Use Functions vs Procedures

**Use Functions when:**
- You need to perform calculations and use results in queries
- The logic is simple and read-only
- You want to encapsulate reusable business logic
- Examples: Currency conversion, string formatting, age calculation

**Use Procedures when:**
- You need to perform write operations (INSERT/UPDATE/DELETE)
- You need complex multi-step logic with error handling
- You need transaction control
- You need to return multiple result sets
- Examples: Monthly payroll processing, customer registration, data cleanup

> **Note:** Stored procedures are planned for future implementation in OxiBase but are not currently available.

## Creating User-Defined Functions

Use the `CREATE FUNCTION` statement to define a user-defined function:

```sql
CREATE FUNCTION function_name(param1 TYPE1, param2 TYPE2, ...)
RETURNS return_type
LANGUAGE DENO AS 'JavaScript code here';
```

### Parameters

- `function_name`: The name of the function (must be unique)
- `param1, param2, ...`: Parameter names and their data types
- `return_type`: The data type of the return value
- `LANGUAGE DENO`: Specifies that the function uses JavaScript/TypeScript
- `AS 'code'`: The JavaScript/TypeScript code that implements the function

### Supported Return Data Types

User-defined functions can return values of these scalar data types:

| Data Type | Description | JavaScript Example |
|-----------|-------------|-------------------|
| **`INTEGER`** | 64-bit signed integers | `return 42;` |
| **`FLOAT`** | 64-bit floating-point numbers | `return 3.14159;` |
| **`TEXT`** | UTF-8 text strings | `return "Hello World";` |
| **`BOOLEAN`** | True/false values | `return arguments[0] > 10;` |
| **`TIMESTAMP`** | Date and time values | `return new Date().toISOString();` |
| **`JSON`** | JSON documents and objects | `return {name: "John", age: 30};` |

Functions must return exactly one value and declare their return type in the `CREATE FUNCTION` statement. The JavaScript runtime automatically converts return values to the appropriate OxiBase type.

> **Note:** OxiBase currently only supports scalar user-defined functions. Table-valued functions and stored procedures are planned for future releases.

## Function Implementation

### Argument Access

Function arguments are accessible through the JavaScript `arguments` array:

```sql
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE DENO AS 'return arguments[0] + arguments[1];';
```

### Return Values

Functions can return any JavaScript value that can be converted to an OxiBase data type:

```sql
-- Return a string
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE DENO AS 'return `Hello, ${arguments[0]}!`;';

-- Return a number
CREATE FUNCTION square(x INTEGER)
RETURNS INTEGER
LANGUAGE DENO AS 'return arguments[0] * arguments[0];';

-- Return a boolean
CREATE FUNCTION is_even(n INTEGER)
RETURNS BOOLEAN
LANGUAGE DENO AS 'return arguments[0] % 2 === 0;';

-- Return JSON
CREATE FUNCTION create_person(name TEXT, age INTEGER)
RETURNS JSON
LANGUAGE DENO AS 'return { name: arguments[0], age: arguments[1] };';
```

### JavaScript Features

User-defined functions have access to standard JavaScript features:

- All ECMAScript built-ins (Math, Date, etc.)
- Arrow functions and modern syntax
- Template literals
- Array and object methods
- JSON parsing and serialization

```sql
CREATE FUNCTION format_currency(amount INTEGER, currency TEXT)
RETURNS TEXT
LANGUAGE DENO AS '
    const formatted = new Intl.NumberFormat("en-US", {
        style: "currency",
        currency: arguments[1]
    }).format(arguments[0] / 100);
    return formatted;
';
```

## Using User-Defined Functions

Once created, user-defined functions can be used in any SQL context where scalar functions are allowed:

```sql
-- Simple usage
SELECT greet('World') as greeting;

-- In expressions
SELECT id, square(price) as price_squared
FROM products;

-- In WHERE clauses
SELECT * FROM users
WHERE is_even(age);

-- In complex queries
SELECT
    name,
    format_currency(salary, 'USD') as formatted_salary
FROM employees;
```

## Security Considerations

User-defined functions execute in a secure sandbox:

- **No file system access** - Cannot read or write files
- **No network access** - Cannot make HTTP requests or open sockets
- **No system access** - Cannot execute system commands or access environment variables
- **Limited runtime** - Functions have execution time limits to prevent abuse
- **Memory isolation** - Each function call runs in its own JavaScript context

## Performance Characteristics

- Each function call creates a new JavaScript runtime instance
- Suitable for most applications but may not be optimal for high-frequency calls
- Consider caching results at the application level if needed
- Runtime creation overhead is typically acceptable for OLTP workloads

## Examples

### String Processing

```sql
CREATE FUNCTION slugify(text TEXT)
RETURNS TEXT
LANGUAGE DENO AS '
    return arguments[0]
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-+|-+$/g, "");
';

SELECT slugify('Hello, World! How are you?') as slug;
-- Result: "hello-world-how-are-you"
```

### Date Calculations

```sql
CREATE FUNCTION days_until(date TIMESTAMP)
RETURNS INTEGER
LANGUAGE DENO AS '
    const target = new Date(arguments[0]);
    const now = new Date();
    const diff = target - now;
    return Math.ceil(diff / (1000 * 60 * 60 * 24));
';

SELECT days_until('2024-12-31') as days_remaining;
```

### JSON Processing

```sql
CREATE FUNCTION extract_field(json_doc JSON, field TEXT)
RETURNS TEXT
LANGUAGE DENO AS '
    const doc = JSON.parse(arguments[0]);
    return doc[arguments[1]] || null;
';

SELECT extract_field(metadata, 'version') as version
FROM documents;
```

## Error Handling

Functions that throw JavaScript exceptions will cause the query to fail:

```sql
CREATE FUNCTION safe_divide(a INTEGER, b INTEGER)
RETURNS FLOAT
LANGUAGE DENO AS '
    if (arguments[1] === 0) {
        throw new Error("Division by zero");
    }
    return arguments[0] / arguments[1];
';
```

## Limitations

- Only scalar functions are supported (not aggregate or window functions)
- Functions cannot access database state directly
- Maximum execution time per function call is limited
- Memory usage per function is bounded
- No access to external modules or npm packages

## Best Practices

1. **Keep functions simple** - Complex logic is better handled in application code
2. **Validate inputs** - JavaScript functions should handle edge cases
3. **Use appropriate return types** - Match the function's purpose with the correct data type
4. **Test thoroughly** - User-defined functions should be well-tested
5. **Consider performance** - Avoid functions in performance-critical query paths
6. **Document your functions** - Use meaningful names and consider adding comments

## Dropping Functions

User-defined functions can be dropped using the `DROP FUNCTION` statement:

```sql
DROP FUNCTION function_name;
```

### Parameters

- `function_name`: The name of the function to drop
- `IF EXISTS`: Optional clause that prevents an error if the function doesn't exist

### Examples

```sql
-- Drop a function
DROP FUNCTION calculate_total;

-- Drop a function only if it exists
DROP FUNCTION IF EXISTS old_function;

-- Drop a schema-qualified function
DROP FUNCTION myschema.add_numbers;
```

### Behavior

- Dropping a function removes it from the database permanently
- The function becomes unavailable for new queries immediately
- Existing queries using the function may fail if the function is dropped during execution
- Functions are dropped from both the system catalog and the runtime registry