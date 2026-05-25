---
layout: default
title: DROP SCHEDULE
parent: Utility Commands
grand_parent: SQL Commands
---

# DROP SCHEDULE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("DROP SCHEDULE"),
        Optional(Sequence([Keyword("IF EXISTS")])),
        NonTerminal("schedule_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Removes a job schedule from the database.

#### Basic Syntax

```sql
DROP SCHEDULE [IF EXISTS] schedule_name;
```

#### Examples

```sql
DROP SCHEDULE IF EXISTS nightly_cleanup;
```
