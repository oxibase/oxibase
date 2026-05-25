---
layout: default
title: ALTER TABLE
parent: Data Definition Language (DDL)
grand_parent: SQL Commands
---

# ALTER TABLE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("ALTER TABLE"),
        NonTerminal("table_name"),
        Choice(0, [
          Sequence([Keyword("ADD COLUMN"), NonTerminal("column_definition")]),
          Sequence([Keyword("DROP COLUMN"), NonTerminal("column_name")]),
          Sequence([Keyword("RENAME COLUMN"), NonTerminal("old_name"), Keyword("TO"), NonTerminal("new_name")]),
          Sequence([Keyword("RENAME TO"), NonTerminal("new_table_name")])
        ])
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Modifies an existing table.

#### Basic Syntax

```sql
ALTER TABLE table_name operation;
```

#### Operations

```sql
-- Add a column
ALTER TABLE users ADD COLUMN last_login TIMESTAMP;

-- Drop a column
ALTER TABLE users DROP COLUMN age;

-- Rename a column
ALTER TABLE users RENAME COLUMN username TO user_name;

-- Rename table
ALTER TABLE users RENAME TO customers;
```
