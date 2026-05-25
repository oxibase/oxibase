---
layout: default
title: SHOW FUNCTIONS
parent: UTILITY
grand_parent: SQL Commands
---

# SHOW FUNCTIONS

Lists all available SQL functions (scalar, aggregate, and window functions).

#### Basic Syntax

```sql
SHOW FUNCTIONS;
SHOW FUNCTION;  -- Also accepted
```

#### Example

```sql
SHOW FUNCTIONS;
```

Output:
```
name       type
---------  ---------
UPPER      SCALAR
COUNT      AGGREGATE
ROW_NUMBER WINDOW
...
```
