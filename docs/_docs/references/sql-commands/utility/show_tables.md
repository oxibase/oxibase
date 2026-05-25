---
layout: default
title: SHOW TABLES
parent: Utility Commands
grand_parent: SQL Commands
---

# SHOW TABLES

<div id="rrdiagram"></div>
<script>
  document.addEventListener("DOMContentLoaded", function() {
    var diagram = Diagram([
      Sequence([
        Keyword("SHOW TABLES")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  });
</script>

Lists all tables in the database.

```sql
SHOW TABLES;
```
