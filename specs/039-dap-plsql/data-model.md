# Data Model & Interface Contracts: DAP Support for PL/SQL

## 1. Abstract Syntax Tree (AST) Modifications
Located in `src/functions/plsql/ast.rs`

We need to add source mapping (line numbers) to the AST nodes to identify execution boundaries.

```rust
// Example additions
pub struct AssignmentStatement {
    pub line_number: usize, // New
    pub target: String,
    pub expression: Expression,
}

pub struct IfStatement {
    pub line_number: usize, // New
    pub condition: Expression,
    pub then_block: BlockStatement,
    // ...
}
```

## 2. Interpreter Hook Interface
Located in `src/functions/plsql/interpreter.rs` (or a related debugging module).

We define an interception point for the interpreter.

```rust
pub trait DebugAdapterHook {
    /// Called before a statement is evaluated.
    /// Blocks if a breakpoint is hit, allowing the DAP server to interact with the environment.
    fn on_statement_before_eval(&self, line_number: usize, env: &Environment);
}

// In PlSqlInterpreter:
pub struct PlSqlInterpreter<'a> {
    pub(crate) _function_registry: Arc<FunctionRegistry>,
    runner: Option<&'a dyn crate::functions::backends::SqlRunner>,
    debug_hook: Option<Arc<dyn DebugAdapterHook>>, // New
}
```

## 3. Environment State Mapping
No structural changes to `Environment` in `src/functions/plsql/env.rs`, but new mapping functions will be added to convert internal state to DAP state.

```rust
impl Environment {
    /// Converts current stack frames and variables to DAP structures
    pub fn to_dap_scopes(&self) -> Vec<DapScope> { ... }
}
```
