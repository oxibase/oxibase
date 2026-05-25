---
layout: default
title: SHOW INDEXES
parent: Utility Commands
grand_parent: SQL Commands
---

# SHOW INDEXES

<div id="rrdiagram"></div>
<script>
  document.addEventListener("DOMContentLoaded", function() {
    var diagram = Diagram([
      Sequence([
        Keyword("SHOW INDEXES FROM"),
        NonTerminal("table_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  });
</script>

Lists all indexes for a table.

```sql
SHOW INDEXES FROM table_name;
```
