---
layout: default
title: "Procedures"
parent: Functions
nav_order: 6
---

# Stored Procedures

Stored procedures are blocks of code that can be executed on the database server. Unlike user-defined functions, procedures can:

- Execute multiple SQL statements
- Access database context and metadata
- Perform data modifications (INSERT, UPDATE, DELETE)
- Control transaction flow (COMMIT, ROLLBACK)
- Return multiple result sets

## Syntax

```sql
CREATE PROCEDURE procedure_name(parameter_list)
LANGUAGE backend AS 'procedure_body';
```

## Parameters

Procedures support the same parameter types as functions:
- Named parameters: `param_name TYPE`
- Input parameters only (procedures don't return values like functions)

## Supported Backends

- **Rhai**: Lightweight scripting with access to database context
- **Deno**: Full JavaScript/TypeScript runtime
- **Python**: Python scripting environment

## Examples

### Rhai Procedure

```sql
CREATE PROCEDURE update_inventory(product_id INTEGER, quantity_change INTEGER)
LANGUAGE RHAI AS '
    // Query current inventory
    let current = query_one("SELECT quantity FROM inventory WHERE id = ?", [product_id]);
    
    // Update with new quantity
    let new_quantity = current + quantity_change;
    execute("UPDATE inventory SET quantity = ? WHERE id = ?", [new_quantity, product_id]);
    
    // Log the change
    execute("INSERT INTO inventory_log (product_id, change_amount, new_total) VALUES (?, ?, ?)", 
            [product_id, quantity_change, new_quantity]);
';
```

### Calling Procedures

```sql
-- Execute a procedure
CALL update_inventory(123, -5);

-- Procedures don't return values, but can affect multiple tables
```

## Differences from Functions

| Aspect | Functions | Procedures |
|--------|-----------|------------|
| Return Values | Always return a single value | No return values |
| SQL Usage | Can be used in SELECT, WHERE, etc. | Called with CALL statement |
| Side Effects | Pure functions (no side effects) | Can modify data and state |
| Context Access | Limited to input parameters | Full access to database context |
| Multiple Results | Single result | Can return multiple result sets |

## Use Cases

- **Data Processing**: Batch operations, ETL processes
- **Business Logic**: Complex workflows, validations
- **Maintenance**: Database cleanup, archiving
- **Reporting**: Multi-step report generation

## Transaction Control

Procedures can control transactions explicitly:

```sql
CREATE PROCEDURE transfer_funds(from_account INTEGER, to_account INTEGER, amount DECIMAL)
LANGUAGE RHAI AS '
    begin_transaction();
    
    try {
        // Debit from account
        execute("UPDATE accounts SET balance = balance - ? WHERE id = ?", [amount, from_account]);
        
        // Credit to account  
        execute("UPDATE accounts SET balance = balance + ? WHERE id = ?", [amount, to_account]);
        
        commit();
    } catch (error) {
        rollback();
        throw error;
    }
';
```

---

*Note: Stored procedures are planned for a future release. This page serves as a placeholder for upcoming functionality.*