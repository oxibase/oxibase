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
    let timestamp = now().to_string(); // Assuming a now() helper or string generation
    let sql = `INSERT INTO transaction_log (account_id, amount, type, timestamp) 
               VALUES (${acc_id}, ${amt}, "${trans_type}", "${timestamp}")`;
    execute(sql);
';
```

> **Note**: In Rhai, you can use backticks for template strings to easily embed variables.

## 3. Implementing Business Logic

Now, let's create a more complex procedure to handle fund transfers. This procedure needs to:
1. Check if the source account exists and has sufficient funds.
2. Deduct the amount from the source account.
3. Add the amount to the destination account.
4. Log both actions using our logical flow.

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
    
    // 3. Log Operations (Manual logging for this example)
    // In a real scenario, you might call another procedure or insert directly
    execute(`INSERT INTO transaction_log (account_id, amount, type) VALUES (${from_id}, ${-amount}, "TRANSFER_OUT")`);
    execute(`INSERT INTO transaction_log (account_id, amount, type) VALUES (${to_id}, ${amount}, "TRANSFER_IN")`);
';
```

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
```

## 5. Error Handling and Transactions

Procedures work seamlessly with transactions. If an error occurs (like "Insufficient funds"), the entire transaction is rolled back if you wrap the call in a transaction block.

```sql
BEGIN;
-- Try to transfer more than Alice has
CALL transfer(1, 2, 5000.0); 
-- This will throw an error: "Insufficient funds..."
COMMIT; -- The commit will not happen for the failed operations if the client handles the error correctly, 
        -- or the engine rolls back the statement.
```

## Summary

In this tutorial, you learned how to:
- Create stored procedures using `CREATE PROCEDURE`.
- Use the `rhai` language backend to write procedural logic.
- Execute SQL queries within procedures using the `execute()` function.
- Handle parameters and errors.
- Call procedures using `CALL`.

Stored procedures are a powerful way to keep your data logic close to your data, improving performance and consistency.
