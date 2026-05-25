---
layout: default
title: PRAGMA Commands
parent: SQL Commands
has_children: true
---

# PRAGMA Commands

**PRAGMA** commands are special, implementation-specific instructions used to modify the behavior of the database engine at runtime.

Unlike standard SQL commands which are portable across different database systems, `PRAGMA` statements are unique to Oxibase. They are used exclusively for low-level configuration, performance tuning, and triggering internal engine operations that are not part of standard data management.

## Available Configuration

*   **[`PRAGMA`]({% link _docs/references/sql-commands/pragma/pragma.md %})**: The master command for setting or querying engine configuration variables. This includes tuning Write-Ahead Log (WAL) behaviors, adjusting snapshot intervals, and forcing manual persistence checkpoints.
