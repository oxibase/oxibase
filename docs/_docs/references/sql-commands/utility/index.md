---
layout: default
title: Utility Commands
parent: SQL Commands
has_children: true
---

# Utility Commands

**Utility Commands** are specialized instructions that don't fit neatly into the standard data query or manipulation categories. 

Instead of dealing with business data, utility commands are generally used to introspect the database itself (metadata), understand its state, or manage automated background processes.

## Metadata & Introspection

These commands allow you to explore the structure and capabilities of your database.

*   **[`SHOW TABLES`]({% link _docs/references/sql-commands/utility/show_tables.md %})**: Lists all tables currently existing in the database.
*   **[`SHOW INDEXES`]({% link _docs/references/sql-commands/utility/show_indexes.md %})**: Displays the indexes attached to a specific table.
*   **[`SHOW FUNCTIONS`]({% link _docs/references/sql-commands/utility/show_functions.md %})**: Lists all built-in and user-defined functions available to use in queries.
*   **[`SHOW CREATE TABLE`]({% link _docs/references/sql-commands/utility/show_create_table.md %})**: Outputs the exact DDL statement required to recreate a specific table.
*   **[`INFORMATION_SCHEMA`]({% link _docs/references/sql-commands/utility/information_schema.md %})**: A deep dive into the standard set of virtual tables (like `information_schema.columns`) that you can query using `SELECT` to programmatically analyze your schema.

## Background Job Scheduling

Oxibase has a built-in cron-based task runner for executing stored procedures automatically in the background.

*   **[`CREATE SCHEDULE`]({% link _docs/references/sql-commands/utility/create_schedule.md %})**: Registers a new automated task.
*   **[`ALTER SCHEDULE`]({% link _docs/references/sql-commands/utility/alter_schedule.md %})**: Pauses, resumes, or changes the frequency of an existing task.
*   **[`DROP SCHEDULE`]({% link _docs/references/sql-commands/utility/drop_schedule.md %})**: Removes a background task entirely.
