---
layout: default
title: CREATE SEQUENCE
parent: DDL
grand_parent: SQL Commands
---

# CREATE SEQUENCE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("CREATE SEQUENCE"),
        Optional(Sequence([Keyword("IF NOT EXISTS")])),
        NonTerminal("sequence_name"),
        Optional(Sequence([Keyword("START WITH"), NonTerminal("start_value")])),
        Optional(Sequence([Keyword("INCREMENT BY"), NonTerminal("increment_value")])),
        Optional(Choice(0, [Sequence([Keyword("MINVALUE"), NonTerminal("min_value")]), Keyword("NO MINVALUE")])),
        Optional(Choice(0, [Sequence([Keyword("MAXVALUE"), NonTerminal("max_value")]), Keyword("NO MAXVALUE")])),
        Optional(Choice(0, [Keyword("CYCLE"), Keyword("NO CYCLE")]))
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Creates a new sequence object to generate unique, monotonic numbers. Highly concurrent and avoids transaction conflicts.

#### Basic Syntax

```sql
CREATE SEQUENCE [IF NOT EXISTS] sequence_name
    [START WITH start_value]
    [INCREMENT BY increment_value]
    [MINVALUE min_value | NO MINVALUE]
    [MAXVALUE max_value | NO MAXVALUE]
    [CYCLE | NO CYCLE];
```

#### Examples

```sql
-- Simple sequence starting at 1
CREATE SEQUENCE my_seq;

-- Sequence starting at 1000 and incrementing by 5
CREATE SEQUENCE custom_seq START WITH 1000 INCREMENT BY 5;

-- A cyclical sequence
CREATE SEQUENCE loop_seq MINVALUE 1 MAXVALUE 3 CYCLE;
```
