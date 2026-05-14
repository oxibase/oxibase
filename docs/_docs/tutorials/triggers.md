---
title: "Event Triggers"
layout: default
parent: Tutorials
has_children: false
nav_order: 4
---

# Using Event Triggers

Event triggers allow you to execute custom procedural logic automatically whenever data in a table is inserted, updated, or deleted. By hooking directly into the database engine, triggers ensure that critical business rules are enforced universally, regardless of which application or user issued the query.

Because Oxibase is a polyglot database, you can write these triggers in Rhai, JavaScript, or Python.

## 1. Data Validation (`BEFORE INSERT` / `BEFORE UPDATE`)

The most common use case for triggers is to prevent bad data from entering your database. A `BEFORE` trigger fires before the storage engine processes the row. If your script throws an error, the operation aborts immediately.

### Example: Enforcing Positive Balances

Suppose you have an `accounts` table:

```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    owner_name TEXT,
    balance FLOAT
);
```

You can define a trigger in **Rhai** to ensure a balance is never negative upon creation or modification:

```sql
CREATE TRIGGER ensure_positive_balance
    BEFORE INSERT ON accounts
    FOR EACH ROW
    LANGUAGE rhai
AS '
    if oxibase.ctx["new"].balance < 0.0 {
        throw "Account balance cannot be negative!";
    }
';
```

If you try to insert invalid data, the transaction aborts:
```sql
-- This succeeds
INSERT INTO accounts (id, owner_name, balance) VALUES (1, 'Alice', 100.0);

-- This throws the error and the insert is rolled back
INSERT INTO accounts (id, owner_name, balance) VALUES (2, 'Bob', -50.0);

-- Verify that Bob was not inserted
SELECT * FROM accounts;
```

## 2. Data Transformation (`BEFORE UPDATE`)

You can modify the `oxibase.ctx.new` row representation inside a `BEFORE` trigger. The mutated data is what gets saved to the disk.

### Example: Normalizing Strings with JavaScript

*(Note: Requires Oxibase to be compiled with the `js` feature)*

```sql
CREATE TRIGGER normalize_owner_name
    BEFORE UPDATE ON accounts
    FOR EACH ROW
    LANGUAGE js
AS '
    // Force the owner name to be uppercase before saving
    oxibase.ctx.new.owner_name = oxibase.ctx.new.owner_name.toUpperCase();
';

-- Trigger the update
UPDATE accounts SET owner_name = 'alice smith' WHERE id = 1;

-- Verify the transformation
SELECT * FROM accounts WHERE id = 1;
```

## 3. Audit Logging (`AFTER UPDATE` / `AFTER DELETE`)

`AFTER` triggers execute once the data is safely persisted but before the transaction completes. They are perfect for generating audit trails by comparing the `oxibase.ctx.old` state of the row to the `oxibase.ctx.new` state.

### Example: Price Change Tracker in Python

*(Note: Requires Oxibase to be compiled with the `python` feature)*

First, create an audit table:
```sql
CREATE TABLE audit_log (
    account_id INTEGER,
    old_balance FLOAT,
    new_balance FLOAT,
    changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

Next, create a trigger that checks if the balance has changed, and if so, writes a record to the `audit_log`:

```sql
CREATE TRIGGER log_balance_changes
    AFTER UPDATE ON accounts
    FOR EACH ROW
    LANGUAGE python
AS '
import oxibase

# Only log if the balance actually changed
if oxibase.ctx.old["balance"] != oxibase.ctx.new["balance"]:
    # Use oxibase.execute to run DML side-effects
    stmt = "INSERT INTO audit_log (account_id, old_balance, new_balance) VALUES (" + str(oxibase.ctx.old["id"]) + ", " + str(oxibase.ctx.old["balance"]) + ", " + str(oxibase.ctx.new["balance"]) + ")"
    oxibase.execute(stmt)
';

-- Trigger the update
UPDATE accounts SET balance = 200.0 WHERE id = 1;

-- View the audit log
SELECT * FROM audit_log;
```

## Dropping Triggers

To remove a trigger when it is no longer needed, use the `DROP TRIGGER` command:

```sql
DROP TRIGGER IF EXISTS log_balance_changes ON accounts;
```
