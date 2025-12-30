// Copyright 2025 Oxibase Contributors
//
// Example demonstrating user-defined functions with Rhai
//
// This example shows how to create and use user-defined functions
// written in Rhai scripting language.
//
// User-defined functions access their arguments via arg0, arg1, etc.

use oxibase::{Database, Result};

fn main() -> Result<()> {
    // Open an in-memory database
    let db = Database::open("memory://")?;

    // Create a user-defined function that returns a string
    db.execute(
        "CREATE FUNCTION hello() RETURNS TEXT LANGUAGE RHAI AS '\"Hello, World!\"';",
        (),
    )?;

    // Use the function in a query
    let result: String = db.query_one("SELECT hello()", ())?;
    println!("Result: {}", result);

    // Create a function that adds two numbers
    db.execute(
        r#"CREATE FUNCTION add_nums(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE RHAI AS 'arg0 + arg1'"#,
        (),
    )?;

    let sum: i64 = db.query_one("SELECT add_nums(5, 3)", ())?;
    println!("5 + 3 = {}", sum);

    // Create a mathematical function (Fibonacci)
    db.execute(
        r#"CREATE FUNCTION fibonacci(n INTEGER) RETURNS INTEGER LANGUAGE RHAI AS '
            if arg0 <= 1 {
                arg0
            } else {
                let a = 0;
                let b = 1;
                for i in 2..=arg0 {
                    let temp = a + b;
                    a = b;
                    b = temp;
                }
                b
            }
        '"#,
        (),
    )?;

    let fib: i64 = db.query_one("SELECT fibonacci(10)", ())?;
    println!("Fibonacci(10) = {}", fib);

    // Create a simple multiplication function
    db.execute(
        r#"CREATE FUNCTION multiply(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE RHAI AS 'arg0 * arg1'"#,
        (),
    )?;

    let product: i64 = db.query_one("SELECT multiply(6, 7)", ())?;
    println!("6 * 7 = {}", product);

    // Create a conditional function
    db.execute(
        r#"CREATE FUNCTION categorize_age(age INTEGER) RETURNS TEXT LANGUAGE RHAI AS '
            if age < 13 {
                "child"
            } else if age < 20 {
                "teenager"
            } else if age < 65 {
                "adult"
            } else {
                "senior"
            }
        '"#,
        (),
    )?;

    let category: String = db.query_one("SELECT categorize_age(25)", ())?;
    println!("Age 25 category: {}", category);

    Ok(())
}