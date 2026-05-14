# Quickstart: Testing the Job Scheduler

This guide explains how to quickly test the job scheduler locally once implemented.

## 1. Start the CLI

Launch the oxibase CLI:

```bash
cargo run --features cli
```

## 2. Create a Schedule

Create a dummy table and a job that runs every second (`* * * * * *` is the cron expression for every second using the 6-field format supported by the `cron` crate, or you can use standard 5-field for minutely).

```sql
CREATE TABLE log_table (msg TEXT);
CREATE SCHEDULE my_logger CRON '0 * * * * *' AS 'INSERT INTO log_table VALUES (''Logged at exact minute!'')';
```
*(Wait 1 minute)*

## 3. Verify Execution

Check that the job executed by querying your target table and the system run log:

```sql
SELECT * FROM log_table;
SELECT * FROM system.cron_runs;
```

## 4. Pause the Schedule

Pause the job and verify it stops logging:

```sql
ALTER SCHEDULE my_logger ACTIVE false;
```

## 5. Clean up

```sql
DROP SCHEDULE my_logger;
```
