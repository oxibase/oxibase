# Feature Specification: Built-in Job Scheduler for Procedures

**Feature Branch**: `014-job-scheduler`  
**Created**: May 14, 2026  
**Status**: Draft  
**Input**: User description: "Built-in Job Scheduler for Procedures (Cron-like execution). CREATE/ALTER schedule syntax. Utilize a system table system.cron similar to pg_cron. Borrowing the idea of moving internal tables to the system schema."

## Clarifications

### Session 2026-05-14

- Q: System table schema migration expectations → A: Migrate all existing `_sys_*` tables (functions, procedures, statistics, triggers, etc.) to the `system` schema without the `_sys_` prefix (e.g., `system.functions`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and Manage Job Schedules (Priority: P1)

As a database administrator or developer, I want to create, alter, and drop scheduled jobs using standard SQL DDL syntax so that I can configure automated background tasks directly within the database.

**Why this priority**: Without the ability to define schedules via SQL, the job scheduler lacks an interface for users. This establishes the foundation.

**Independent Test**: Can be tested via unit and integration tests executing `CREATE SCHEDULE`, `ALTER SCHEDULE`, and `DROP SCHEDULE` statements, verifying that rows are correctly inserted, updated, and deleted in the `system.cron` table.

**Acceptance Scenarios**:

1. **Given** an initialized database, **When** the user executes `CREATE SCHEDULE my_job CRON '0 0 * * *' AS 'CALL my_proc()'`, **Then** a new active job is recorded in the `system.cron` table.
2. **Given** an existing schedule `my_job`, **When** the user executes `ALTER SCHEDULE my_job ACTIVE false`, **Then** the job is marked as inactive in `system.cron` and will no longer be queued.
3. **Given** an existing schedule `my_job`, **When** the user executes `DROP SCHEDULE my_job`, **Then** the job is removed from `system.cron`.

---

### User Story 2 - Autonomous Job Execution (Priority: P1)

As a database administrator, I want the database to automatically evaluate cron schedules in the background and execute the associated commands when their time arrives, logging the result, so that I don't need external orchestration tools.

**Why this priority**: The core value proposition is autonomous execution of the configured jobs.

**Independent Test**: Can be tested by creating a schedule with a frequent interval (e.g., every second), waiting briefly in the test loop, and verifying that records appear in `system.cron_runs` and that the configured command (e.g., an INSERT) actually occurred.

**Acceptance Scenarios**:

1. **Given** an active schedule in `system.cron` configured to run frequently, **When** the scheduled time is reached, **Then** the background worker executes the command.
2. **Given** a job execution, **When** the execution completes (either successfully or failing with an error), **Then** a log record is written to `system.cron_runs` containing the job ID, status (`SUCCESS` or `FAILED`), start time, end time, and any error message.
3. **Given** the database process is shutting down, **When** the system initiates shutdown, **Then** the background scheduler thread gracefully terminates.

---

### User Story 3 - System Schema Migration (Priority: P2)

As a database administrator, I expect all internal database catalogs and metadata tables to be neatly organized under a single `system` schema, avoiding clutter in the public namespace.

**Why this priority**: While the core feature is the job scheduler, achieving a clean architectural baseline by moving existing `_sys_*` tables establishes the standard for the new `system.cron` tables.

**Independent Test**: Can be tested by starting the database and verifying that queries to `system.functions`, `system.procedures`, `system.table_stats`, `system.column_stats`, and `system.triggers` succeed, while queries to their old `_sys_*` equivalents fail or return nothing.

**Acceptance Scenarios**:

1. **Given** database initialization, **When** the system schemas are created, **Then** all internal metadata tables are placed within the `system` schema (e.g., `system.functions`, `system.procedures`, `system.table_stats`, `system.column_stats`, `system.triggers`).
2. **Given** an existing function or procedure, **When** it is loaded from disk or queried, **Then** the engine correctly reads from the new `system.*` tables instead of `_sys_*`.

---

### Edge Cases

- What happens if the scheduled SQL command contains invalid syntax or references a non-existent procedure? (The job should fail gracefully, catching the error and recording it in `system.cron_runs` with a `FAILED` status).
- How does the system handle concurrent database instances reading the same `system.cron` table on disk? (Since this runs within the embedded/single-process database context, the single active instance will handle background execution. File locks prevent multiple instances).
- What if a job's execution takes longer than the interval between its scheduled runs? (Subsequent runs should ideally skip or queue depending on policy; by default, skip if currently running, or execute concurrently).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Engine MUST parse and execute the DDL: `CREATE SCHEDULE <name> CRON '<cron_expr>' AS '<sql>'`.
- **FR-002**: Engine MUST parse and execute the DDL: `ALTER SCHEDULE <name> ACTIVE <boolean>`.
- **FR-003**: Engine MUST parse and execute the DDL: `DROP SCHEDULE <name>`.
- **FR-004**: System MUST automatically initialize the `system.cron` and `system.cron_runs` tables if they do not exist.
- **FR-005**: System MUST persist job configurations in the `system.cron` table with columns: `id`, `name`, `schedule`, `command`, `active`.
- **FR-006**: System MUST persist execution logs in the `system.cron_runs` table with columns: `id`, `job_id`, `status`, `return_message`, `start_time`, `end_time`.
- **FR-007**: System MUST spawn a background thread/worker during database initialization that periodically wakes up, evaluates cron schedules against the current time, and executes due jobs.
- **FR-008**: The background worker MUST catch and log execution errors to `system.cron_runs` without crashing the thread or database.
- **FR-009**: System MUST migrate all existing `_sys_*` metadata tables (e.g., `_sys_functions`, `_sys_procedures`, `_sys_table_stats`, `_sys_column_stats`, `_sys_triggers`) to the `system` schema, dropping the `_sys_` prefix (e.g., `system.functions`, `system.procedures`).

### Key Entities

- **Job Schedule**: The definition of a task to be run, including its name, schedule expression, and command.
- **Job Run**: An audit record of a specific execution of a job, tracking its start/end times and outcome.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `CREATE SCHEDULE` and `DROP SCHEDULE` DDL statements execute without error and accurately reflect in the `system.cron` table.
- **SC-002**: The background worker successfully triggers and executes a job strictly adhering to its cron schedule.
- **SC-003**: Job outcomes (successes and failures) are reliably logged in `system.cron_runs`.
- **SC-004**: Passes all automated quality, testing, and validation checks without introducing regressions or unsafe error handling.
- **SC-005**: Job execution occurs asynchronously and does not block the main database operations or connection handling.
- **SC-006**: Existing system tables are successfully renamed/moved to the `system` schema, and all internal engine operations (like loading functions and statistics) correctly use the new paths.

## Assumptions

- We assume a standard, robust cron expression parser will be used for evaluating schedules.
- The `system` schema concept exists or will be established to cleanly separate user data from internal metadata.
- Scheduled commands are stored as plain text and evaluated dynamically at execution time.
