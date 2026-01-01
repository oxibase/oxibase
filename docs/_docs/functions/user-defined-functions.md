---
layout: default
title: "User-Defined Functions"
parent: Functions
nav_order: 4
---

# User-Defined Functions

OxiBase supports user-defined functions written in multiple scripting languages through pluggable backends. By default, functions can be written in Rhai (a lightweight, fast scripting language), with optional support for JavaScript/TypeScript (via Deno) and Python (via RustPython). This allows you to extend the database with custom logic while choosing the right tool for each use case.

## Overview

User-defined functions (UDFs) enable you to create custom scalar functions that can be called from SQL queries. Functions run in secure, isolated environments with controlled access to system resources.

## Scripting Backends

OxiBase supports multiple scripting backends, each optimized for different use cases:

### Rhai Backend (Default)
- **Language**: `LANGUAGE RHAI`
- **Description**: Lightweight, fast scripting language written in Rust
- **Performance**: Excellent performance for simple calculations and logic
- **Availability**: Always enabled
- **Use Case**: General-purpose scripting, high-performance requirements

### Deno Backend (Optional)
- **Language**: `LANGUAGE DENO` or `LANGUAGE JAVASCRIPT`
- **Description**: Full JavaScript/TypeScript runtime with modern ES features
- **Performance**: Good performance with rich ecosystem support
- **Availability**: Enable with `--features deno`
- **Use Case**: Complex logic, JSON processing, date manipulation

### Python Backend (Optional)
- **Language**: `LANGUAGE PYTHON`
- **Description**: Python scripting with access to standard library
- **Performance**: Good performance with extensive libraries
- **Availability**: Enable with `--features python`
- **Use Case**: Scientific computing, data processing, ML/AI integration

## Enabling Optional Backends

To use JavaScript/TypeScript or Python functions, enable the corresponding feature flags:

```bash
# Enable JavaScript/TypeScript support
cargo build --features deno

# Enable Python support
cargo build --features python

# Enable both
cargo build --features deno,python
```

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
LANGUAGE DENO AS 'return price * 0.08;';
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
- `LANGUAGE RHAI|DENO|PYTHON`: Specifies the scripting backend to use
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

All backends now support **named parameters**. Arguments are accessed by their parameter names:

#### Rhai Backend
Parameters are bound to variables with their declared names:

```sql
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE RHAI AS 'a + b';
```

#### Deno Backend
Parameters are set as global variables with their declared names:

```sql
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE DENO AS 'return a + b;';
```

#### Python Backend
Parameters are set as local variables with their declared names:

```sql
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE PYTHON AS 'return a + b';
```

Python functions support the same argument and return types as JavaScript:
- **INTEGER**: Python `int`
- **FLOAT**: Python `float`
- **TEXT**: Python `str`
- **BOOLEAN**: Python `bool`
- **JSON**: Python objects (dict/list parsed from JSON string)

Example functions:

```sql
-- String manipulation
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE PYTHON AS 'return f"Hello, {name}!"';

-- Mathematical operations
CREATE FUNCTION power(base INTEGER, exp INTEGER)
RETURNS INTEGER
LANGUAGE PYTHON AS 'return base ** exp';

-- JSON processing
CREATE FUNCTION extract_field(json_data JSON, field TEXT)
RETURNS TEXT
LANGUAGE PYTHON AS '''
import json
data = json.loads(json_data)
return data.get(field, "")
''';
```

### Return Values

Functions return values using backend-specific syntax:

#### Rhai Backend
```sql
-- Return a string
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE RHAI AS '"Hello, " + name + "!"';

-- Return a number
CREATE FUNCTION square(x INTEGER)
RETURNS INTEGER
LANGUAGE RHAI AS 'x * x';

-- Return a boolean
CREATE FUNCTION is_even(n INTEGER)
RETURNS BOOLEAN
LANGUAGE RHAI AS 'n % 2 == 0';
```

#### Deno Backend
```sql
-- Return a string
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE DENO AS 'return `Hello, ${name}!`;';

-- Return a number
CREATE FUNCTION square(x INTEGER)
RETURNS INTEGER
LANGUAGE DENO AS 'return x * x;';

-- Return a boolean
CREATE FUNCTION is_even(n INTEGER)
RETURNS BOOLEAN
LANGUAGE DENO AS 'return n % 2 === 0;';

-- Return JSON
CREATE FUNCTION create_person(name TEXT, age INTEGER)
RETURNS JSON
LANGUAGE DENO AS 'return { name: name, age: age };';
```

