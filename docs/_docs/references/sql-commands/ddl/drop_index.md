---
layout: default
title: DROP INDEX
parent: DDL
grand_parent: SQL Commands
---

# DROP INDEX

<div id="rrdiagram"></div>
<script>
  document.addEventListener("DOMContentLoaded", function() {
    var diagram = Diagram([
      Sequence([
        Keyword("DROP INDEX"),
        Optional(Sequence([Keyword("IF EXISTS")])),
        NonTerminal("index_name"),
        Keyword("ON"),
        NonTerminal("table_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  });
</script>

Removes an index from a table.

#### Basic Syntax

```sql
DROP INDEX [IF EXISTS] index_name ON table_name;
```

#### Example

```sql
DROP INDEX idx_user_email ON users;
DROP INDEX IF EXISTS idx_old ON products;
```
