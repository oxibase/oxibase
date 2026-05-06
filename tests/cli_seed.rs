// Copyright 2025 Stoolap Contributors
// Copyright 2025 Oxibase Contributors
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

use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn test_seed_command() {
    let temp_dir = tempdir().unwrap();
    let app_name = "test-app";

    // Create the app scaffold
    let mut cmd = Command::cargo_bin("oxibase").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("create-app")
        .arg(app_name)
        .assert()
        .success();

    let db_path = temp_dir.path().join("test.db");
    let db_uri = format!("file://{}", db_path.to_string_lossy());

    // Seed the database
    let mut cmd = Command::cargo_bin("oxibase").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("seed")
        .arg(app_name)
        .arg("--db")
        .arg(&db_uri)
        .assert()
        .success()
        .stdout(predicates::str::contains("App seeded successfully"));

    // Verify database contents (the -e flag is a top-level arg, and db connects implicitly via DSN or needs to be set properly. We use -e without repl command because repl is interactive, execute is global)
    let mut cmd = Command::cargo_bin("oxibase").unwrap();
    cmd.arg("-e")
        .arg("SELECT count(*) FROM users;")
        .arg("repl") // The way oxibase argument parsing is setup, -e is a top level flag, then repl db follows
        .arg("-d")
        .arg(&db_uri)
        .assert()
        .success()
        .stdout(predicates::str::contains("2")); // 2 users from init.sql

    let mut cmd = Command::cargo_bin("oxibase").unwrap();
    cmd.arg("-e")
        .arg("SELECT count(*) FROM routes.definitions;")
        .arg("repl")
        .arg("-d")
        .arg(&db_uri)
        .assert()
        .success()
        .stdout(predicates::str::contains("1"));

    let mut cmd = Command::cargo_bin("oxibase").unwrap();
    cmd.arg("-e")
        .arg("SELECT count(*) FROM templates.source;")
        .arg("repl")
        .arg("-d")
        .arg(&db_uri)
        .assert()
        .success()
        .stdout(predicates::str::contains("2")); // layout.html and index.html
}
