---
layout: default
title: ALTER SCHEDULE
parent: Utility Commands
grand_parent: SQL Commands
---

# ALTER SCHEDULE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("ALTER SCHEDULE"),
        Optional(Sequence([Keyword("IF EXISTS")])),
        NonTerminal("schedule_name"),
        Choice(0, [
          Sequence([Keyword("CRON"), NonTerminal("new_cron_expression")]),
          Keyword("ACTIVE"),
          Keyword("INACTIVE")
        ])
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Modifies an existing job schedule to change its cron expression or toggle its active state.

#### Basic Syntax

```sql
ALTER SCHEDULE [IF EXISTS] schedule_name 
    { CRON 'new_cron_expression' | { ACTIVE | INACTIVE } };
```

#### Examples

```sql
-- Pause a schedule
ALTER SCHEDULE nightly_cleanup INACTIVE;

-- Resume a schedule
ALTER SCHEDULE nightly_cleanup ACTIVE;

-- Change the execution interval
ALTER SCHEDULE refresh_stats CRON '0 0 * * * *';
```
