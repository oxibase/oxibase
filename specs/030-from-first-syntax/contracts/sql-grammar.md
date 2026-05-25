# SQL Grammar Contract

This document outlines the grammar changes introduced by the FROM-first syntax.

## FROM-First Query Grammar

The parser supports a new top-level statement structure that starts with the `FROM` keyword. 
After the `FROM` keyword and table expression, any of the standard `SELECT` modifiers can follow in any order.

```ebnf
<from_first_statement> ::= "FROM" <table_expression> [ <from_clause_list> ]

<from_clause_list> ::= <from_clause> [ <from_clause_list> ]

<from_clause> ::= 
    | "SELECT" [ "DISTINCT" ] <select_list>
    | "WHERE" <expression>
    | "GROUP BY" <group_by_list>
    | "HAVING" <expression>
    | "ORDER BY" <order_by_list>
    | "LIMIT" <expression>
    | "OFFSET" <expression> [ "ROWS" | "ROW" ]
    | "WINDOW" <window_definition_list>

```

### Semantic Rules
- If the `<from_clause_list>` does not contain a `"SELECT"` clause, the parser implicitly constructs a `"SELECT *"` projection.
- Set operations (`UNION`, `INTERSECT`, `EXCEPT`) can be applied to `<from_first_statement>` blocks just like standard `<select_statement>` blocks.
- The `table_expression` can be a table name, a subquery, a table function, or a `VALUES` clause, identical to standard `FROM` clauses.