# Contracts: PL/SQL Syntax Extension

This documents the extended grammar contract for the PL/SQL parser.

## Extended Grammar

```ebnf
return_statement ::= "RETURN" [ expression ] ";"
```

The PL/SQL parser expects `RETURN` followed by an optional SQL expression, terminating with a semicolon.
