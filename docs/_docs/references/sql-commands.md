---
layout: default
title: SQL Commands
parent: References
nav_order: 1
has_children: true
---

# SQL Commands

This section provides a comprehensive reference to the SQL commands supported by Oxibase. 

Oxibase is a fully-featured relational database that implements a modern SQL dialect. To make it easier to find the command you need, the documentation is divided into standard SQL categories based on their primary purpose.

If you are new to SQL, here is a quick overview of how commands are categorized:

*   **[Data Query Language (DQL)]({% link _docs/references/sql-commands/dql/index.md %})**: Commands used exclusively to ask the database questions and retrieve data without changing it.
*   **[Data Manipulation Language (DML)]({% link _docs/references/sql-commands/dml/index.md %})**: Commands used to add, modify, or remove the actual data stored inside your tables.
*   **[Data Definition Language (DDL)]({% link _docs/references/sql-commands/ddl/index.md %})**: Commands used to define the structure of your database—creating or modifying tables, indexes, views, and other schema objects.
*   **[Transaction Control Language (TCL)]({% link _docs/references/sql-commands/tcl/index.md %})**: Commands used to manage "transactions," ensuring that a group of database operations either completely succeed or completely fail together safely.
*   **[Utility Commands]({% link _docs/references/sql-commands/utility/index.md %})**: Specialized commands for inspecting database metadata (like seeing what tables exist) and managing background jobs.
*   **[PRAGMA Commands]({% link _docs/references/sql-commands/pragma/index.md %})**: Oxibase-specific commands to configure underlying engine settings and database behaviors.
