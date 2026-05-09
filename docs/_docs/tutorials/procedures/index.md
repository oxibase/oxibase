---
title: "Stored Procedures"
layout: default
parent: Tutorials
has_children: true
nav_order: 3
---

# Stored Procedures

Stored Procedures allow you to encapsulate and execute business logic directly within the database engine. By moving the logic closer to your data, you can reduce network latency, ensure consistent business rule application, and build complex procedural operations natively.

Unlike [Scalar Functions](/docs/sql-features/functions), which return a single value and are evaluated row-by-row inside `SELECT` queries, stored procedures are invoked as standalone statements using the `CALL` command. They can modify variables and return updated data via `OUT` or `INOUT` parameters (following the PostgreSQL-style convention).

## Overview

A procedure is created using the `CREATE OR REPLACE PROCEDURE` syntax and invoked with `CALL`. Oxibase is an embedded polyglot database, meaning it supports multiple scripting languages natively out-of-the-box. 

### Supported Languages

You can write your procedures in any of the following supported dialects:

- **[Rhai](/docs/tutorials/procedures/rhai)**: The default, ultra-fast embedded scripting language tailored for Rust applications. (Always available)
- **[PL/SQL](/docs/tutorials/procedures/plsql)**: A native database procedural language clone of PL/pgSQL. (Always available)
- **[JavaScript](/docs/tutorials/procedures/javascript)**: Write logic using the ubiquitous ECMAScript standard. (Requires the `js` feature flag)
- **[Python](/docs/tutorials/procedures/python)**: Utilize Python for your business logic. (Requires the `python` feature flag)

## General Syntax

The syntax for creating a procedure is standard across languages. You declare parameters with optional modes (`IN`, `OUT`, `INOUT`). If no mode is specified, `IN` is assumed.

```sql
CREATE OR REPLACE PROCEDURE calculate_discount(price FLOAT, OUT final_price FLOAT) 
LANGUAGE <backend>
AS '
    -- Your logic here 
';
```

Executing the procedure:
```sql
CALL calculate_discount(100.0, 0.0);
```

### Parameter Modes
- `IN`: (Default) The value is passed into the procedure but cannot be passed back to the caller.
- `OUT`: The initial value is ignored. The procedure assigns a value to this parameter, and it is returned to the caller as part of a result set.
- `INOUT`: The parameter provides an initial value, and the procedure can mutate it to return the new value.

When a procedure contains `OUT` or `INOUT` parameters, the `CALL` statement will return a single-row result set containing those updated variables.

---

Choose a language below to see specific tutorials and examples:

- [Rhai Procedures](/docs/tutorials/procedures/rhai)
- [PL/SQL Procedures](/docs/tutorials/procedures/plsql) 
- [JavaScript Procedures](/docs/tutorials/procedures/javascript)
- [Python Procedures](/docs/tutorials/procedures/python)
