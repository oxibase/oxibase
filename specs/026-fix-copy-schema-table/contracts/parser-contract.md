# Parser Contract: COPY Statement Syntax

The SQL parser contract for the `COPY` statement is updated to accept schema-qualified table names.

## Syntax Rules

```ebnf
<copy_statement> ::= COPY <table_name> [ ( <column_list> ) ] FROM <file_path> [ WITH ( <copy_options> ) ]

<table_name> ::= [ <identifier> . ] <identifier>

<column_list> ::= <identifier> [ , <identifier> ... ]

<file_path> ::= '<string_literal>'
```

The primary change is in `<table_name>`, which now explicitly permits the `[ <schema_name> . ] <table_name>` two-part identifier structure, aligned with the rest of the DDL and DML statements.