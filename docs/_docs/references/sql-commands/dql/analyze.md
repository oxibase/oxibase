---
layout: default
title: ANALYZE
parent: DQL
grand_parent: SQL Commands
---

# ANALYZE

Collects statistics for the query optimizer.

```sql
-- Analyze a specific table
ANALYZE table_name;
```

Statistics are used by the cost-based optimizer to choose efficient query plans.
