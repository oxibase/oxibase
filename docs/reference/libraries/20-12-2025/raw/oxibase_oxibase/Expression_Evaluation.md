# Page: Expression Evaluation

# Expression Evaluation

<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

- [src/api/database.rs](src/api/database.rs)
- [src/api/transaction.rs](src/api/transaction.rs)
- [src/executor/ddl.rs](src/executor/ddl.rs)
- [src/executor/expression/evaluator_bridge.rs](src/executor/expression/evaluator_bridge.rs)
- [src/executor/expression/mod.rs](src/executor/expression/mod.rs)
- [src/functions/scalar/conversion.rs](src/functions/scalar/conversion.rs)
- [src/functions/scalar/datetime.rs](src/functions/scalar/datetime.rs)
- [src/functions/scalar/utility.rs](src/functions/scalar/utility.rs)

</details>



The expression evaluation system provides high-performance compilation and execution of SQL expressions using a stack-based bytecode virtual machine. This system replaces recursive AST traversal with compiled programs that enable zero-recursion evaluation, thread-safe parallel execution, and minimal memory allocation.

For information about aggregate function execution, see [Aggregation and GROUP BY](#3.4). For window function evaluation, see [Window Functions](#3.5). For built-in function reference, see [Built-in Functions](#5.2).

---

## Architecture Overview

The expression evaluation system follows a compile-once, execute-many architecture with clear separation between compilation and execution phases:

```mermaid
graph TB
    subgraph "Input Layer"
        AST["Expression AST<br/>(parser::ast::Expression)"]
    end
    
    subgraph "Compilation Phase"
        Compiler["ExprCompiler<br/>src/executor/expression/compiler.rs"]
        Context["CompileContext<br/>columns, function_registry,<br/>outer_columns, aliases"]
        Compiler -->|uses| Context
    end
    
    subgraph "Program Representation"
        Program["Program<br/>Arc&lt;Vec&lt;Op&gt;&gt;<br/>Immutable bytecode"]
        Constants["Constant Pool<br/>Pre-compiled values"]
        Program -->|contains| Constants
    end
    
    subgraph "Execution Phase"
        VM["ExprVM<br/>Stack-based VM<br/>Reusable instance"]
        ExecCtx["ExecuteContext<br/>row data, params,<br/>outer_row, transaction_id"]
        VM -->|uses| ExecCtx
    end
    
    subgraph "High-Level APIs"
        ExprEval["ExpressionEval<br/>Single expression"]
        RowFilter["RowFilter<br/>WHERE/HAVING<br/>Send + Sync"]
        MultiExpr["MultiExpressionEval<br/>SELECT projections"]
        JoinFilter["JoinFilter<br/>Join conditions"]
    end
    
    subgraph "Function System"
        FuncReg["FunctionRegistry<br/>101+ built-in functions"]
        Scalar["ScalarFunction"]
        Aggregate["AggregateFunction"]
        Window["WindowFunction"]
        
        FuncReg --> Scalar
        FuncReg --> Aggregate
        FuncReg --> Window
    end
    
    AST -->|compile| Compiler
    Compiler -->|generates| Program
    
    Program -->|Arc clone| ExprEval
    Program -->|Arc clone| RowFilter
    Program -->|Vec clone| MultiExpr
    Program -->|Arc clone| JoinFilter
    
    ExprEval -->|owns| VM
    RowFilter -->|thread_local| VM
    MultiExpr -->|owns| VM
    JoinFilter -->|thread_local| VM
    
    VM -->|executes| Program
    VM -->|calls| FuncReg
```

**Sources:** [src/executor/expression/mod.rs:1-56](), [src/executor/expression/evaluator_bridge.rs:1-50]()

### Key Design Principles

The expression VM is designed around four core principles:

| Principle | Implementation | Benefit |
|-----------|---------------|---------|
| **Zero Recursion** | Linear instruction sequences | Eliminates stack overflow risk |
| **Minimal Allocation** | Reusable stack, pre-allocated constants | Reduces GC pressure |
| **Fast Dispatch** | Direct enum match on `Op` | Avoids string comparisons |
| **Thread Safety** | `Arc<Program>` sharing | Safe parallel execution |

**Sources:** [src/executor/expression/mod.rs:20-24]()

---

## Compilation Phase

### ExprCompiler

The `ExprCompiler` transforms AST expressions into linear bytecode sequences. It performs a depth-first traversal of the expression tree, emitting instructions in execution order.

```mermaid
graph LR
    subgraph "Expression Types"
        Literal["Literal<br/>IntegerLiteral<br/>StringLiteral<br/>BooleanLiteral"]
        Infix["Infix<br/>+, -, *, /<br/>=, !=, &lt;, &gt;<br/>AND, OR"]
        FuncCall["FunctionCall<br/>UPPER, SUM<br/>JSON_EXTRACT"]
        Subquery["Subquery<br/>EXISTS<br/>IN<br/>Scalar"]
        Case["Case<br/>WHEN...THEN<br/>ELSE"]
    end
    
    subgraph "Compiler Operations"
        ConstPool["Constant Pool<br/>Pre-computed values"]
        InstEmit["Instruction Emission<br/>Op::Load<br/>Op::BinOp<br/>Op::Call"]
        ColMap["Column Mapping<br/>name → index"]
    end
    
    subgraph "Output"
        Bytecode["Bytecode Program<br/>Vec&lt;Op&gt;"]
    end
    
    Literal -->|emit LoadConst| ConstPool
    Infix -->|emit BinOp| InstEmit
    FuncCall -->|emit Call| InstEmit
    Subquery -->|emit Subquery| InstEmit
    Case -->|emit JumpIf| InstEmit
    
    ConstPool --> Bytecode
    InstEmit --> Bytecode
    ColMap --> InstEmit
```

**Sources:** [src/executor/expression/compiler.rs]()

### CompileContext

The `CompileContext` provides compilation-time information:

```rust
// From src/executor/expression/compiler.rs
CompileContext {
    columns: Vec<String>,              // Current row columns
    function_registry: &FunctionRegistry,
    outer_columns: Option<Vec<String>>, // For correlated subqueries
    columns2: Option<Vec<String>>,      // Second row (joins)
    expression_aliases: FxHashMap<String, u16>, // HAVING clause aliases
    column_aliases: FxHashMap<String, String>,
}
```

**Sources:** [src/executor/expression/compiler.rs]()

### Compilation Example

The compilation of `WHERE age > 18 AND name LIKE 'A%'` produces:

```mermaid
graph TD
    Start["Start Compilation"] --> LoadAge["Op::LoadColumn 'age' → col_idx=1"]
    LoadAge --> LoadConst18["Op::LoadConst 18 → const_idx=0"]
    LoadConst18 --> CmpGt["Op::BinOp Gt"]
    
    CmpGt --> LoadName["Op::LoadColumn 'name' → col_idx=2"]
    LoadName --> LoadPattern["Op::LoadConst 'A%' → const_idx=1"]
    LoadPattern --> Like["Op::Like false"]
    
    Like --> And["Op::And"]
    And --> Return["Op::Return"]
    
    Return --> Program["Program<br/>instructions: Vec&lt;Op&gt;<br/>constants: Vec&lt;Constant&gt;"]
```

**Sources:** [src/executor/expression/compiler.rs](), [src/executor/expression/program.rs]()

---

## Execution Phase

### ExprVM

The `ExprVM` is a stack-based virtual machine that executes compiled programs. It maintains an execution stack and processes instructions sequentially.

```mermaid
graph TB
    subgraph "VM State"
        Stack["Value Stack<br/>Vec&lt;Value&gt;<br/>Reused across evaluations"]
        PC["Program Counter<br/>usize"]
    end
    
    subgraph "Execution Loop"
        Fetch["Fetch Instruction<br/>program.ops[pc]"]
        Decode["Decode Op"]
        Execute["Execute Op"]
        Advance["Advance PC"]
    end
    
    subgraph "Operations"
        LoadOp["Load Operations<br/>LoadColumn<br/>LoadConst<br/>LoadParam"]
        ArithOp["Arithmetic<br/>Add, Sub, Mul, Div<br/>Pop 2, Push 1"]
        LogicOp["Logic<br/>And, Or, Not"]
        FuncOp["Function Call<br/>Pop args<br/>Call function<br/>Push result"]
        JumpOp["Control Flow<br/>JumpIf<br/>Jump"]
    end
    
    Stack --> Fetch
    PC --> Fetch
    Fetch --> Decode
    Decode --> Execute
    Execute --> LoadOp
    Execute --> ArithOp
    Execute --> LogicOp
    Execute --> FuncOp
    Execute --> JumpOp
    
    LoadOp --> Stack
    ArithOp --> Stack
    LogicOp --> Stack
    FuncOp --> Stack
    
    Execute --> Advance
    Advance --> Fetch
```

**Sources:** [src/executor/expression/vm.rs]()

### ExecuteContext

The `ExecuteContext` provides runtime data during execution:

```rust
// From src/executor/expression/vm.rs
ExecuteContext {
    row: &[Value],                     // Current row data
    row2: Option<&[Value]>,            // Second row (joins)
    params: &[Value],                  // Positional parameters
    named_params: &FxHashMap<String, Value>,
    outer_row: Option<&FxHashMap<Arc<str>, Value>>, // Correlated subquery
    transaction_id: Option<u64>,
}
```

**Sources:** [src/executor/expression/vm.rs]()

### Execution Flow

```mermaid
sequenceDiagram
    participant Caller
    participant VM as ExprVM
    participant Stack as Value Stack
    participant Func as FunctionRegistry
    
    Caller->>VM: execute(program, context)
    VM->>Stack: clear()
    
    loop For each instruction
        VM->>VM: fetch op
        alt Op::LoadColumn
            VM->>Stack: push(context.row[idx])
        else Op::LoadConst
            VM->>Stack: push(program.constants[idx])
        else Op::BinOp
            VM->>Stack: right = pop()
            VM->>Stack: left = pop()
            VM->>Stack: push(left op right)
        else Op::Call
            VM->>Stack: args = pop_n(arg_count)
            VM->>Func: call(func_name, args)
            Func-->>VM: result
            VM->>Stack: push(result)
        else Op::Return
            VM->>Stack: result = pop()
            VM-->>Caller: return result
        end
    end
```

**Sources:** [src/executor/expression/vm.rs]()

---

## High-Level APIs

The expression system provides multiple APIs for different use cases:

### API Comparison

| API | Use Case | Thread Safety | Ownership |
|-----|----------|---------------|-----------|
| `ExpressionEval` | Single expression, serial evaluation | No | Owns VM |
| `RowFilter` | WHERE/HAVING, parallel filtering | Yes (Send + Sync) | Thread-local VM |
| `MultiExpressionEval` | SELECT projections | No | Owns VM |
| `JoinFilter` | Join conditions | Yes (Send + Sync) | Thread-local VM |
| `CompiledEvaluator` | Legacy, dynamic compilation | No | Owns VM, has cache |

**Sources:** [src/executor/expression/evaluator_bridge.rs:15-28]()

### ExpressionEval

The simplest API for evaluating a single expression repeatedly:

```rust
// From src/executor/expression/evaluator_bridge.rs:375-654
// Usage:
let mut eval = ExpressionEval::compile(&expr, &columns)?;
for row in rows {
    let value = eval.eval(&row)?;
}
```

**Methods:**
- `compile(expr, columns)` - Compile expression
- `eval(&mut self, row)` - Evaluate for row
- `eval_bool(&mut self, row)` - Boolean evaluation (WHERE/HAVING)
- `eval_slice(&mut self, row_data)` - Avoid Row wrapper overhead
- `with_params(params)` - Set query parameters
- `with_context(ctx)` - Set execution context

**Sources:** [src/executor/expression/evaluator_bridge.rs:375-654]()

### RowFilter

Thread-safe filter for parallel execution:

```rust
// From src/executor/expression/evaluator_bridge.rs:92-286
// Usage:
let filter = RowFilter::new(&where_expr, &columns)?;

// Use in closure (filter is cloned)
let predicate = move |row: &Row| filter.matches(row);

// Or parallel iteration
rows.par_iter().filter(|row| filter.matches(row)).collect()
```

**Architecture:**
- Pre-compiled `Arc<Program>` (shared across threads)
- Thread-local `ExprVM` instances (via `thread_local!`)
- Zero allocation in hot path

**Sources:** [src/executor/expression/evaluator_bridge.rs:92-286]()

### Thread Safety Implementation

```mermaid
graph TB
    subgraph "Main Thread"
        Compile["Compile Expression<br/>RowFilter::new"]
        Program["Arc&lt;Program&gt;<br/>Shared, immutable"]
    end
    
    subgraph "Thread 1"
        Clone1["filter.clone()<br/>Arc::clone"]
        VM1["thread_local VM<br/>RefCell&lt;ExprVM&gt;"]
        Execute1["filter.matches(row1)"]
    end
    
    subgraph "Thread 2"
        Clone2["filter.clone()<br/>Arc::clone"]
        VM2["thread_local VM<br/>RefCell&lt;ExprVM&gt;"]
        Execute2["filter.matches(row2)"]
    end
    
    subgraph "Thread N"
        CloneN["filter.clone()<br/>Arc::clone"]
        VMN["thread_local VM<br/>RefCell&lt;ExprVM&gt;"]
        ExecuteN["filter.matches(rowN)"]
    end
    
    Compile --> Program
    Program --> Clone1
    Program --> Clone2
    Program --> CloneN
    
    Clone1 --> Execute1
    Clone2 --> Execute2
    CloneN --> ExecuteN
    
    Execute1 -.->|borrows| VM1
    Execute2 -.->|borrows| VM2
    ExecuteN -.->|borrows| VMN
```

**Sources:** [src/executor/expression/evaluator_bridge.rs:222-250]()

### MultiExpressionEval

Efficient evaluation of multiple expressions (SELECT projections):

```rust
// From src/executor/expression/evaluator_bridge.rs:656-811
// Usage:
let mut eval = MultiExpressionEval::compile(&select_exprs, &columns)?;
for row in rows {
    let values = eval.eval_all(&row)?; // Returns Vec<Value>
}
```

**Optimization:**
- Single VM instance for all expressions
- Pre-compiled all expressions at once
- Reuses stack between expressions

**Sources:** [src/executor/expression/evaluator_bridge.rs:656-811]()

---

## Instruction Set

The VM instruction set (`Op` enum) provides primitive operations:

### Instruction Categories

```mermaid
graph TB
    subgraph "Load Instructions"
        LoadCol["LoadColumn(u16)<br/>Push row[idx]"]
        LoadConst["LoadConst(u16)<br/>Push constants[idx]"]
        LoadParam["LoadParam(u16)<br/>Push params[idx]"]
        LoadNamed["LoadNamedParam(Arc&lt;str&gt;)<br/>Push named_params[name]"]
        LoadOuter["LoadOuterColumn(Arc&lt;str&gt;)<br/>Push outer_row[col]"]
    end
    
    subgraph "Arithmetic Operations"
        Add["Add<br/>Pop 2, Push sum"]
        Sub["Sub<br/>Pop 2, Push diff"]
        Mul["Mul<br/>Pop 2, Push product"]
        Div["Div<br/>Pop 2, Push quotient"]
        Mod["Mod<br/>Pop 2, Push remainder"]
        Neg["Neg<br/>Pop 1, Push -value"]
    end
    
    subgraph "Comparison Operations"
        Eq["Eq<br/>Pop 2, Push a == b"]
        Ne["Ne<br/>Pop 2, Push a != b"]
        Lt["Lt<br/>Pop 2, Push a &lt; b"]
        Lte["Lte<br/>Pop 2, Push a &lt;= b"]
        Gt["Gt<br/>Pop 2, Push a &gt; b"]
        Gte["Gte<br/>Pop 2, Push a &gt;= b"]
    end
    
    subgraph "Logic Operations"
        And["And<br/>Pop 2, Push a AND b"]
        Or["Or<br/>Pop 2, Push a OR b"]
        Not["Not<br/>Pop 1, Push NOT a"]
    end
    
    subgraph "Control Flow"
        JumpIf["JumpIf(offset)<br/>Pop condition<br/>Jump if false"]
        Jump["Jump(offset)<br/>Unconditional jump"]
        Return["Return<br/>Pop and return"]
    end
    
    subgraph "Complex Operations"
        Call["Call(fn_name, arg_count)<br/>Pop args, call function"]
        Like["Like(bool)<br/>Pattern matching"]
        InSet["InHashSet<br/>Membership test"]
        Between["Between<br/>Range test"]
        Subquery["Subquery ops<br/>EXISTS, IN, Scalar"]
    end
```

**Sources:** [src/executor/expression/ops.rs]()

### Instruction Format

The `Op` enum is defined in [src/executor/expression/ops.rs]():

```rust
pub enum Op {
    // Load operations
    LoadColumn(u16),           // Load from current row by index
    LoadConst(u16),            // Load from constant pool
    LoadParam(u16),            // Load positional parameter
    LoadNamedParam(Arc<str>),  // Load named parameter
    LoadOuterColumn(Arc<str>), // Load from outer row (correlated subquery)
    LoadColumn2(u16),          // Load from second row (joins)
    
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Neg, Pow,
    
    // Comparison
    Eq, Ne, Lt, Lte, Gt, Gte,
    
    // Logic
    And, Or, Not,
    
    // String operations
    Concat, Like(bool), // bool = negated
    
    // Set operations
    In, InHashSet, NotIn,
    
    // Range operations
    Between(bool), // bool = negated
    
    // Null operations
    IsNull, IsNotNull, Coalesce(u8), // u8 = arg count
    
    // Function calls
    Call(Arc<str>, u8), // (function_name, arg_count)
    
    // Control flow
    JumpIf(i16),  // Jump offset if top of stack is false
    Jump(i16),    // Unconditional jump
    Return,       // Return top of stack
    
    // Subquery operations
    SubqueryScalar(Box<SelectStatement>),
    SubqueryExists(Box<SelectStatement>),
    SubqueryIn(Box<SelectStatement>),
    SubqueryNotIn(Box<SelectStatement>),
    SubqueryAny(Box<SelectStatement>, Operator),
    SubqueryAll(Box<SelectStatement>, Operator),
    
    // Type conversion
    Cast(Arc<str>), // type_name
}
```

**Sources:** [src/executor/expression/ops.rs]()

---

## Function Integration

The expression VM integrates with the function registry for built-in function calls:

```mermaid
graph LR
    subgraph "Compilation"
        AST["FunctionCall AST<br/>UPPER(name)"]
        Compiler["ExprCompiler"]
        Emit["Emit Instructions<br/>LoadColumn('name')<br/>Call('UPPER', 1)"]
    end
    
    subgraph "Execution"
        VM["ExprVM"]
        Stack["Value Stack"]
        Registry["FunctionRegistry<br/>101+ functions"]
    end
    
    subgraph "Function Types"
        Scalar["ScalarFunction<br/>UPPER, LOWER<br/>CONCAT, SUBSTR"]
        Aggregate["AggregateFunction<br/>SUM, COUNT<br/>AVG, MIN, MAX"]
        Window["WindowFunction<br/>ROW_NUMBER, RANK<br/>LEAD, LAG"]
    end
    
    AST --> Compiler
    Compiler --> Emit
    
    Emit --> VM
    VM -->|"Op::Call"| Stack
    Stack -->|"pop args"| Registry
    Registry --> Scalar
    Registry --> Aggregate
    Registry --> Window
    
    Scalar -->|result| Stack
    Aggregate -->|result| Stack
    Window -->|result| Stack
```

**Sources:** [src/executor/expression/vm.rs](), [src/functions/]()

### Function Call Execution

When the VM encounters `Op::Call(fn_name, arg_count)`:

1. **Pop arguments** from stack (in reverse order)
2. **Lookup function** in registry by name
3. **Invoke function** with arguments
4. **Push result** onto stack

**Sources:** [src/executor/expression/vm.rs]()

### Example: UPPER(name) Execution

```mermaid
sequenceDiagram
    participant VM as ExprVM
    participant Stack
    participant Reg as FunctionRegistry
    participant Upper as UpperFunction
    
    VM->>Stack: Execute LoadColumn(2) # 'name'
    Stack-->>VM: Value::Text("alice")
    
    VM->>Stack: Execute Call("UPPER", 1)
    VM->>Stack: Pop 1 arg
    Stack-->>VM: ["alice"]
    
    VM->>Reg: lookup("UPPER")
    Reg-->>VM: &UpperFunction
    
    VM->>Upper: evaluate(["alice"])
    Upper-->>VM: Value::Text("ALICE")
    
    VM->>Stack: Push result
    Stack-->>VM: Done
```

**Sources:** [src/executor/expression/vm.rs](), [src/functions/scalar/string.rs]()

---

## Expression Compilation Patterns

### Simple Expressions

**Input:** `age > 18`

**Bytecode:**
```
0: LoadColumn(1)      # Load 'age' column
1: LoadConst(0)       # Load constant 18
2: Gt                 # Compare >
3: Return
```

**Sources:** [src/executor/expression/compiler.rs]()

### Complex Expressions with AND/OR

**Input:** `age > 18 AND status = 'active'`

**Bytecode:**
```
0: LoadColumn(1)      # 'age'
1: LoadConst(0)       # 18
2: Gt
3: LoadColumn(2)      # 'status'
4: LoadConst(1)       # 'active'
5: Eq
6: And                # Combine conditions
7: Return
```

**Sources:** [src/executor/expression/compiler.rs]()

### CASE Expression

**Input:** `CASE WHEN age < 18 THEN 'minor' ELSE 'adult' END`

**Bytecode with control flow:**
```
0: LoadColumn(1)      # 'age'
1: LoadConst(0)       # 18
2: Lt
3: JumpIf(6)          # Jump to ELSE if false
4: LoadConst(1)       # 'minor'
5: Jump(7)            # Skip ELSE
6: LoadConst(2)       # 'adult' (ELSE)
7: Return
```

**Sources:** [src/executor/expression/compiler.rs]()

### Function Calls

**Input:** `CONCAT(first_name, ' ', last_name)`

**Bytecode:**
```
0: LoadColumn(0)      # 'first_name'
1: LoadConst(0)       # ' '
2: LoadColumn(1)      # 'last_name'
3: Call("CONCAT", 3)  # Call with 3 args
4: Return
```

**Sources:** [src/executor/expression/compiler.rs]()

---

## Performance Optimizations

### Constant Folding

The compiler pre-evaluates constant expressions:

**Input:** `price * 1.08`

**Optimization:** If `1.08` is identified as constant, it's stored in the constant pool once rather than computed per row.

**Sources:** [src/executor/expression/compiler.rs]()

### Stack Reuse

The VM maintains a persistent stack across evaluations:

```mermaid
graph LR
    subgraph "Per-Row Evaluation"
        Start["Start: stack.len() = 0"]
        Exec["Execute: stack grows/shrinks"]
        Clear["End: stack.clear()"]
    end
    
    Start --> Exec
    Exec --> Clear
    Clear -.->|"Next row"| Start
    
    Note["Note: Stack capacity retained<br/>No reallocation needed"]
```

**Benefit:** Avoids repeated allocation of `Vec<Value>` for every row evaluation.

**Sources:** [src/executor/expression/vm.rs]()

### Program Sharing

Programs are immutable and wrapped in `Arc<Program>` for zero-cost sharing:

**Single Query:**
- Compile once: `Arc::new(program)`
- Clone Arc: Cheap pointer copy (8 bytes)
- Multiple threads: Each gets Arc clone, shares same bytecode

**Sources:** [src/executor/expression/evaluator_bridge.rs:814]()

### Thread-Local VMs

`RowFilter` uses thread-local VMs to avoid synchronization:

```rust
// From src/executor/expression/evaluator_bridge.rs:224-227
thread_local! {
    static VM: std::cell::RefCell<ExprVM> = std::cell::RefCell::new(ExprVM::new());
}
```

**Benefit:** Each thread has its own VM (with its own stack), avoiding lock contention while sharing the program.

**Sources:** [src/executor/expression/evaluator_bridge.rs:224-250]()

---

## Usage Examples

### WHERE Clause Filtering

```rust
// From usage in query executor
let filter = RowFilter::new(&where_expr, &columns)?
    .with_params(ctx.params().to_vec());

let filtered_rows: Vec<Row> = rows
    .into_iter()
    .filter(|row| filter.matches(row))
    .collect();
```

**Sources:** [src/executor/expression/evaluator_bridge.rs:111-135](), [src/executor/ddl.rs:735-763]()

### SELECT Projection

```rust
// From usage in query executor
let mut eval = MultiExpressionEval::compile(&select_exprs, &columns)?
    .with_context(&ctx);

let projected_rows: Vec<Row> = rows
    .into_iter()
    .map(|row| {
        let values = eval.eval_all(&row)?;
        Ok(Row::from_values(values))
    })
    .collect::<Result<Vec<_>>>()?;
```

**Sources:** [src/executor/expression/evaluator_bridge.rs:660-811]()

### HAVING Clause with Aggregate Aliases

```rust
// From aggregation executor
let aliases = vec![
    ("sum(amount)".to_string(), 2),  // aggregate result at index 2
    ("count(*)".to_string(), 3),
];

let filter = RowFilter::with_aliases(&having_expr, &result_columns, &aliases)?;

let filtered_groups: Vec<Row> = grouped_rows
    .into_iter()
    .filter(|row| filter.matches(row))
    .collect();
```

**Sources:** [src/executor/expression/evaluator_bridge.rs:140-182]()

### Correlated Subquery

```rust
// From subquery executor
let mut eval = ExpressionEval::compile(&subquery_expr, &inner_columns)?;

// For each outer row
for outer_row in outer_rows {
    // Set outer context
    let outer_map: FxHashMap<String, Value> = outer_columns
        .iter()
        .zip(outer_row.as_slice())
        .map(|(col, val)| (col.clone(), val.clone()))
        .collect();
    
    eval.set_outer_row(&outer_map);
    
    // Evaluate subquery with outer context
    let result = eval.eval(&inner_row)?;
}
```

**Sources:** [src/executor/expression/evaluator_bridge.rs:544-556](), [src/api/transaction.rs:214-216]()

---

## Integration with Query Execution

The expression VM is used throughout the query execution pipeline:

```mermaid
graph TB
    subgraph "Query Executor"
        Parse["SQL Parser<br/>AST generation"]
        Plan["Query Planner"]
    end
    
    subgraph "Expression VM Usage"
        WHERE["WHERE Clause<br/>RowFilter"]
        SELECT["SELECT Projection<br/>MultiExpressionEval"]
        JOIN["JOIN Condition<br/>JoinFilter"]
        HAVING["HAVING Clause<br/>RowFilter with aliases"]
        ORDERBY["ORDER BY<br/>ExpressionEval"]
        WINDOW["Window Functions<br/>ExpressionEval"]
    end
    
    subgraph "Execution Engines"
        QueryExec["Query Executor"]
        AggExec["Aggregation Engine"]
        WindowExec["Window Engine"]
        SubqueryExec["Subquery Engine"]
    end
    
    Parse --> Plan
    Plan --> QueryExec
    
    QueryExec -->|filter| WHERE
    QueryExec -->|project| SELECT
    QueryExec -->|join| JOIN
    QueryExec -->|sort| ORDERBY
    
    AggExec -->|filter groups| HAVING
    WindowExec -->|compute| WINDOW
    SubqueryExec -->|evaluate| SELECT
```

**Sources:** [src/executor/query.rs](), [src/executor/aggregation/](), [src/executor/window/]()

---

## Comparison with Legacy Evaluator

The `CompiledEvaluator` (deprecated) vs new APIs:

| Aspect | CompiledEvaluator (Deprecated) | New APIs (ExpressionEval, RowFilter) |
|--------|-------------------------------|-------------------------------------|
| **Compilation** | Lazy (per expression) | Eager (upfront) |
| **Caching** | Internal FxHashMap cache | Pre-compiled Arc<Program> |
| **Thread Safety** | No (mutable cache) | Yes (RowFilter is Send + Sync) |
| **API Complexity** | Complex (many methods) | Simple (focused APIs) |
| **Use Case** | General purpose | Specialized per use case |
| **Performance** | Good | Better (no cache lookup) |

**Migration Example:**

```rust
// Old: CompiledEvaluator
let mut eval = CompiledEvaluator::new(&registry);
eval.init_columns(&columns);
for row in rows {
    eval.set_row_array(&row);
    let value = eval.evaluate(&expr)?;
}

// New: ExpressionEval
let mut eval = ExpressionEval::compile(&expr, &columns)?;
for row in rows {
    let value = eval.eval(&row)?;
}
```

**Sources:** [src/executor/expression/evaluator_bridge.rs:818-867]()

---

## Error Handling

The expression VM propagates errors through `Result<Value>`:

**Common Error Types:**
- **Compile errors** - Invalid column references, unknown functions
- **Runtime errors** - Division by zero, type mismatches, function errors
- **Subquery errors** - Multiple rows for scalar subquery

```mermaid
graph TB
    Compile["Compilation Phase"]
    Execute["Execution Phase"]
    
    Compile -->|"Error::ColumnNotFound"| CompileErr["Compilation Error"]
    Compile -->|"Error::FunctionNotFound"| CompileErr
    
    Execute -->|"Error::DivisionByZero"| RuntimeErr["Runtime Error"]
    Execute -->|"Error::TypeMismatch"| RuntimeErr
    Execute -->|"Error::FunctionError"| RuntimeErr
    
    CompileErr --> Propagate["Propagate to caller"]
    RuntimeErr --> Propagate
```

**Sources:** [src/executor/expression/compiler.rs](), [src/executor/expression/vm.rs](), [src/core/error.rs]()