#### Python Backend
```sql
-- Return a string
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE PYTHON AS 'return f"Hello, {name}!"';

-- Return a number
CREATE FUNCTION square(x INTEGER)
RETURNS INTEGER
LANGUAGE PYTHON AS 'return x * x';

-- Return a boolean
CREATE FUNCTION is_even(n INTEGER)
RETURNS BOOLEAN
LANGUAGE PYTHON AS 'return n % 2 == 0';

-- Return JSON
CREATE FUNCTION create_person(name TEXT, age INTEGER)
RETURNS JSON
LANGUAGE PYTHON AS '''
import json
return json.dumps({"name": name, "age": age})
''';
```

**Note**: Python functions support both `return` statements (recommended) and direct `result =` assignments for backward compatibility. If no value is returned, the function returns `NULL`.

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
        currency: currency
    }).format(amount / 100);
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

All backends execute in secure sandboxes with controlled access:

### Rhai Backend
- **No file system access** - Pure computation only
- **No network access** - Cannot make HTTP requests
- **Memory isolation** - Each call runs in isolated context
- **Limited runtime** - Execution time limits prevent abuse

### Deno Backend
- **No file system access** - Cannot read or write files
- **No network access** - Cannot make HTTP requests or open sockets
- **No system access** - Cannot execute system commands or access environment variables
- **Limited runtime** - Functions have execution time limits to prevent abuse
- **Memory isolation** - Each function call runs in its own JavaScript context

### Python Backend
- **No file system access** - Cannot read or write files
- **No network access** - Cannot make HTTP requests
- **Limited system access** - Cannot execute system commands
- **Memory isolation** - Each call runs in isolated context
- **Limited runtime** - Execution time limits prevent abuse

## Performance Characteristics

Performance varies by backend:

### Rhai Backend
- **Runtime Creation**: Minimal overhead (microseconds)
- **Execution Speed**: Fastest for simple calculations
- **Memory Usage**: Low memory footprint
- **Best For**: High-frequency calls, simple logic

### Deno Backend
- **Runtime Creation**: Moderate overhead (~milliseconds)
- **Execution Speed**: Good for complex logic
- **Memory Usage**: Higher memory usage
- **Best For**: Complex algorithms, JSON processing

### Python Backend
- **Runtime Creation**: Moderate overhead (~milliseconds)
- **Execution Speed**: Good for numerical computing
- **Memory Usage**: Moderate memory usage
- **Best For**: Scientific computing, data processing

### General Guidelines
- **Rhai** is recommended for most use cases due to its speed and low overhead
- Each function call creates a new runtime instance for isolation
- Consider caching results at the application level for expensive operations
- Runtime creation overhead is typically acceptable for OLTP workloads

## Examples

### Basic Arithmetic (Rhai)

```sql
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE RHAI AS 'a + b';

CREATE FUNCTION calculate_area(width INTEGER, height INTEGER)
RETURNS INTEGER
LANGUAGE RHAI AS 'width * height';

SELECT add_numbers(5, 3) as sum, calculate_area(10, 20) as area;
-- Result: sum = 8, area = 200
```

### String Processing (Rhai)

```sql
CREATE FUNCTION slugify(text TEXT)
RETURNS TEXT
LANGUAGE RHAI AS '
    text
        .to_lower()
        .replace(re("[^a-z0-9]+"), "-")
        .replace(re("^-+|-+$"), "")
';

SELECT slugify('Hello, World! How are you?') as slug;
-- Result: "hello-world-how-are-you"
```

### String Processing (Deno)

```sql
CREATE FUNCTION slugify(text TEXT)
RETURNS TEXT
LANGUAGE DENO AS '
    return text
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-+|-+$/g, "");
';

SELECT slugify('Hello, World! How are you?') as slug;
-- Result: "hello-world-how-are-you"
```

### Date Calculations (Rhai)

```sql
CREATE FUNCTION days_until(date TEXT)
RETURNS INTEGER
LANGUAGE RHAI AS '
    // Simple date difference calculation
    // Note: Rhai has limited date support, consider Deno for complex date operations
    30  // Placeholder - use Deno for real date calculations
';

// For complex date operations, use Deno:
CREATE FUNCTION days_until(date TIMESTAMP)
RETURNS INTEGER
LANGUAGE DENO AS '
    const target = new Date(date);
    const now = new Date();
    const diff = target - now;
    return Math.ceil(diff / (1000 * 60 * 60 * 24));
';

SELECT days_until('2024-12-31') as days_remaining;
```

