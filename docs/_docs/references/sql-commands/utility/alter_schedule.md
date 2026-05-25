---
layout: default
title: ALTER SCHEDULE
parent: UTILITY
grand_parent: SQL Commands
---

# ALTER SCHEDULE

Modifies an existing job schedule to change its cron expression or toggle its active state.

#### Basic Syntax

```sql
ALTER SCHEDULE [IF EXISTS] schedule_name 
    { CRON 'new_cron_expression' | { ACTIVE | INACTIVE } };
```

#### Examples

```sql
-- Pause a schedule
ALTER SCHEDULE nightly_cleanup INACTIVE;

-- Resume a schedule
ALTER SCHEDULE nightly_cleanup ACTIVE;

-- Change the execution interval
ALTER SCHEDULE refresh_stats CRON '0 0 * * * *';
```
