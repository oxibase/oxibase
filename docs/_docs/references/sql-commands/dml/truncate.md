---
layout: default
title: TRUNCATE
parent: DML
grand_parent: SQL Commands
---

# TRUNCATE

<div id="rrdiagram"></div>
<script>
  document.addEventListener("DOMContentLoaded", function() {
    var diagram = Diagram([
      Sequence([
        Keyword("TRUNCATE TABLE"),
        NonTerminal("table_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  });
</script>

The TRUNCATE statement removes all rows from a table efficiently.

#### Basic Syntax

```sql
TRUNCATE TABLE table_name;
```

#### Example

```sql
-- Remove all rows (faster than DELETE)
TRUNCATE TABLE logs;
```

Note: TRUNCATE is faster than DELETE for removing all rows because it doesn't log individual row deletions.
