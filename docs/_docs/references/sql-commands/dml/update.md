---
layout: default
title: UPDATE
parent: DML
grand_parent: SQL Commands
---

# UPDATE

<div id="rrdiagram"></div>
<script>
  document.addEventListener("DOMContentLoaded", function() {
    var diagram = Diagram([
      Sequence([
        Keyword("UPDATE"),
        NonTerminal("table_name"),
        Keyword("SET"),
        OneOrMore(Sequence([NonTerminal("column_name"), Keyword("="), NonTerminal("value")]), Keyword(",")),
        Optional(Sequence([Keyword("WHERE"), NonTerminal("condition")])),
        Optional(Sequence([Keyword("RETURNING"), Choice(0, [Keyword("*"), OneOrMore(NonTerminal("column_name"), Keyword(","))])]))
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  });
</script>

The UPDATE statement modifies existing data.

#### Basic Syntax

```sql
UPDATE table_name
SET column1 = value1, column2 = value2, ...
[WHERE condition]
[RETURNING *|column1, column2, ...];
```

#### Examples

```sql
-- Update single row
UPDATE customers
SET email = 'new.email@example.com'
WHERE id = 1;

-- Update multiple rows
UPDATE products
SET price = price * 1.1
WHERE category = 'Electronics';

-- Update all rows
UPDATE settings
SET last_updated = NOW();

-- With RETURNING clause
UPDATE accounts
SET balance = balance + 100
WHERE id = 1
RETURNING id, balance;

-- Update using subquery
UPDATE products
SET discount = 0.15
WHERE category IN (
    SELECT name FROM categories WHERE is_premium = true
);
```
