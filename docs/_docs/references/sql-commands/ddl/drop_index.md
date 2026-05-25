---
layout: default
title: DROP INDEX
parent: DDL
grand_parent: SQL Commands
---

# DROP INDEX

Removes an index from a table.

#### Basic Syntax

```sql
DROP INDEX [IF EXISTS] index_name ON table_name;
```

#### Example

```sql
DROP INDEX idx_user_email ON users;
DROP INDEX IF EXISTS idx_old ON products;
```
