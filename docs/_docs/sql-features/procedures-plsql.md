---
title: "Stored Procedures in PL/SQL"
permalink: /docs/sql-features/procedures/plsql/
excerpt: "How to write stored procedures using Oxibase's native PL/SQL dialect."
layout: doc
---

# Stored Procedures in PL/SQL

Oxibase implements a dedicated native procedural language clone, inspired heavily by PostgreSQL's `PL/pgSQL`. This allows you to write procedures using standard database control flows, without needing to learn a new external scripting language.

Because this interpreter is built natively within Oxibase, it provides deep integration and will serve as the foundation for our upcoming debugger (DAP) capabilities.

## Basic Usage

You can define a PL/SQL procedure using `LANGUAGE sql` or `LANGUAGE plsql`. The procedure body generally requires a `BEGIN ... END;` block.

```sql
CREATE PROCEDURE check_val(val INT, OUT is_positive BOOLEAN) 
LANGUAGE plsql 
AS '
BEGIN 
    IF val > 0 THEN 
        is_positive := TRUE; 
    ELSE 
        is_positive := FALSE; 
    END IF; 
END; 
';
```

Call the procedure:
```sql
CALL check_val(5, false);
```

**Result:**
| is_positive |
| :--- |
| true |

## Variable Assignment

In PL/SQL, variables are assigned using the `:=` operator. Note that currently, variables act dynamically during assignment in the MVP implementation.

```sql
CREATE PROCEDURE calculate_fee(base_price INT, INOUT total INT)
LANGUAGE plsql
AS '
BEGIN
    IF base_price > 100 THEN
        total := base_price + 20;
    ELSE
        total := base_price + 5;
    END IF;
END;
';
```

## Supported Syntax Overview

The PL/SQL native dialect currently supports:
- **Blocks**: `BEGIN ... END;`
- **Conditionals**: `IF condition THEN ... ELSE ... END IF;`
- **Assignments**: `variable := expression;`
- **Return**: `RETURN;` (to exit the block early)

_Note: The PL/SQL dialect is continuously evolving. Features like looping (`WHILE`, `FOR`), explicit `DECLARE` blocks, and native transaction control (`COMMIT`, `ROLLBACK`) are slated for upcoming releases._
