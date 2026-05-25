---
layout: default
title: DROP VIEW
parent: Data Definition Language (DDL)
grand_parent: SQL Commands
---

# DROP VIEW

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("DROP VIEW"),
        Optional(Sequence([Keyword("IF EXISTS")])),
        NonTerminal("view_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Removes a view.

#### Basic Syntax

```sql
DROP VIEW [IF EXISTS] view_name;
```

#### Example

```sql
DROP VIEW active_products;
DROP VIEW IF EXISTS old_report;
```