### JSON Processing (Deno)

```sql
CREATE FUNCTION extract_field(json_doc JSON, field TEXT)
RETURNS TEXT
LANGUAGE DENO AS '
    const doc = JSON.parse(json_doc);
    return doc[field] || null;
';

SELECT extract_field(metadata, 'version') as version
FROM documents;
```

### Mathematical Functions (Rhai)

```sql
CREATE FUNCTION fibonacci(n INTEGER)
RETURNS INTEGER
LANGUAGE RHAI AS '
    if n <= 1 { n }
    else {
        let a = 0;
        let b = 1;
        for i in 2..=n {
            let temp = a + b;
            a = b;
            b = temp;
        }
        b
    }
';

CREATE FUNCTION factorial(n INTEGER)
RETURNS INTEGER
LANGUAGE RHAI AS '
    if n <= 1 { 1 }
    else { n * factorial(n - 1) }
';

SELECT fibonacci(10) as fib, factorial(5) as fact;
-- Result: fib = 55, fact = 120
```

### Data Validation (Deno)

```sql
CREATE FUNCTION validate_email(email TEXT)
RETURNS BOOLEAN
LANGUAGE DENO AS '
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
';

CREATE FUNCTION is_strong_password(password TEXT)
RETURNS BOOLEAN
LANGUAGE DENO AS '
    // At least 8 characters, 1 uppercase, 1 lowercase, 1 number
    const strongRegex = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[a-zA-Z\d]{8,}$/;
    return strongRegex.test(password);
';

SELECT validate_email('user@example.com') as valid_email,
       is_strong_password('MyPass123') as strong_password;
```

## Error Handling

Functions that throw exceptions will cause the query to fail:

```sql
-- Rhai
CREATE FUNCTION safe_divide(a INTEGER, b INTEGER)
RETURNS FLOAT
LANGUAGE RHAI AS '
    if b == 0 {
        throw "Division by zero";
    }
    a / b
';

-- Deno
CREATE FUNCTION safe_divide(a INTEGER, b INTEGER)
RETURNS FLOAT
LANGUAGE DENO AS '
    if (b === 0) {
        throw new Error("Division by zero");
    }
    return a / b;
';
```

## Backend-Specific Considerations

### Rhai Backend
- **Syntax**: Simple, Rust-like syntax with automatic type inference
- **Features**: Basic arithmetic, string operations, conditionals, loops
- **Limitations**: Limited standard library, no built-in JSON parsing
- **Performance**: Excellent for numerical computations and simple logic

### Deno Backend
- **Syntax**: Full JavaScript/TypeScript with modern ES features
- **Features**: Rich standard library, JSON support, date/time operations
- **Limitations**: Higher memory usage and startup time
- **Performance**: Good for complex algorithms and data processing

### Python Backend
- **Syntax**: Standard Python syntax
- **Features**: Extensive standard library, good for numerical computing
- **Limitations**: Moderate startup time, higher memory usage
- **Performance**: Good for algorithms requiring complex data structures

## Limitations

- Only scalar functions are supported (not aggregate or window functions)
- Functions cannot access database state directly
- Maximum execution time per function call is limited
- Memory usage per function is bounded
- No access to external modules or packages (backend-specific limitations apply)
- Rhai has a smaller standard library compared to JavaScript/Python

## Best Practices

1. **Choose the right backend**:
   - Use **Rhai** for simple, high-performance calculations
   - Use **Deno** for complex logic, JSON processing, or date operations
   - Use **Python** for scientific computing or when you need extensive libraries

2. **Keep functions simple** - Complex logic is better handled in application code

3. **Validate inputs** - Functions should handle edge cases and invalid inputs gracefully

4. **Use appropriate return types** - Match the function's purpose with the correct data type

5. **Test thoroughly** - User-defined functions should be well-tested across all backends

6. **Consider performance** - Rhai is fastest, but choose based on your specific needs

7. **Document your functions** - Use meaningful names and consider adding comments

8. **Handle errors appropriately** - Different backends have different error handling patterns

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