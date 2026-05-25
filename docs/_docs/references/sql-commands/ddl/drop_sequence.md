---
layout: default
title: DROP SEQUENCE
parent: DDL
grand_parent: SQL Commands
---

# DROP SEQUENCE

<div id="rrdiagram"></div>
<script>
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("DROP SEQUENCE"),
        Optional(Sequence([Keyword("IF EXISTS")])),
        NonTerminal("sequence_name")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Removes a sequence from the database.

#### Basic Syntax

```sql
DROP SEQUENCE [IF EXISTS] sequence_name;
```

#### Examples

```sql
DROP SEQUENCE IF EXISTS my_seq;
```
