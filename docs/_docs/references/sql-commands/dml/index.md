---
layout: default
title: Data Manipulation Language (DML)
parent: SQL Commands
has_children: true
---

# Data Manipulation Language (DML)

**Data Manipulation Language (DML)** is the subset of SQL used to add, modify, or delete the actual data stored within your database tables. 

Unlike DDL (which alters the structure of the database), DML operations only affect the rows of data inside those structures. In Oxibase, all DML operations are fully ACID-compliant and respect Multi-Version Concurrency Control (MVCC), meaning your data remains consistent even when multiple users are reading and writing simultaneously.

## Core Operations

*   **[`INSERT`]({% link _docs/references/sql-commands/dml/insert.md %})**: Adds new rows of data into a table. Supports single-row, multi-row, and upsert (`ON DUPLICATE KEY UPDATE`) operations.
*   **[`UPDATE`]({% link _docs/references/sql-commands/dml/update.md %})**: Modifies existing rows of data in a table based on a condition.
*   **[`DELETE`]({% link _docs/references/sql-commands/dml/delete.md %})**: Removes specific rows of data from a table based on a condition.

## Bulk Operations

When dealing with large volumes of data, standard row-by-row DML operations can be slow. Oxibase provides optimized commands for these scenarios:

*   **[`COPY FROM`]({% link _docs/references/sql-commands/dml/copy_from.md %})**: The fastest and recommended way to bulk-import massive amounts of data from CSV or JSON files into a table.
*   **[`TRUNCATE`]({% link _docs/references/sql-commands/dml/truncate.md %})**: Instantly removes all rows from a table. This is significantly faster than a `DELETE` statement without a `WHERE` clause because it reclaims the storage immediately without logging individual row deletions.
