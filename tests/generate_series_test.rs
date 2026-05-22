// Copyright 2026 Oxibase Contributors
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

//! Integration tests for GENERATE_SERIES table-valued function

use oxibase::Database;

fn create_test_db(name: &str) -> Database {
    Database::open(&format!("memory://{}", name)).expect("Failed to create in-memory database")
}

fn collect_i64(db: &Database, sql: &str) -> Vec<i64> {
    let result = db.query(sql, ()).unwrap();
    let mut values = Vec::new();
    for row in result {
        let row = row.unwrap();
        values.push(row.get::<i64>(0).unwrap());
    }
    values
}

fn collect_f64(db: &Database, sql: &str) -> Vec<f64> {
    let result = db.query(sql, ()).unwrap();
    let mut values = Vec::new();
    for row in result {
        let row = row.unwrap();
        values.push(row.get::<f64>(0).unwrap());
    }
    values
}

fn collect_string(db: &Database, sql: &str) -> Vec<String> {
    let result = db.query(sql, ()).unwrap();
    let mut values = Vec::new();
    for row in result {
        let row = row.unwrap();
        values.push(row.get::<String>(0).unwrap());
    }
    values
}

// ============================================================================
// Basic Integer Tests
// ============================================================================

#[test]
fn test_generate_series_basic() {
    let db = create_test_db("gs_basic");
    let values = collect_i64(&db, "SELECT * FROM generate_series(1, 5)");
    assert_eq!(values, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_generate_series_single_value() {
    let db = create_test_db("gs_single");
    let values = collect_i64(&db, "SELECT * FROM generate_series(3, 3)");
    assert_eq!(values, vec![3]);
}

// ============================================================================
// Float Tests
// ============================================================================

#[test]
fn test_generate_series_float() {
    let db = create_test_db("gs_float");
    let values = collect_f64(&db, "SELECT * FROM generate_series(0.0, 1.0, 0.5)");
    assert_eq!(values.len(), 3);
    assert!((values[0] - 0.0).abs() < 1e-10);
    assert!((values[1] - 0.5).abs() < 1e-10);
    assert!((values[2] - 1.0).abs() < 1e-10);
}

// ============================================================================
// Timestamp/Date Tests
// ============================================================================

#[test]
fn test_generate_series_date_days() {
    let db = create_test_db("gs_date_days");
    let values = collect_string(
        &db,
        "SELECT * FROM generate_series('2024-01-01', '2024-01-05', '1 day')",
    );
    assert_eq!(values.len(), 5);
    assert!(values[0].starts_with("2024-01-01"));
    assert!(values[4].starts_with("2024-01-05"));
}

#[test]
fn test_generate_series_timestamp_hours() {
    let db = create_test_db("gs_ts_hours");
    let values = collect_string(
        &db,
        "SELECT * FROM generate_series('2024-01-01 00:00:00', '2024-01-01 06:00:00', '2 hours')",
    );
    assert_eq!(values.len(), 4); // 00:00, 02:00, 04:00, 06:00
}

// ============================================================================
// Scalar (SELECT without FROM) Tests
// ============================================================================

#[test]
fn test_generate_series_scalar_returns_array() {
    let db = create_test_db("gs_scalar");
    let result = db.query("SELECT generate_series(1, 5)", ()).unwrap();
    let mut values = Vec::new();
    for row in result {
        let row = row.unwrap();
        let val: String = row.get(0).unwrap();
        values.push(val);
    }
    assert_eq!(values, vec!["[1, 2, 3, 4, 5]"]);
}

#[test]
fn test_generate_series_with_step() {
    let db = create_test_db("gs_step");
    let values = collect_i64(&db, "SELECT * FROM generate_series(0, 10, 2)");
    assert_eq!(values, vec![0, 2, 4, 6, 8, 10]);
}

#[test]
fn test_generate_series_descending() {
    let db = create_test_db("gs_desc");
    let values = collect_i64(&db, "SELECT * FROM generate_series(5, 1, -1)");
    assert_eq!(values, vec![5, 4, 3, 2, 1]);
}

#[test]
fn test_generate_series_auto_descending() {
    let db = create_test_db("gs_auto_desc");
    let values = collect_i64(&db, "SELECT * FROM generate_series(5, 1)");
    assert_eq!(values, vec![5, 4, 3, 2, 1]);
}

#[test]
fn test_generate_series_zero_step_error() {
    let db = create_test_db("gs_err_zero");
    let result = db.query("SELECT * FROM generate_series(1, 10, 0)", ());
    assert!(result.is_err());
}
