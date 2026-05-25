---
layout: default
title: Data Query Language (DQL)
parent: SQL Commands
has_children: true
---

# Data Query Language (DQL)

**Data Query Language (DQL)** consists of commands used to retrieve data from the database. These operations are strictly read-only and do not modify the underlying data or the database schema.

In almost all relational databases, including Oxibase, the primary and most powerful DQL command is `SELECT`.

## Core Commands

*   **[`SELECT`]({% link _docs/references/sql-commands/dql/select.md %})**: The foundational command for querying data. It supports filtering (`WHERE`), grouping (`GROUP BY`), sorting (`ORDER BY`), joins, subqueries, and advanced analytical features like Common Table Expressions (CTEs) and Window Functions.

## Query Analysis

Oxibase provides built-in tools to help you understand how your DQL queries are being executed behind the scenes. These are critical for optimizing performance.

*   **[`EXPLAIN`]({% link _docs/references/sql-commands/dql/explain.md %})**: Shows the planned execution strategy for a query without actually running it.
*   **[`EXPLAIN ANALYZE`]({% link _docs/references/sql-commands/dql/explain_analyze.md %})**: Executes the query and returns the plan alongside actual runtime statistics (time taken, rows processed).
*   **[`ANALYZE`]({% link _docs/references/sql-commands/dql/analyze.md %})**: Collects statistics about the contents of tables to help the query optimizer make better routing decisions.
