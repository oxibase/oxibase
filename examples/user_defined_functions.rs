// Copyright 2025 Oxibase Contributors
//
// Example demonstrating user-defined functions with Deno
//
// This example shows how to create and use user-defined functions
// written in JavaScript/TypeScript that execute using the Deno runtime.
//
// User-defined functions access their arguments via the 'arguments' array.

use oxibase::{Database, Result};

fn main() -> Result<()> {
    // Open an in-memory database
    let db = Database::open("memory://")?;

    // Create a user-defined function that returns a string
    db.execute(
        "CREATE FUNCTION hello() RETURNS TEXT LANGUAGE DENO AS 'return \"Hello, World!\";'",
        (),
    )?;

    // Use the function in a query
    let result: String = db.query_one("SELECT hello()", ())?;
    println!("Result: {}", result);

    // Create a function that adds two numbers
    db.execute(
        r#"CREATE FUNCTION add_nums(a INTEGER, b INTEGER) RETURNS INTEGER LANGUAGE DENO AS 'return arguments[0] + arguments[1];'"#,
        (),
    )?;

    let sum: i64 = db.query_one("SELECT add_nums(5, 3)", ())?;
    println!("5 + 3 = {}", sum);

    // Create a greeting function
    db.execute(
        "CREATE FUNCTION greet(name TEXT) RETURNS TEXT LANGUAGE DENO AS 'return `Hello, ${arguments[0]}!`;'",
        (),
    )?;

    let greeting: String = db.query_one("SELECT greet('Rust')", ())?;
    println!("Greeting: {}", greeting);

    Ok(())
}
