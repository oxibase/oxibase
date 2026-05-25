---
layout: default
title: CREATE SEQUENCE
parent: DDL
grand_parent: SQL Commands
---

# CREATE SEQUENCE

Creates a new sequence object to generate unique, monotonic numbers. Highly concurrent and avoids transaction conflicts.

#### Basic Syntax

```sql
CREATE SEQUENCE [IF NOT EXISTS] sequence_name
    [START WITH start_value]
    [INCREMENT BY increment_value]
    [MINVALUE min_value | NO MINVALUE]
    [MAXVALUE max_value | NO MAXVALUE]
    [CYCLE | NO CYCLE];
```

#### Examples

```sql
-- Simple sequence starting at 1
CREATE SEQUENCE my_seq;

-- Sequence starting at 1000 and incrementing by 5
CREATE SEQUENCE custom_seq START WITH 1000 INCREMENT BY 5;

-- A cyclical sequence
CREATE SEQUENCE loop_seq MINVALUE 1 MAXVALUE 3 CYCLE;
```
