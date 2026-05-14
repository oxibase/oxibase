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

use oxibase::Database;

#[test]
fn test_create_alter_drop_schedule() {
    let db = Database::open("memory://job_test_1").unwrap();

    // Verify system.cron table is empty initially
    let result = db.query("SELECT COUNT(*) FROM system.cron", ()).unwrap();
    let rows: Vec<_> = result.collect();
    let count: i64 = rows[0].as_ref().unwrap().get(0).unwrap();
    assert_eq!(count, 0);

    // Create a schedule
    db.execute(
        "CREATE SCHEDULE my_job CRON '0 0 0 * * * *' AS 'CALL my_proc()'",
        (),
    )
    .unwrap();

    // Verify it was added
    let result = db
        .query(
            "SELECT name, schedule, command, active FROM system.cron",
            (),
        )
        .unwrap();
    let rows: Vec<_> = result.collect();
    assert_eq!(rows.len(), 1);

    let row = rows[0].as_ref().unwrap();
    let name: String = row.get(0).unwrap();
    let schedule: String = row.get(1).unwrap();
    let command: String = row.get(2).unwrap();
    let active: bool = row.get(3).unwrap();

    assert_eq!(name, "MY_JOB");
    assert_eq!(schedule, "0 0 0 * * * *");
    assert_eq!(command, "CALL my_proc()");
    assert!(active);

    // Alter schedule
    db.execute("ALTER SCHEDULE my_job ACTIVE false", ())
        .unwrap();

    // Verify it was altered
    let result = db
        .query("SELECT active FROM system.cron WHERE name = 'MY_JOB'", ())
        .unwrap();
    let rows: Vec<_> = result.collect();
    let active: bool = rows[0].as_ref().unwrap().get(0).unwrap();
    assert!(!active);

    // Drop schedule
    db.execute("DROP SCHEDULE my_job", ()).unwrap();

    // Verify it was dropped
    let result = db.query("SELECT COUNT(*) FROM system.cron", ()).unwrap();
    let rows: Vec<_> = result.collect();
    let count: i64 = rows[0].as_ref().unwrap().get(0).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_autonomous_job_execution() {
    let db = Database::open("memory://job_test_2?scheduler=true").unwrap();

    // Create a target table
    db.execute("CREATE TABLE job_log (msg TEXT)", ()).unwrap();

    // Create a job that runs every second
    db.execute(
        "CREATE SCHEDULE my_job CRON '* * * * * * *' AS 'INSERT INTO job_log VALUES (''Job executed'')'",
        (),
    )
    .unwrap();

    // Wait for 2 seconds to allow the job to run at least once
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Check if the job executed
    let result = db.query("SELECT COUNT(*) FROM job_log", ()).unwrap();
    let rows: Vec<_> = result.collect();
    let count: i64 = rows[0].as_ref().unwrap().get(0).unwrap();
    assert!(count >= 1, "Job should have executed at least once");

    // Check if the run log was created
    let result = db.query("SELECT status FROM system.cron_runs", ()).unwrap();
    let rows: Vec<_> = result.collect();
    assert!(!rows.is_empty(), "Run log should have been created");

    let status: String = rows[0].as_ref().unwrap().get(0).unwrap();
    assert_eq!(status, "SUCCESS");
}
