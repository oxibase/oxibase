---
layout: default
title: CREATE TABLE
parent: DDL
grand_parent: SQL Commands
---

# CREATE TABLE

<div id="rrdiagram"></div>
<script>
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("CREATE TABLE"),
        Optional(Sequence([Keyword("IF NOT EXISTS")])),
        NonTerminal("table_name"),
        Keyword("("),
        OneOrMore(
          Sequence([
            NonTerminal("column_name"),
            NonTerminal("data_type"),
            ZeroOrMore(NonTerminal("constraint"))
          ]),
          Keyword(",")
        ),
        Keyword(")")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Creates a new table.

#### Basic Syntax

```sql
CREATE TABLE [IF NOT EXISTS] table_name (
    column_name data_type [constraints...],
    column_name data_type [constraints...],
    ...
);
```

#### Data Types

| Type | Description |
|------|-------------|
| INTEGER | 64-bit signed integer |
| FLOAT | 64-bit floating point |
| TEXT | Variable-length string |
| BOOLEAN | true/false |
| TIMESTAMP | Date and time |
| JSON | JSON data |

#### Column Constraints

| Constraint | Description |
|------------|-------------|
| PRIMARY KEY | Unique identifier, cannot be NULL |
| NOT NULL | Column cannot contain NULL values |
| AUTO_INCREMENT | Automatically generates sequential values |

#### Examples

```sql
-- Basic table
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT,
    age INTEGER,
    created_at TIMESTAMP
);

-- With AUTO_INCREMENT
CREATE TABLE posts (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    title TEXT NOT NULL,
    content TEXT,
    author_id INTEGER
);

-- With IF NOT EXISTS
CREATE TABLE IF NOT EXISTS products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price FLOAT NOT NULL
);
```
