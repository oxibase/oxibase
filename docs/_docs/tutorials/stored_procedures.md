---
layout: default
title: "Tutorial: Stored Procedures"
parent: Tutorials
nav_order: 4
---

# Tutorial: Working with Stored Procedures

This tutorial guides you through creating and using stored procedures in Oxibase using the Rhai scripting language. Stored procedures allow you to encapsulate complex logic and database operations on the server side.

## Prerequisites

- A running instance of Oxibase.
- Basic knowledge of SQL.

## 1. Setting Up the Environment

First, let's create some tables to work with. We'll simulate a simple banking system with accounts and a transaction log.

```sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    balance FLOAT NOT NULL
);

CREATE TABLE transaction_log (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    account_id INTEGER,
    amount FLOAT,
    type TEXT,
    timestamp TEXT
);

INSERT INTO accounts VALUES (1, 'Alice', 1000.0);
INSERT INTO accounts VALUES (2, 'Bob', 500.0);
```

## 2. Creating Your First Procedure

Let's create a simple procedure to log transactions. This procedure will take an account ID, amount, and transaction type, and insert a record into the `transaction_log` table.

```sql
CREATE PROCEDURE log_transaction(acc_id INTEGER, amt FLOAT, trans_type TEXT)
LANGUAGE rhai AS '
    let timestamp = "2023-01-01T12:00:00Z"; // For demo purposes, using a fixed timestamp
    execute(`INSERT INTO transaction_log (account_id, amount, type, timestamp) VALUES (${acc_id}, ${amt}, ''${trans_type}'', ''${timestamp}'')`);
';
```

> **Note**: Rhai supports string interpolation with backticks, and you can execute SQL queries using the built-in `execute()` function. Note the double single quotes (`''`) for escaping single quotes within SQL strings.

## 3. Implementing Business Logic

Now, let's create a more complex procedure to handle fund transfers. This procedure needs to:
1. Check if the source account exists and has sufficient funds.
2. Deduct the amount from the source account.
3. Add the amount to the destination account.
4. Log both actions by calling our `log_transaction` procedure.

```sql
CREATE PROCEDURE transfer(from_id INTEGER, to_id INTEGER, amount FLOAT)
LANGUAGE rhai AS '
    // 1. Validation
    let src_rows = execute(`SELECT balance FROM accounts WHERE id = ${from_id}`);
    if src_rows.len() == 0 {
        throw "Source account not found";
    }

    let dest_rows = execute(`SELECT id FROM accounts WHERE id = ${to_id}`);
    if dest_rows.len() == 0 {
        throw "Destination account not found";
    }

    let balance = src_rows[0].balance;
    if balance < amount {
        throw `Insufficient funds: Balance is ${balance}, required ${amount}`;
    }

    // 2. Perform Transfer
    execute(`UPDATE accounts SET balance = balance - ${amount} WHERE id = ${from_id}`);
    execute(`UPDATE accounts SET balance = balance + ${amount} WHERE id = ${to_id}`);

    // 3. Log Transaction
    execute(`CALL log_transaction(${from_id}, ${amount}, ''debit'')`);
    execute(`CALL log_transaction(${to_id}, ${amount}, ''credit'')`);
';
```

> **Note**: Procedures can call other procedures using `CALL` statements within the `execute()` function. This allows you to build complex logic by composing smaller procedures.

## 4. Executing Procedures

Now let's test our `transfer` procedure.

```sql
-- Check initial balances
SELECT * FROM accounts;

-- Execute the transfer
CALL transfer(1, 2, 200.0);

-- Check balances after transfer
SELECT * FROM accounts;
-- Alice should have 800.0, Bob should have 700.0

-- Check the log
SELECT * FROM transaction_log;
-- Should show two entries: debit from Alice and credit to Bob
```

> **Note**: Procedures are automatically persisted to the database and will be available after restarting Oxibase. You can inspect all stored procedures by querying the system table:
>
> ```sql
> SELECT schema, name, language FROM _sys_procedures;
> ```

## 5. Error Handling and Transactions

Procedures work seamlessly with transactions. If an error occurs (like "Insufficient funds"), the entire transaction is rolled back if you wrap the call in a transaction block.

```sql
BEGIN;
-- Try to transfer more than Alice has
CALL transfer(1, 2, 5000.0);
-- This will throw an error: "Insufficient funds..."
COMMIT; -- The transaction will be rolled back due to the error
```

## 6. Managing Procedures

You can also drop procedures when they're no longer needed:

```sql
-- Drop a specific procedure
DROP PROCEDURE transfer;

-- Drop a procedure if it exists
DROP PROCEDURE IF EXISTS log_transaction;
```

## 7. Advanced Features

### Procedure Parameters

Procedures support various parameter types:
- `INTEGER` for integer values
- `FLOAT` for floating-point numbers
- `TEXT` for string values
- `BOOLEAN` for boolean values

### Return Values

Currently, procedures don't return values directly, but you can use `execute()` to run SELECT queries and work with the results within the procedure.

## Summary

In this tutorial, you learned how to:
- Create stored procedures using `CREATE PROCEDURE` with the Rhai scripting language.
- Use the `execute()` function to run SQL queries within procedures.
- Handle parameters, validation, and error handling in Rhai scripts.
- Call procedures from other procedures to build complex logic.
- Execute procedures using `CALL` statements.
- Drop procedures using `DROP PROCEDURE`.
- Work with procedures in transaction contexts.

Stored procedures are automatically persisted and will survive database restarts. They provide a powerful way to encapsulate business logic on the database server, improving both performance and data consistency.
