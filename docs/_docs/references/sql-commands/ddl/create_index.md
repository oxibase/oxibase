---
layout: default
title: CREATE INDEX
parent: DDL
grand_parent: SQL Commands
---

# CREATE INDEX

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("CREATE"),
        Optional(Keyword("UNIQUE")),
        Keyword("INDEX"),
        Optional(Sequence([Keyword("IF NOT EXISTS")])),
        NonTerminal("index_name"),
        Keyword("ON"),
        NonTerminal("table_name"),
        Keyword("("),
        OneOrMore(NonTerminal("column_name"), Keyword(",")),
        Keyword(")")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Creates an index on table columns for faster queries.

#### Basic Syntax

```sql
CREATE [UNIQUE] INDEX [IF NOT EXISTS] index_name
ON table_name (column_name [, column_name...]);
```

#### Index Type Selection

Oxibase automatically selects the optimal index type based on column data type:

| Data Type | Index Type | Best For |
|-----------|------------|----------|
| INTEGER, FLOAT, TIMESTAMP | B-tree | Range queries, equality, sorting |
| TEXT, JSON | Hash | Equality lookups, IN clauses |
| BOOLEAN | Bitmap | Low-cardinality columns |

#### Examples

```sql
-- Single-column index
CREATE INDEX idx_user_email ON users (email);

-- Multi-column index
CREATE INDEX idx_order_customer_date ON orders (customer_id, order_date);

-- Unique index
CREATE UNIQUE INDEX idx_unique_email ON users (email);

-- With IF NOT EXISTS
CREATE INDEX IF NOT EXISTS idx_name ON products (name);
```

See [Indexing]({% link _docs/explanations/architecture/indexing.md %}) for detailed documentation.
