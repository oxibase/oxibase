# Data Model & State Transitions: plsql-table-iteration

## PL/SQL AST Structure Additions

```rust
// In src/functions/plsql/ast.rs

#[derive(Debug, Clone, PartialEq)]
pub enum PlSqlStatement {
    // ... Existing statements
    
    /// FOR loop statement: FOR var IN expr LOOP body END LOOP;
    ForLoop(ForLoopStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoopStatement {
    pub token: Token,
    pub loop_variable: String,
    pub collection_expr: Expression,
    pub body: Vec<PlSqlStatement>,
}
```

---

## State & Variable Scoping Model

### Loop-scoped stack frame transitions

During the execution of a `FOR` loop statement:

1. **Evaluation**:
   - The `collection_expr` is evaluated to a `Value`.
   - If the value is `Value::Null(_)`, loop execution terminates immediately.
   - If the value is a string or JSON, it is parsed as a JSON array (`serde_json::Value::Array`). If not an array, it is wrapped in an array of a single element for maximum robustness.

2. **Iteration (for each row in array)**:
   - Push a loop stack frame onto the Environment: `env.push_frame("for_loop_body")`.
   - Convert the current JSON array item to a serialized `Value::Json` representation.
   - Define the `loop_variable` inside the loop frame containing this JSON row: `env.define(&loop_variable, current_row_val)`.
   - Execute the body statements sequentially.
   - Support control flow (e.g., encountering a `RETURN` statement bubbles up through the loop and terminates execution).
   - Pop the loop frame: `env.pop_frame()`.

---

## Field Assignments Dot-Notation Model

When performing an assignment statement:
- **Left-hand side checking**:
  - If `assign.variable` contains a `.` (e.g., `row.age`):
    - Split into `base_var = "row"` and `field = "age"`.
    - Retrieve `row` from the current Environment.
    - Ensure `row` is `Value::Json`.
    - Parse JSON to map, insert/update field `age` with the serialized value of the right-hand expression.
    - Serialize back to JSON.
    - Update `row` in the environment via `env.assign("row", updated_row)`.
