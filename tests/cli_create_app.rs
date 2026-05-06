use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_create_app() {
    let temp_dir = tempdir().unwrap();
    let app_name = "test-app";
    let app_path = temp_dir.path().join(app_name);

    // Run oxibase create-app test-app
    let mut cmd = Command::cargo_bin("oxibase").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("create-app")
        .arg(app_name)
        .assert()
        .success()
        .stdout(predicates::str::contains(format!(
            "App '{}' created",
            app_name
        )));

    // Verify directory structure
    assert!(app_path.exists());
    assert!(app_path.join("data").exists());
    assert!(app_path.join("templates").exists());
    assert!(app_path.join("routes").exists());
    assert!(app_path.join("functions").exists());

    // Verify files were created
    assert!(app_path.join("data/001_init.sql").exists());
    assert!(app_path.join("templates/layout.html").exists());
    assert!(app_path.join("templates/index.html").exists());
    assert!(app_path.join("routes/web.json").exists());
    assert!(app_path.join("functions/hello.rhai").exists());
}

#[test]
fn test_create_app_already_exists() {
    let temp_dir = tempdir().unwrap();
    let app_name = "existing-app";
    let app_path = temp_dir.path().join(app_name);

    // Create the directory beforehand
    fs::create_dir(&app_path).unwrap();

    // Run oxibase create-app existing-app
    let mut cmd = Command::cargo_bin("oxibase").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("create-app")
        .arg(app_name)
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
}
