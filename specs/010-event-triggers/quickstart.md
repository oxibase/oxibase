# Quickstart: Event Triggers

This guide demonstrates how to create your first validation and audit triggers in Oxibase using the default Rhai scripting backend.

## 1. Create a Validation Trigger (`BEFORE INSERT`)

Let's assume you have an `accounts` table. You want to ensure no account can be created with a negative balance.

```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    owner_name TEXT,
    balance FLOAT
);

-- Define the trigger
CREATE TRIGGER ensure_positive_balance
    BEFORE INSERT ON accounts
    FOR EACH ROW
    LANGUAGE rhai
AS $$
    if NEW.balance < 0.0 {
        throw "Account balance cannot be negative upon creation!";
    }
$$;
```

If you try to insert invalid data, the transaction will immediately abort:
```sql
-- This succeeds
INSERT INTO accounts (id, owner_name, balance) VALUES (1, 'Alice', 100.0);

-- This throws the error and the insert is aborted
INSERT INTO accounts (id, owner_name, balance) VALUES (2, 'Bob', -50.0);
```

## 2. Create a Data Transformation Trigger (`BEFORE UPDATE`)

You can modify the `NEW` row before it is saved to the database.

```sql
CREATE TRIGGER normalize_owner_name
    BEFORE UPDATE ON accounts
    FOR EACH ROW
    LANGUAGE rhai
AS $$
    // Force uppercase
    NEW.owner_name = NEW.owner_name.to_upper();
$$;
```

## 3. Create an Audit Trigger (`AFTER UPDATE`)

Let's log changes to account balances into an `audit_log` table.

```sql
CREATE TABLE audit_log (
    account_id INTEGER,
    old_balance FLOAT,
    new_balance FLOAT,
    changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER log_balance_changes
    AFTER UPDATE ON accounts
    FOR EACH ROW
    LANGUAGE rhai
AS $$
    // Only log if the balance actually changed
    if OLD.balance != NEW.balance {
        let stmt = "INSERT INTO audit_log (account_id, old_balance, new_balance) VALUES (?, ?, ?)";
        execute(stmt, OLD.id, OLD.balance, NEW.balance);
    }
$$;
```

## 4. Dropping Triggers

To remove a trigger:
```sql
DROP TRIGGER IF EXISTS ensure_positive_balance ON accounts;
```
