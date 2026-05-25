---
layout: default
title: ALTER SEQUENCE
parent: DDL
grand_parent: SQL Commands
---

# ALTER SEQUENCE

Modifies the properties of an existing sequence.

#### Basic Syntax

```sql
ALTER SEQUENCE [IF EXISTS] sequence_name
    [RESTART WITH restart_value]
    [INCREMENT BY increment_value]
    [MINVALUE min_value | NO MINVALUE]
    [MAXVALUE max_value | NO MAXVALUE]
    [CYCLE | NO CYCLE];
```

#### Examples

```sql
ALTER SEQUENCE my_seq RESTART WITH 50 INCREMENT BY 10;
```
