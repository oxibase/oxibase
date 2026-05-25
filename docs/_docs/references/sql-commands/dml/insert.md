---
layout: default
title: INSERT
parent: DML
grand_parent: SQL Commands
---

# INSERT

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("INSERT INTO"),
        NonTerminal("table_name"),
        Optional(Sequence([Keyword("("), OneOrMore(NonTerminal("column_name"), Keyword(",")), Keyword(")")])),
        Choice(0, [
          Sequence([Keyword("VALUES"), OneOrMore(Sequence([Keyword("("), OneOrMore(NonTerminal("value"), Keyword(",")), Keyword(")")]), Keyword(","))]),
          Sequence([Keyword("SELECT"), NonTerminal("...")])
        ]),
        Optional(Sequence([Keyword("ON DUPLICATE KEY UPDATE"), OneOrMore(Sequence([NonTerminal("column_name"), Keyword("="), NonTerminal("value")]), Keyword(","))])),
        Optional(Sequence([Keyword("RETURNING"), Choice(0, [Keyword("*"), OneOrMore(NonTerminal("column_name"), Keyword(","))])]))
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

The INSERT statement adds new rows to a table.

#### Basic Syntax

```sql
-- Single row
INSERT INTO table_name [(column1, column2, ...)]
VALUES (value1, value2, ...)
[RETURNING *|column1, column2, ...];

-- Multiple rows
INSERT INTO table_name [(column1, column2, ...)]
VALUES
  (value1_1, value1_2, ...),
  (value2_1, value2_2, ...);

-- With ON DUPLICATE KEY UPDATE
INSERT INTO table_name [(column1, column2, ...)]
VALUES (value1, value2, ...)
ON DUPLICATE KEY UPDATE
  column1 = new_value1,
  column2 = new_value2;
```

#### Examples

```sql
-- Basic insertion
INSERT INTO customers (id, name, email)
VALUES (1, 'John Doe', 'john@example.com');

-- Multiple rows
INSERT INTO products (id, name, price) VALUES
(1, 'Laptop', 1200.00),
(2, 'Smartphone', 800.00),
(3, 'Tablet', 500.00);

-- With RETURNING clause
INSERT INTO users (name, email)
VALUES ('Alice', 'alice@example.com')
RETURNING id, name;

-- Upsert with ON DUPLICATE KEY UPDATE
INSERT INTO inventory (product_id, quantity)
VALUES (101, 50)
ON DUPLICATE KEY UPDATE
  quantity = quantity + 50;
```

See [ON DUPLICATE KEY UPDATE]({% link _docs/references/sql-features/on-duplicate-key-update.md %}) for detailed documentation.
