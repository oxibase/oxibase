---
layout: default
title: DROP SCHEDULE
parent: UTILITY
grand_parent: SQL Commands
---

# DROP SCHEDULE

Removes a job schedule from the database.

#### Basic Syntax

```sql
DROP SCHEDULE [IF EXISTS] schedule_name;
```

#### Examples

```sql
DROP SCHEDULE IF EXISTS nightly_cleanup;
```
