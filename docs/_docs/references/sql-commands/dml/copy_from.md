---
layout: default
title: COPY FROM
parent: DML
grand_parent: SQL Commands
---

# COPY FROM

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("COPY"),
        NonTerminal("table_name"),
        Optional(Sequence([Keyword("("), OneOrMore(NonTerminal("column_name"), Keyword(",")), Keyword(")")])),
        Keyword("FROM"),
        NonTerminal("file_path"),
        Optional(Sequence([Keyword("WITH"), Keyword("("), OneOrMore(NonTerminal("option"), Keyword(",")), Keyword(")")]))
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

The COPY FROM statement bulk imports data from CSV or JSON files efficiently, bypassing standard row-by-row SQL parsing.

#### Basic Syntax

```sql
COPY table_name [(column1, column2, ...)] 
FROM 'file_path' 
[ WITH ( option [, ...] ) ]

-- Available options:
-- FORMAT format_name  (csv or json)
-- HEADER boolean      (for CSV only)
-- DELIMITER 'char'    (for CSV only, default ',')
-- NULL 'string'       (string representing null)
```

#### Examples

```sql
-- Load data from a CSV file
COPY employees FROM 'employees.csv' WITH (FORMAT CSV, HEADER true);

-- Load data from a JSON array or JSON Lines (JSONL) file
COPY events FROM 'events.json' WITH (FORMAT JSON);

-- Load data mapping only specific columns
COPY users (id, email) FROM 'users.csv' WITH (FORMAT CSV, NULL 'N/A', DELIMITER '|');
```
