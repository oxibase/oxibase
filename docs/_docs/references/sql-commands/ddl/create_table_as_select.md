---
layout: default
title: CREATE TABLE AS SELECT
parent: Data Definition Language (DDL)
grand_parent: SQL Commands
---

# CREATE TABLE AS SELECT

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("CREATE TABLE"),
        NonTerminal("table_name"),
        Keyword("AS"),
        Keyword("SELECT"),
        NonTerminal("...")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Creates a new table from the result of a SELECT query.

#### Basic Syntax

```sql
CREATE TABLE table_name AS SELECT ...;
```

#### Example

```sql
-- Create table from query result
CREATE TABLE active_users AS
SELECT id, name, email
FROM users
WHERE last_login > '2024-01-01';

-- Create summary table
CREATE TABLE daily_sales AS
SELECT DATE_TRUNC('day', order_date) as day,
       SUM(amount) as total
FROM orders
GROUP BY DATE_TRUNC('day', order_date);
```
