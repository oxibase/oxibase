# Quickstart: SQL Sequences

## Overview
Oxibase supports standard SQL sequences, providing an atomic, highly concurrent way to generate unique numeric identifiers without causing transaction blocks.

## Creating a Sequence

Create a simple sequence (starts at 1, increments by 1):
```sql
CREATE SEQUENCE my_sequence;
```

Create a custom sequence with bounds and cycling:
```sql
CREATE SEQUENCE custom_seq
    START WITH 1000
    INCREMENT BY 5
    MINVALUE 1000
    MAXVALUE 9999
    CYCLE;
```

## Using Sequences

**Get the next value:**
```sql
SELECT NEXTVAL('my_sequence');
-- Output: 1
```

**Get the current value (in your active session):**
```sql
SELECT CURRVAL('my_sequence');
-- Output: 1
```
*Note: `CURRVAL` will fail if you haven't called `NEXTVAL` in your current session yet.*

**Override the sequence counter:**
```sql
SELECT SETVAL('my_sequence', 500);
SELECT NEXTVAL('my_sequence');
-- Output: 501
```

## Introspection
You can view all active sequences via the information schema:

```sql
SELECT * FROM information_schema.sequences;
```

## Managing Sequences

```sql
ALTER SEQUENCE my_sequence INCREMENT BY 10;
DROP SEQUENCE my_sequence;
```
