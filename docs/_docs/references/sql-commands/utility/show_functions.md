---
layout: default
title: SHOW FUNCTIONS
parent: Utility Commands
grand_parent: SQL Commands
---

# SHOW FUNCTIONS

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("SHOW"),
        Choice(0, [Keyword("FUNCTIONS"), Keyword("FUNCTION")])
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Lists all available SQL functions (scalar, aggregate, and window functions).

#### Basic Syntax

```sql
SHOW FUNCTIONS;
SHOW FUNCTION;  -- Also accepted
```

#### Example

```sql
SHOW FUNCTIONS;
```

Output:
```
name       type
---------  ---------
UPPER      SCALAR
COUNT      AGGREGATE
ROW_NUMBER WINDOW
...
```
