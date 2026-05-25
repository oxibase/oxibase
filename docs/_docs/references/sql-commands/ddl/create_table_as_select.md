---
layout: default
title: CREATE TABLE AS SELECT
parent: DDL
grand_parent: SQL Commands
---

# CREATE TABLE AS SELECT

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
