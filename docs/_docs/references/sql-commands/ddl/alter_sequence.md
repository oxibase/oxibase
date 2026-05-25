---
layout: default
title: ALTER SEQUENCE
parent: DDL
grand_parent: SQL Commands
---

# ALTER SEQUENCE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("ALTER SEQUENCE"),
        Optional(Keyword("IF EXISTS")),
        NonTerminal("sequence_name"),
        Optional(Sequence([Keyword("RESTART WITH"), NonTerminal("restart_value")])),
        Optional(Sequence([Keyword("INCREMENT BY"), NonTerminal("increment_value")])),
        Optional(Choice(0, [Sequence([Keyword("MINVALUE"), NonTerminal("min_value")]), Keyword("NO MINVALUE")])),
        Optional(Choice(0, [Sequence([Keyword("MAXVALUE"), NonTerminal("max_value")]), Keyword("NO MAXVALUE")])),
        Optional(Choice(0, [Keyword("CYCLE"), Keyword("NO CYCLE")]))
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Modifies the properties of an existing sequence.

#### Basic Syntax

```sql
ALTER SEQUENCE [IF EXISTS] sequence_name
    [RESTART WITH restart_value]
    [INCREMENT BY increment_value]
    [MINVALUE min_value | NO MINVALUE]
    [MAXVALUE max_value | NO MAXVALUE]
    [CYCLE | NO CYCLE];
```

#### Examples

```sql
ALTER SEQUENCE my_seq RESTART WITH 50 INCREMENT BY 10;
```
