---
layout: default
title: CREATE FUNCTION
parent: Data Definition Language (DDL)
grand_parent: SQL Commands
---

# CREATE FUNCTION

<div id="rrdiagram"></div>
<script class="railroad-diagram-script">
  (function() {
    var diagram = Diagram([
      Sequence([
        Keyword("CREATE FUNCTION"),
        Optional(Sequence([Keyword("IF NOT EXISTS")])),
        NonTerminal("function_name"),
        Keyword("("),
        ZeroOrMore(Sequence([NonTerminal("param_name"), NonTerminal("data_type")]), Keyword(",")),
        Keyword(")"),
        Keyword("RETURNS"),
        NonTerminal("return_type"),
        Keyword("LANGUAGE RHAI AS"),
        NonTerminal("code")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Creates a user-defined function.

#### Basic Syntax

```sql
CREATE FUNCTION [IF NOT EXISTS] function_name (
    param1 data_type,
    param2 data_type,
    ...
)
RETURNS return_type
LANGUAGE RHAI AS 'code';
```

#### Parameters

- **function_name**: Name of the function
- **param1, param2, ...**: Parameter names and their data types
- **return_type**: Return value data type
- **LANGUAGE RHAI**: Specifies the language
- **AS 'code'**: The function implementation code

#### Supported Data Types

- `INTEGER` - 64-bit signed integers
- `FLOAT` - 64-bit floating-point numbers
- `TEXT` - UTF-8 text strings
- `BOOLEAN` - True/false values
- `TIMESTAMP` - Date and time values
- `JSON` - JSON documents

#### Examples

```sql
-- Simple arithmetic function
CREATE FUNCTION add_numbers(a INTEGER, b INTEGER)
RETURNS INTEGER
LANGUAGE RHAI AS 'a + b';

-- String manipulation
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE RHAI AS '"Hello, " + name + "!"';
```

See [User-Defined Functions]({% link _docs/references/functions/user-defined-functions.md %}) for detailed documentation.
