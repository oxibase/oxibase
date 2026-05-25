---
layout: default
title: CREATE SCHEDULE
parent: UTILITY
grand_parent: SQL Commands
---

# CREATE SCHEDULE

Creates a new background job schedule that executes a stored procedure at specified intervals using standard Cron syntax.

#### Basic Syntax

```sql
CREATE SCHEDULE [IF NOT EXISTS] schedule_name 
    CRON 'cron_expression' 
    CALL procedure_name();
```

#### Examples

```sql
-- Run the cleanup procedure every night at midnight
CREATE SCHEDULE nightly_cleanup 
    CRON '0 0 0 * * *' 
    CALL cleanup_old_data();

-- Run every 5 minutes
CREATE SCHEDULE refresh_stats 
    CRON '0 */5 * * * *' 
    CALL update_statistics();
```
