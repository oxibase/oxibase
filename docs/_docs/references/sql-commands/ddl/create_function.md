---
layout: default
title: CREATE FUNCTION
parent: DDL
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
        Keyword("LANGUAGE BOA AS"),
        NonTerminal("javascript_code")
      ])
    ]);
    document.getElementById("rrdiagram").innerHTML = diagram.toString();
  })();
</script>

Creates a user-defined function using JavaScript/TypeScript.

#### Basic Syntax

```sql
CREATE FUNCTION [IF NOT EXISTS] function_name (
    param1 data_type,
    param2 data_type,
    ...
)
RETURNS return_type
LANGUAGE BOA AS 'JavaScript code';
```

#### Parameters

- **function_name**: Name of the function
- **param1, param2, ...**: Parameter names and their data types
- **return_type**: Return value data type
- **LANGUAGE BOA**: Specifies JavaScript/TypeScript implementation
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
LANGUAGE BOA AS 'return arguments[0] + arguments[1];';

-- String manipulation
CREATE FUNCTION greet(name TEXT)
RETURNS TEXT
LANGUAGE BOA AS 'return `Hello, ${arguments[0]}!`;';

-- Date processing
CREATE FUNCTION format_date(ts TIMESTAMP)
RETURNS TEXT
LANGUAGE BOA AS '
    const date = new Date(arguments[0]);
    return date.toLocaleDateString();
';

-- JSON processing
CREATE FUNCTION extract_field(json_doc JSON, field TEXT)
RETURNS TEXT
LANGUAGE BOA AS '
    const doc = JSON.parse(arguments[0]);
    return doc[arguments[1]] || null;
';
```

See [User-Defined Functions]({% link _docs/references/functions/user-defined-functions.md %}) for detailed documentation.
