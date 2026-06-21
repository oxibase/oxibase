# SQL Interface Contract: Schema-Qualified Foreign Keys

## 1. Syntax Specification

### Column Constraint Definition
```antlr
column_constraint_references
    : 'REFERENCES' table_name ( '(' identifier ')' )?
    ;
```

### Table Constraint Definition
```antlr
table_constraint_foreign_key
    : ( 'CONSTRAINT' constraint_name )? 'FOREIGN' 'KEY' '(' identifier ')' 'REFERENCES' table_name '(' identifier ')'
    ;
```

### Table Name Definition
```antlr
table_name
    : identifier ( '.' identifier )?
    ;
```

## 2. Examples

* `customer_id INTEGER REFERENCES crm.customers(id)`
* `FOREIGN KEY (customer_id) REFERENCES crm.customers(id) ON DELETE CASCADE`
