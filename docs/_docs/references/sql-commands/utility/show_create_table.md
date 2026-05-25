---
layout: default
title: SHOW CREATE TABLE
parent: Utility Commands
grand_parent: SQL Commands
---

# SHOW CREATE TABLE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("SHOW CREATE TABLE"),
        NonTerminal("table_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Shows the CREATE TABLE statement for a table.

```sql
SHOW CREATE TABLE table_name;
```
