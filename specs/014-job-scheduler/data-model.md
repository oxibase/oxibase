# Data Model: Job Scheduler & System Schema

This document details the new data structures, AST nodes, and system tables introduced for the Job Scheduler feature, alongside the migration of existing system tables.

## System Tables Migration

All existing metadata tables are migrated to the `system` schema, dropping the `_sys_` prefix.

| Old Table Name | New Table Name |
| :--- | :--- |
| `_sys_procedures` | `system.procedures` |
| `_sys_functions` | `system.functions` |
| `_sys_triggers` | `system.triggers` |
| `_sys_table_stats` | `system.table_stats` |
| `_sys_column_stats` | `system.column_stats` |

## New Job Scheduler Tables

The scheduler relies on two new tables in the `system` schema. These will be automatically initialized when the database starts.

### 1. `system.cron`

Stores the active and inactive job configurations.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | `INTEGER` | `PRIMARY KEY AUTO_INCREMENT` | Unique identifier for the job |
| `name` | `TEXT` | `UNIQUE NOT NULL` | Unique name of the job schedule |
| `schedule` | `TEXT` | `NOT NULL` | Standard cron expression (e.g., `0 0 * * *`) |
| `command` | `TEXT` | `NOT NULL` | The SQL command to execute (e.g., `CALL proc()`) |
| `active` | `BOOLEAN`| `DEFAULT true` | Whether the job should be executed |

### 2. `system.cron_runs`

Audit log of all job executions.

| Column | Type | Constraints | Description |
| :--- | :--- | :--- | :--- |
| `id` | `INTEGER` | `PRIMARY KEY AUTO_INCREMENT` | Unique identifier for the run |
| `job_id` | `INTEGER` | `NOT NULL` | Foreign key reference to `system.cron.id` |
| `status` | `TEXT` | `NOT NULL` | `'RUNNING'`, `'SUCCESS'`, or `'FAILED'` |
| `return_message`| `TEXT` | | Error message or execution result |
| `start_time` | `TIMESTAMP` | `NOT NULL` | When the execution started |
| `end_time` | `TIMESTAMP` | | When the execution finished |

## Abstract Syntax Tree (AST)

New nodes added to `src/parser/ast.rs`:

```rust
pub struct CreateScheduleStatement {
    pub name: String,
    pub cron_expr: String,
    pub command: String,
}

pub struct DropScheduleStatement {
    pub name: String,
}

pub struct AlterScheduleStatement {
    pub name: String,
    pub active: bool,
}
```

## Physical Execution (Executor)

The `Executor` will translate the AST nodes into underlying row operations on the `system.cron` table.

- `CreateScheduleStatement`: Evaluates `INSERT INTO system.cron (name, schedule, command) VALUES (...)`.
- `AlterScheduleStatement`: Evaluates `UPDATE system.cron SET active = ... WHERE name = ...`.
- `DropScheduleStatement`: Evaluates `DELETE FROM system.cron WHERE name = ...`.
