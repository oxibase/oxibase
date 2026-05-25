---
layout: default
title: Transaction Control
parent: Transaction Control Language (TCL)
grand_parent: SQL Commands
---

# Transaction Control

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Choice(0, [
        Sequence([Choice(0, [Keyword("BEGIN"), Keyword("BEGIN TRANSACTION")])]),
        Keyword("COMMIT"),
        Keyword("ROLLBACK"),
        Sequence([Keyword("SAVEPOINT"), NonTerminal("savepoint_name")]),
        Sequence([Keyword("ROLLBACK TO SAVEPOINT"), NonTerminal("savepoint_name")]),
        Sequence([Keyword("RELEASE SAVEPOINT"), NonTerminal("savepoint_name")])
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

## BEGIN TRANSACTION

Starts a new transaction.

```sql
BEGIN TRANSACTION;
-- or simply
BEGIN;
```

## COMMIT

Commits the current transaction, making all changes permanent.

```sql
COMMIT;
```

## ROLLBACK

Rolls back the current transaction, discarding all changes.

```sql
ROLLBACK;
```

## SAVEPOINT

Creates a savepoint within a transaction for partial rollback.

```sql
-- Create a savepoint
SAVEPOINT savepoint_name;

-- Rollback to a savepoint
ROLLBACK TO SAVEPOINT savepoint_name;

-- Release a savepoint
RELEASE SAVEPOINT savepoint_name;
```

#### Example

```sql
BEGIN TRANSACTION;

INSERT INTO accounts (id, balance) VALUES (1, 1000);
SAVEPOINT after_insert;

UPDATE accounts SET balance = 500 WHERE id = 1;
-- Oops, wrong update
ROLLBACK TO SAVEPOINT after_insert;

-- Continue with correct update
UPDATE accounts SET balance = 900 WHERE id = 1;
COMMIT;
```

See [Savepoints]({% link _docs/references/sql-features/savepoints.md %}) for detailed documentation.
