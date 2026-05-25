---
layout: default
title: EXPLAIN ANALYZE
parent: Data Query Language (DQL)
grand_parent: SQL Commands
---

# EXPLAIN ANALYZE

Shows the execution plan with actual runtime statistics.

```sql
EXPLAIN ANALYZE SELECT * FROM products WHERE price > 100;
```

Output:
```
plan
----
SELECT (actual time=1.2ms, rows=150)
  Columns: *
  -> Seq Scan on products (actual rows=150)
       Filter: (price > 100)
```

See [EXPLAIN]({% link _docs/how-to/explain.md %}) for detailed documentation.
