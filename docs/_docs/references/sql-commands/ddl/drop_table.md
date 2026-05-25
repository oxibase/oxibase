---
layout: default
title: DROP TABLE
parent: Data Definition Language (DDL)
grand_parent: SQL Commands
---

# DROP TABLE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("DROP TABLE"),
        Optional(Keyword("IF EXISTS")),
        NonTerminal("table_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Removes a table and all its data.

#### Basic Syntax

```sql
DROP TABLE [IF EXISTS] table_name;
```

#### Examples

```sql
DROP TABLE temporary_data;
DROP TABLE IF EXISTS old_logs;
```
