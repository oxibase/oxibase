# Contract: Procedural Random Functions

This document specifies the exact API contracts for random number generation in Rhai, Python, and PL/SQL.

## 1. Rhai API
- **Namespace**: `oxibase::`
- **Signature**: `random() -> float`
- **Example Usage**:
  ```javascript
  let r = oxibase::random();
  if r < 0.5 {
      // Do something
  }
  ```

## 2. Python API
- **Namespace**: `oxibase.`
- **Signature**: `random() -> float`
- **Example Usage**:
  ```python
  import oxibase
  r = oxibase.random()
  if r < 0.5:
      # Do something
  ```

## 3. PL/SQL API
- **Syntax**: `RANDOM() -> FLOAT`
- **Example Usage**:
  ```sql
  DECLARE
      v_rand FLOAT;
  BEGIN
      v_rand := random();
  END;
  ```
