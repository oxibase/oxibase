---
layout: default
title: Data Definition Language (DDL)
parent: SQL Commands
has_children: true
---

# Data Definition Language (DDL)

**Data Definition Language (DDL)** commands are used to define, alter, and manage the underlying structure (schema) of your database.

While DML is used to manipulate the *data* inside the database, DDL is used to build the containers that hold the data. This includes creating tables, setting up indexes for performance, defining views, and registering custom functions.

## Tables

Tables are the primary storage objects in a relational database.

*   **[`CREATE TABLE`]({% link _docs/references/sql-commands/ddl/create_table.md %})**: Defines a new table, its columns, data types, and constraints.
*   **[`CREATE TABLE AS SELECT`]({% link _docs/references/sql-commands/ddl/create_table_as_select.md %})**: Creates a new table and populates it dynamically using the results of a query.
*   **[`ALTER TABLE`]({% link _docs/references/sql-commands/ddl/alter_table.md %})**: Modifies the structure of an existing table (e.g., adding, dropping, or renaming columns).
*   **[`DROP TABLE`]({% link _docs/references/sql-commands/ddl/drop_table.md %})**: Permanently deletes a table and all the data it contains.

## Indexes

Indexes are background data structures that dramatically speed up data retrieval operations.

*   **[`CREATE INDEX`]({% link _docs/references/sql-commands/ddl/create_index.md %})**: Builds an index on one or more columns of a table.
*   **[`DROP INDEX`]({% link _docs/references/sql-commands/ddl/drop_index.md %})**: Removes an existing index.

## Views

Views are virtual tables representing the result of a stored query. They do not store data themselves.

*   **[`CREATE VIEW`]({% link _docs/references/sql-commands/ddl/create_view.md %})**: Defines a new reusable virtual table.
*   **[`DROP VIEW`]({% link _docs/references/sql-commands/ddl/drop_view.md %})**: Removes an existing view.

## Sequences

Sequences are specialized database objects used to generate unique, strictly increasing or decreasing numbers, typically used for primary keys.

*   **[`CREATE SEQUENCE`]({% link _docs/references/sql-commands/ddl/create_sequence.md %})**: Creates a new sequence generator.
*   **[`ALTER SEQUENCE`]({% link _docs/references/sql-commands/ddl/alter_sequence.md %})**: Modifies the properties (like the current value or increment step) of a sequence.
*   **[`DROP SEQUENCE`]({% link _docs/references/sql-commands/ddl/drop_sequence.md %})**: Removes a sequence.

## Functions

Because Oxibase embeds business logic directly into the database, you can define custom functions.

*   **[`CREATE FUNCTION`]({% link _docs/references/sql-commands/ddl/create_function.md %})**: Registers a User-Defined Function (UDF) written in a language like JavaScript or Python to be used within your SQL queries.
