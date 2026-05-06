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
