---
layout: default
title: DELETE
parent: Data Manipulation Language (DML)
grand_parent: SQL Commands
---

# DELETE

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("DELETE FROM"),
        NonTerminal("table_name"),
        Optional(Sequence([Keyword("WHERE"), NonTerminal("condition")])),
        Optional(Sequence([Keyword("RETURNING"), Choice(0, [Keyword("*"), OneOrMore(NonTerminal("column_name"), Keyword(","))])]))
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

The DELETE statement removes rows from a table.

#### Basic Syntax

```sql
DELETE FROM table_name
[WHERE condition]
[RETURNING *|column1, column2, ...];
```

#### Examples

```sql
-- Delete single row
DELETE FROM customers WHERE id = 1;

-- Delete multiple rows
DELETE FROM orders WHERE order_date < '2023-01-01';

-- Delete all rows
DELETE FROM temporary_logs;

-- With RETURNING clause
DELETE FROM users WHERE inactive = true
RETURNING id, name;

-- Delete using subquery
DELETE FROM orders
WHERE customer_id IN (
    SELECT id FROM customers WHERE status = 'inactive'
);
```
