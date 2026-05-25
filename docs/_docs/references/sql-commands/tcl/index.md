---
layout: default
title: Transaction Control Language (TCL)
parent: SQL Commands
has_children: true
---

# Transaction Control Language (TCL)

**Transaction Control Language (TCL)** commands are used to manage transactions within the database. 

A **transaction** is a sequence of one or more SQL operations treated as a single, indivisible unit of work. TCL ensures that either *all* the operations within the transaction succeed and are saved permanently, or *none* of them take effect if an error occurs. This is the foundation of data integrity (the "A" for Atomicity in ACID).

Oxibase utilizes a Multi-Version Concurrency Control (MVCC) engine. This means that when you start a transaction, you get a consistent snapshot of the database at that exact moment in time, entirely isolated from the changes other concurrent users might be making.

## Core Commands

All transaction control flows are managed through a few key commands, which are fully detailed on the [**Transaction Control**]({% link _docs/references/sql-commands/tcl/transaction.md %}) reference page:

*   **`BEGIN TRANSACTION`**: Starts a new isolated transaction block.
*   **`COMMIT`**: Successfully concludes the transaction, making all of your data modifications permanently visible to other users.
*   **`ROLLBACK`**: Aborts the transaction, undoing all modifications made since the `BEGIN` statement.
*   **`SAVEPOINT`**: Creates a checkpoint within a large transaction, allowing you to partially roll back to a specific state without aborting the entire transaction.
