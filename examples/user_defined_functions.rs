// Copyright 2025 Oxibase Contributors
//
// Example demonstrating user-defined functions with multiple backends
//
// This example shows how to create and use user-defined functions
// written in Rhai, JavaScript (Deno), and Python scripting languages.
//
// User-defined functions access their arguments by name (e.g., 'a', 'b').
// Named parameters make functions more readable and maintainable.

use oxibase::{Database, Result};

fn main() -> Result<()> {
    // Open an in-memory database
    let db = Database::open("memory://")?;

    // Create a user-defined function that returns a string
    db.execute(
        "CREATE FUNCTION hello() RETURNS TEXT LANGUAGE RHAI AS 'return \"Hello, World!\";'",
        (),
    )?;

    // Use the function in a query
    let result: String = db.query_one("SELECT hello()", ())?;
    println!("Result: {}", result);

    // Create a function that adds two numbers
    db.execute(
        r#"CREATE FUNCTION add_nums(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE RHAI AS 'a + b'"#,
        (),
    )?;

    let sum: i64 = db.query_one("SELECT add_nums(5, 3)", ())?;
    println!("5 + 3 = {}", sum);

    // Create a greeting function
    db.execute(
        r#"CREATE FUNCTION greet(name TEXT) RETURNS TEXT LANGUAGE RHAI AS '"Hello, " + name + "!";'"#,
        (),
    )?;

    let greeting: String = db.query_one("SELECT greet('Rust')", ())?;
    println!("Greeting: {}", greeting);

    // Create a more complex function with conditional logic
    db.execute(
        r#"CREATE FUNCTION discount(price FLOAT, rate FLOAT) RETURNS FLOAT LANGUAGE RHAI AS '
            if rate > 0.5 {
                return price * 0.5; // Cap discount at 50%
            } else {
                return price * (1.0 - rate);
            }
        '"#,
        (),
    )?;

    let discounted: f64 = db.query_one("SELECT discount(100.0, 0.2)", ())?;
    println!("Discounted price: ${:.2}", discounted);

    #[cfg(feature = "deno")]
    {
        println!("\n--- Deno/JavaScript Examples ---");

        // JavaScript function using Deno backend
        db.execute(
            r#"CREATE FUNCTION js_add(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return a + b;'"#,
            (),
        )?;

        let js_sum: i64 = db.query_one("SELECT js_add(10, 15)", ())?;
        println!("JS Add: 10 + 15 = {}", js_sum);

        // String manipulation in JavaScript
        db.execute(
            r#"CREATE FUNCTION js_format(name TEXT, age INTEGER) RETURNS TEXT LANGUAGE DENO AS 'return `${name} is ${age} years old`;' "#,
            (),
        )?;

        let js_formatted: String = db.query_one("SELECT js_format('Alice', 30)", ())?;
        println!("JS Format: {}", js_formatted);
    }

    #[cfg(feature = "python")]
    {
        println!("\n--- Python Examples ---");

        // Python function
        db.execute(
            r#"CREATE FUNCTION py_multiply(x INTEGER, y INTEGER) RETURNS INTEGER LANGUAGE PYTHON AS 'return x * y'"#,
            (),
        )?;

        let py_product: i64 = db.query_one("SELECT py_multiply(7, 8)", ())?;
        println!("Python Multiply: 7 * 8 = {}", py_product);

        // List operations in Python
        db.execute(
            r#"CREATE FUNCTION py_list_sum(nums TEXT) RETURNS INTEGER LANGUAGE PYTHON AS '
import json
numbers = json.loads(nums)
return sum(numbers)
'"#,
            (),
        )?;

        let py_sum: i64 = db.query_one("SELECT py_list_sum('[1,2,3,4,5]')", ())?;
        println!("Python List Sum: {}", py_sum);
    }

    Ok(())
}
