---
layout: default
title: EXPLAIN
parent: Data Query Language (DQL)
grand_parent: SQL Commands
---

# EXPLAIN

Shows the query execution plan.

```sql
EXPLAIN SELECT * FROM users WHERE id = 1;
```

Output:
```
plan
----
SELECT
  Columns: *
  -> PK Lookup on users
       id = 1
```
