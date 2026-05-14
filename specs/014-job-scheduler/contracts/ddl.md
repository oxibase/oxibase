# DDL Contract: Job Scheduler

The Job Scheduler exposes the following SQL commands to end users.

## 1. CREATE SCHEDULE

Creates a new automated background job.

**Syntax**:
```sql
CREATE SCHEDULE <job_name> CRON '<cron_expression>' AS '<sql_command>';
```

**Example**:
```sql
CREATE SCHEDULE nightly_backup CRON '0 0 * * *' AS 'CALL perform_backup()';
```

## 2. ALTER SCHEDULE

Modifies an existing job schedule, specifically to pause or resume it.

**Syntax**:
```sql
ALTER SCHEDULE <job_name> ACTIVE <true|false>;
```

**Example**:
```sql
ALTER SCHEDULE nightly_backup ACTIVE false;
```

## 3. DROP SCHEDULE

Permanently removes a job schedule.

**Syntax**:
```sql
DROP SCHEDULE <job_name>;
```

**Example**:
```sql
DROP SCHEDULE nightly_backup;
```
