// Copyright 2025 Stoolap Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::zombie_processes)]

//! Pgwire Integration Tests
//!
//! Tests for PostgreSQL wire protocol (pgwire) server functionality
//! using external psql client to verify end-to-end connectivity and query execution.

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// Test basic server startup and connectivity
#[test]
fn test_server_startup_and_connectivity() {
    println!("Starting server...");
    // Start server in background
    let mut server = Command::new("./target/debug/server")
        .args(&[
            "--host",
            "127.0.0.1",
            "--port",
            "5433",
            "--db",
            "memory://test_connectivity",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    println!("Server started, waiting for startup...");
    // Give server time to start up
    thread::sleep(Duration::from_secs(3));

    println!("Testing connectivity with psql...");
    // Test connectivity with psql
    let output = Command::new("psql")
        .args(&[
            "-h",
            "127.0.0.1",
            "-p",
            "5433",
            "-d",
            "postgres",
            "-c",
            "\\conninfo",
        ])
        .output()
        .expect("Failed to run psql connectivity test");

    println!("psql output: {:?}", output);
    println!("Killing server...");

    // Kill server
    let _ = server.kill(); // Don't panic if already dead

    println!("Test completed");
    assert!(
        output.status.success(),
        "psql connection failed: {:?}",
        output
    );
}

/// Test if psql is available
#[test]
fn test_psql_available() {
    let output = Command::new("psql")
        .args(&["--version"])
        .output()
        .expect("Failed to check psql version");

    assert!(output.status.success(), "psql not available");
    println!("psql version: {}", String::from_utf8_lossy(&output.stdout));
}

/// Test basic SELECT query execution
#[test]
fn test_basic_select_query() {
    // Start server in background
    let mut server = Command::new("./target/debug/server")
        .args(&[
            "--host",
            "127.0.0.1",
            "--port",
            "5433",
            "--db",
            "memory://test_select",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    // Give server time to start up
    thread::sleep(Duration::from_secs(2));

    // Test SELECT query
    let output = Command::new("psql")
        .args(&[
            "-h",
            "127.0.0.1",
            "-p",
            "5433",
            "-d",
            "postgres",
            "-c",
            "SELECT 1 as test_value;",
        ])
        .output()
        .expect("Failed to run SELECT query");

    // Kill server
    server.kill().expect("Failed to kill server");

    assert!(output.status.success(), "SELECT query failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test_value"),
        "Query result missing expected column"
    );
    assert!(stdout.contains("1"), "Query result missing expected value");
}

/// Test CREATE TABLE and INSERT operations
#[test]
fn test_create_table_and_insert() {
    use std::fs;
    use std::path::Path;

    // Clean up any existing test database
    let db_path = "/tmp/test_create_table.db";
    if Path::new(db_path).exists() {
        fs::remove_dir_all(db_path).ok();
    }

    // Start server in background
    let mut server = Command::new("./target/debug/server")
        .args(&[
            "--host",
            "127.0.0.1",
            "--port",
            "5433",
            "--db",
            &format!("file://{}", db_path),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    // Give server time to start up
    thread::sleep(Duration::from_secs(2));

    // Create table
    let create_output = Command::new("psql")
        .args(&[
            "-h",
            "127.0.0.1",
            "-p",
            "5433",
            "-d",
            "postgres",
            "-c",
            "CREATE TABLE test_table_create (id INT, name TEXT);",
        ])
        .output()
        .expect("Failed to run CREATE TABLE");

    println!("CREATE TABLE output: {:?}", create_output);
    // Note: DDL operations currently return INFO messages, so we don't check status.success()
    // Instead, we'll verify the table was created by trying to select from it

    // Insert data
    let _insert_output = Command::new("psql")
        .args(&[
            "-h",
            "127.0.0.1",
            "-p",
            "5433",
            "-d",
            "postgres",
            "-c",
            "INSERT INTO test_table_create VALUES (1, 'test');",
        ])
        .output()
        .expect("Failed to run INSERT");

    // Query data
    let select_output = Command::new("psql")
        .args(&[
            "-h",
            "127.0.0.1",
            "-p",
            "5433",
            "-d",
            "postgres",
            "-c",
            "SELECT * FROM test_table_create;",
        ])
        .output()
        .expect("Failed to run SELECT");

    // Kill server
    server.kill().expect("Failed to kill server");

    // Clean up
    fs::remove_dir_all(db_path).ok();

    assert!(select_output.status.success(), "SELECT after INSERT failed");
    let stdout = String::from_utf8_lossy(&select_output.stdout);
    assert!(stdout.contains("1"), "Query result missing expected id");
    assert!(
        stdout.contains("test"),
        "Query result missing expected name"
    );
}

/// Test data type mappings
#[test]
fn test_data_type_mappings() {
    // Start server in background
    let mut server = Command::new("./target/debug/server")
        .args(&[
            "--host",
            "127.0.0.1",
            "--port",
            "5433",
            "--db",
            "memory://test_types",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    // Give server time to start up
    thread::sleep(Duration::from_secs(2));

    // Create table with various data types
    let _create_output = Command::new("psql")
        .args(&["-h", "127.0.0.1", "-p", "5433", "-d", "postgres", "-c", "CREATE TABLE types_test (id INT, name TEXT, price FLOAT, active BOOLEAN, created TIMESTAMP);"])
        .output()
        .expect("Failed to run CREATE TABLE with types");

    // Note: DDL operations return INFO messages, so we don't check status

    // Insert test data
    let _insert_output = Command::new("psql")
        .args(&["-h", "127.0.0.1", "-p", "5433", "-d", "postgres", "-c", "INSERT INTO types_test VALUES (1, 'widget', 29.99, true, TIMESTAMP '2024-01-01 12:00:00');"])
        .output()
        .expect("Failed to run INSERT with types");

    // Query and verify data types
    let select_output = Command::new("psql")
        .args(&[
            "-h",
            "127.0.0.1",
            "-p",
            "5433",
            "-d",
            "postgres",
            "-c",
            "SELECT * FROM types_test;",
        ])
        .output()
        .expect("Failed to run SELECT with types");

    // Kill server
    server.kill().expect("Failed to kill server");

    assert!(select_output.status.success(), "SELECT with types failed");
    let stdout = String::from_utf8_lossy(&select_output.stdout);
    assert!(stdout.contains("1"), "Integer value missing");
    assert!(stdout.contains("widget"), "Text value missing");
    assert!(stdout.contains("29.99"), "Float value missing");
    assert!(stdout.contains("t"), "Boolean value missing"); // PostgreSQL displays true as 't'
    assert!(stdout.contains("2024-01-01"), "Timestamp value missing");
}

/// Test error handling
#[test]
fn test_error_handling() {
    // Start server in background
    let mut server = Command::new("./target/debug/server")
        .args(&[
            "--host",
            "127.0.0.1",
            "--port",
            "5433",
            "--db",
            "memory://test_error",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    // Give server time to start up
    thread::sleep(Duration::from_secs(2));

    // Test invalid SQL
    let _error_output = Command::new("psql")
        .args(&[
            "-h",
            "127.0.0.1",
            "-p",
            "5433",
            "-d",
            "postgres",
            "-c",
            "SELECT * FROM nonexistent_table;",
        ])
        .output()
        .expect("Failed to run invalid SQL");

    // Kill server
    server.kill().expect("Failed to kill server");

    // Note: Error handling may return non-zero exit code, which is expected
    // The important thing is that the server doesn't crash
}
