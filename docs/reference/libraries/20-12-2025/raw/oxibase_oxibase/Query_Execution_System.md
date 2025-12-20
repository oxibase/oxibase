# Page: Query Execution System

# Query Execution System

<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

- [.github/workflows/ci.yml](.github/workflows/ci.yml)
- [Cargo.toml](Cargo.toml)
- [src/executor/query.rs](src/executor/query.rs)
- [src/lib.rs](src/lib.rs)

</details>



## Purpose and Scope

The Query Execution System is the core of OxiBase's query processing pipeline. This document provides an overview of how SQL queries are parsed, analyzed, optimized, and executed. It covers the high-level architecture, execution flow, feature detection, and result processing.

For detailed information on specific subsystems:
- Core SELECT execution, table scans, joins, and optimization strategies: see [Query Execution Pipeline](#3.1)
- Expression compilation and bytecode evaluation: see [Expression Evaluation](#3.2)
- Subquery handling and optimization: see [Subquery Execution](#3.3)
- Aggregation and GROUP BY execution: see [Aggregation and GROUP BY](#3.4)
- Window function processing: see [Window Functions](#3.5)
- Common Table Expression (CTE) handling: see [Common Table Expressions](#3.6)

For storage-level operations and MVCC transaction management, see [Storage Engine](#4) and its subsections.

## Architecture Overview

The query execution system sits between the public API layer and the storage engine, transforming SQL statements into row results through multiple processing stages.

### Layered Architecture

```mermaid
graph TB
    subgraph "Public API Layer"
        Database["Database<br/>api::Database"]
        Transaction["Transaction<br/>api::Transaction"]
    end
    
    subgraph "Query Execution System"
        Parser["SQL Parser<br/>parser module<br/>AST generation"]
        Router["Query Router<br/>Executor::execute_select"]
        
        subgraph "Feature Detection"
            CTEDetect["CTE Detection<br/>has_cte"]
            SubqueryDetect["Subquery Detection<br/>process_where_subqueries"]
            AggDetect["Aggregation Detection<br/>has_aggregation"]
            WindowDetect["Window Detection<br/>has_window_functions"]
        end
        
        subgraph "Execution Engines"
            QueryExec["Query Executor<br/>execute_select_internal"]
            CTEExec["CTE Engine<br/>execute_select_with_ctes"]
            AggExec["Aggregation Engine<br/>execute_select_with_aggregation"]
            WindowExec["Window Engine<br/>execute_select_with_windows"]
        end
        
        ExprVM["Expression VM<br/>ExprVM, Program<br/>Bytecode evaluation"]
        
        Optimizer["Query Optimizer<br/>optimizer module<br/>Cost-based decisions"]
    end
    
    subgraph "Storage Layer"
        MVCCEngine["MVCCEngine<br/>Transaction management"]
        VersionStore["VersionStore<br/>MVCC row storage"]
        Indexes["Index Subsystem<br/>BTree, Hash, Bitmap"]
    end
    
    Database -->|"execute/query"| Parser
    Transaction -->|"execute/query"| Parser
    
    Parser -->|"SelectStatement AST"| Router
    
    Router --> CTEDetect
    Router --> SubqueryDetect
    Router --> AggDetect
    Router --> WindowDetect
    
    CTEDetect -->|"WITH clause"| CTEExec
    SubqueryDetect -->|"EXISTS, IN"| QueryExec
    AggDetect -->|"GROUP BY"| AggExec
    WindowDetect -->|"OVER clause"| WindowExec
    
    QueryExec --> ExprVM
    AggExec --> ExprVM
    WindowExec --> ExprVM
    CTEExec --> QueryExec
    
    QueryExec --> Optimizer
    Optimizer -->|"Access plans"| VersionStore
    
    QueryExec --> VersionStore
    VersionStore --> Indexes
    VersionStore -.->|"MVCC visibility"| MVCCEngine
    
    style Router fill:#f9f9f9
    style QueryExec fill:#f9f9f9
    style ExprVM fill:#f9f9f9
    style Optimizer fill:#f9f9f9
```

**Sources:** [src/executor/query.rs:1-66](), [src/lib.rs:56-73](), High-level diagrams

## Query Execution Pipeline

The execution pipeline transforms SQL text into row results through five major phases:

| Phase | Component | Input | Output | Purpose |
|-------|-----------|-------|--------|---------|
| **1. Parsing** | `parser` module | SQL text string | `SelectStatement` AST | Convert SQL syntax to structured AST |
| **2. Analysis** | `Executor::execute_select` | `SelectStatement` | Execution plan components | Detect query features (CTEs, aggregation, window functions) |
| **3. Planning** | `QueryPlanner` | AST + Statistics | Logical/physical plan | Generate optimized access strategies |
| **4. Execution** | `QueryExec`, `AggExec`, `WindowExec` | Execution plan | Raw row stream | Fetch and transform data |
| **5. Post-processing** | Result wrappers | Row stream | `QueryResult` | Apply ORDER BY, LIMIT, DISTINCT |

**Sources:** [src/executor/query.rs:154-719](), Diagram 2 (Query Execution Data Flow)

### Query Execution Entry Point

```mermaid
flowchart TD
    Entry["Executor::execute_select<br/>query.rs:154-719"]
    
    TimeoutGuard["TimeoutGuard::new<br/>Start timeout at top level<br/>query_depth == 0"]
    
    ClearCaches["Clear Subquery Caches<br/>clear_scalar_subquery_cache<br/>clear_in_subquery_cache<br/>clear_semi_join_cache"]
    
    ValidateWhere["Validate WHERE Clause<br/>No aggregates allowed<br/>expression_contains_aggregate"]
    
    CTECheck{"has_cte?<br/>Check for WITH clause"}
    
    CTEPath["execute_select_with_ctes<br/>Process CTEs first<br/>See page 3.6"]
    
    EvalLimitOffset["Evaluate LIMIT/OFFSET<br/>ExpressionEval::compile<br/>Early evaluation for optimization"]
    
    MainExec["execute_select_internal<br/>Core execution logic"]
    
    SetOps["execute_set_operations<br/>UNION/INTERSECT/EXCEPT<br/>Early termination for UNION ALL"]
    
    PostProcess["Post-processing:<br/>- DISTINCT<br/>- ORDER BY with TOP-N<br/>- LIMIT/OFFSET"]
    
    Result["QueryResult<br/>Box<dyn QueryResult>"]
    
    Entry --> TimeoutGuard
    TimeoutGuard --> ClearCaches
    ClearCaches --> ValidateWhere
    ValidateWhere --> CTECheck
    
    CTECheck -->|"Yes"| CTEPath
    CTEPath --> Result
    
    CTECheck -->|"No"| EvalLimitOffset
    EvalLimitOffset --> MainExec
    MainExec --> SetOps
    SetOps --> PostProcess
    PostProcess --> Result
    
    style Entry fill:#f9f9f9
    style MainExec fill:#f9f9f9
    style Result fill:#f9f9f9
```

**Sources:** [src/executor/query.rs:154-719]()

## Key Components

### Executor

The `Executor` struct is the main coordinator for query execution. It holds references to the storage engine, function registry, and manages execution context.

```mermaid
graph LR
    subgraph "Executor Structure"
        Executor["Executor<br/>src/executor/mod.rs"]
        
        Engine["engine: Arc&lt;MVCCEngine&gt;<br/>Storage backend"]
        
        FuncReg["function_registry:<br/>Arc&lt;FunctionRegistry&gt;<br/>101+ functions"]
        
        ActiveTx["active_transaction:<br/>Mutex&lt;Option&lt;TxState&gt;&gt;<br/>Explicit transaction state"]
        
        QueryCache["query_cache:<br/>Arc&lt;QueryCache&gt;<br/>Semantic caching"]
    end
    
    Executor --> Engine
    Executor --> FuncReg
    Executor --> ActiveTx
    Executor --> QueryCache
```

**Sources:** [src/lib.rs:142-145](), [src/executor/query.rs:153]()

### ExecutionContext

The `ExecutionContext` carries runtime state through the execution pipeline, including parameters, outer row context for correlated subqueries, timeout settings, and cancellation tokens.

**Key fields:**
- `params`: Query parameters for parameterized statements
- `outer_row`: HashMap for correlated subquery evaluation
- `query_depth`: Recursion depth tracking to prevent stack overflow
- `timeout`: Optional query timeout duration
- `cancel_token`: Shared cancellation flag for query interruption
- `cte_context`: Stack of CTE definitions for nested WITH clauses

**Sources:** [src/executor/query.rs:42-47](), referenced in [src/executor/query.rs:158-179]()

### Feature Detection and Dispatching

The executor detects query features early in the pipeline to route execution through specialized engines:

```mermaid
flowchart TD
    SelectStmt["SelectStatement AST"]
    
    CTECheck{"has_cte?<br/>stmt.cte.is_some"}
    AggCheck{"has_aggregation?<br/>Scan columns for<br/>aggregate functions"}
    WindowCheck{"has_window_functions?<br/>Scan for OVER clauses"}
    SubqueryCheck{"Has subqueries?<br/>In WHERE/SELECT"}
    
    CTEEngine["CTE Engine<br/>execute_select_with_ctes<br/>Handle WITH clauses<br/>Recursive support"]
    
    AggEngine["Aggregation Engine<br/>execute_select_with_aggregation<br/>GROUP BY processing<br/>ROLLUP/CUBE support"]
    
    WindowEngine["Window Engine<br/>execute_select_with_windows<br/>Partition management<br/>Frame calculations"]
    
    SubqueryEngine["Subquery Processing<br/>process_where_subqueries<br/>Semi-join optimization<br/>Caching"]
    
    SimpleExec["Simple Query Execution<br/>execute_select_internal<br/>Table scan + filter + project"]
    
    SelectStmt --> CTECheck
    
    CTECheck -->|"Yes"| CTEEngine
    CTECheck -->|"No"| AggCheck
    
    AggCheck -->|"Yes"| AggEngine
    AggCheck -->|"No"| WindowCheck
    
    WindowCheck -->|"Yes"| WindowEngine
    WindowCheck -->|"No"| SubqueryCheck
    
    SubqueryCheck -->|"Yes"| SubqueryEngine
    SubqueryCheck -->|"No"| SimpleExec
    
    CTEEngine -.->|"After CTE materialization"| AggCheck
    SubqueryEngine --> SimpleExec
    
    style CTEEngine fill:#f9f9f9
    style AggEngine fill:#f9f9f9
    style WindowEngine fill:#f9f9f9
    style SubqueryEngine fill:#f9f9f9
    style SimpleExec fill:#f9f9f9
```

**Detection methods:**
- **CTE Detection**: `has_cte(&stmt)` - checks for `WITH` clause in statement [src/executor/query.rs:191]()
- **Aggregation Detection**: `has_aggregation(&stmt)` - scans SELECT columns for aggregate function calls (COUNT, SUM, AVG, etc.)
- **Window Function Detection**: `has_window_functions(&stmt)` - scans for OVER clauses in expressions [src/executor/query.rs:1019]()
- **Subquery Detection**: Scans WHERE and SELECT clauses for subquery expressions during processing [src/executor/query.rs:792-794]()

**Sources:** [src/executor/query.rs:154-193](), [src/executor/query.rs:1019]()

## Execution Flow for Simple Queries

For queries without CTEs, aggregations, or window functions, execution follows this path:

```mermaid
flowchart TD
    Internal["execute_select_internal<br/>query.rs:737-783"]
    
    TableExpr{"Table expression<br/>type?"}
    
    SimpleTable["TableSource<br/>execute_simple_table_scan<br/>query.rs:963-1424"]
    
    JoinSource["JoinSource<br/>execute_join_source<br/>Handle JOIN operations"]
    
    SubqSource["SubquerySource<br/>execute_subquery_source<br/>FROM subquery"]
    
    ValuesSource["ValuesSource<br/>execute_values_source<br/>VALUES clause"]
    
    NoFrom["No FROM clause<br/>execute_expression_select<br/>query.rs:786-859"]
    
    CTELookup{"CTE in context?<br/>ctx.get_cte"}
    
    ViewLookup{"View exists?<br/>engine.get_view_lowercase"}
    
    TableScan["Table Scan<br/>Fetch rows from storage"]
    
    CTEExec["Execute on CTE data<br/>execute_query_on_memory_result"]
    
    ViewExec["Execute view query<br/>execute_view_query<br/>Recursive depth check"]
    
    WhereFilter["Apply WHERE clause<br/>RowFilter or ExprFilteredResult<br/>Predicate pushdown"]
    
    Project["Project columns<br/>SELECT expressions<br/>ExprMappedResult"]
    
    Result["QueryResult stream<br/>+ column names<br/>+ limit_offset_applied flag"]
    
    Internal --> TableExpr
    
    TableExpr -->|"None"| NoFrom
    TableExpr -->|"TableSource"| CTELookup
    TableExpr -->|"JoinSource"| JoinSource
    TableExpr -->|"SubquerySource"| SubqSource
    TableExpr -->|"ValuesSource"| ValuesSource
    
    CTELookup -->|"Found"| CTEExec
    CTELookup -->|"Not found"| ViewLookup
    
    ViewLookup -->|"Found"| ViewExec
    ViewLookup -->|"Not found"| TableScan
    
    TableScan --> WhereFilter
    WhereFilter --> Project
    Project --> Result
    
    CTEExec --> Result
    ViewExec --> Result
    JoinSource --> Result
    SubqSource --> Result
    ValuesSource --> Result
    NoFrom --> Result
    
    style Internal fill:#f9f9f9
    style SimpleTable fill:#f9f9f9
    style Result fill:#f9f9f9
```

**Sources:** [src/executor/query.rs:737-783](), [src/executor/query.rs:963-1424](), [src/executor/query.rs:786-859]()

## Table Scan Execution

The simple table scan is the most common execution path, optimized for both transactional consistency and performance:

```mermaid
flowchart TD
    ScanStart["execute_simple_table_scan<br/>query.rs:963-1424"]
    
    TxCheck{"Active explicit<br/>transaction?"}
    
    UseTx["Use active transaction<br/>See uncommitted changes<br/>active_transaction.lock"]
    
    NewTx["Create standalone transaction<br/>tx = engine.begin_transaction<br/>Snapshot isolation"]
    
    TemporalCheck{"AS OF clause?"}
    
    Temporal["execute_temporal_query<br/>Time-travel query<br/>See page 5.4"]
    
    GetTable["Get table<br/>tx.get_table<br/>MVCCTable handle"]
    
    Schema["Extract schema<br/>table.schema.column_names_owned<br/>Build column list"]
    
    OptimCheck{"Optimization<br/>opportunities?"}
    
    PushdownOpt["Apply optimizations:<br/>- Predicate pushdown<br/>- Index selection<br/>- Projection pushdown disabled<br/>query.rs:1036-1118"]
    
    Scan["Create scanner<br/>table.scan or table.range_scan<br/>MVCCScanner iteration"]
    
    WhereExpr["Compile WHERE filter<br/>RowFilter::new<br/>Arc&lt;Program&gt; + thread_local VM"]
    
    FilterResult["ExprFilteredResult<br/>Parallel filtering possible<br/>MVCC visibility check"]
    
    ProjectExpr["Compile SELECT expressions<br/>MultiExpressionEval<br/>Vec&lt;Program&gt;"]
    
    ProjectResult["ExprMappedResult or<br/>StreamingProjectionResult<br/>Column transformation"]
    
    Return["Return:<br/>QueryResult + columns<br/>+ limit_offset_applied"]
    
    ScanStart --> TxCheck
    
    TxCheck -->|"Yes"| UseTx
    TxCheck -->|"No"| NewTx
    
    UseTx --> TemporalCheck
    NewTx --> TemporalCheck
    
    TemporalCheck -->|"Yes"| Temporal
    Temporal --> Return
    
    TemporalCheck -->|"No"| GetTable
    GetTable --> Schema
    Schema --> OptimCheck
    
    OptimCheck -->|"Applicable"| PushdownOpt
    OptimCheck -->|"None"| Scan
    
    PushdownOpt --> Scan
    Scan --> WhereExpr
    WhereExpr --> FilterResult
    FilterResult --> ProjectExpr
    ProjectExpr --> ProjectResult
    ProjectResult --> Return
    
    style ScanStart fill:#f9f9f9
    style Scan fill:#f9f9f9
    style FilterResult fill:#f9f9f9
    style ProjectResult fill:#f9f9f9
```

**Key optimizations in table scan:**
- **Transaction reuse**: Active explicit transactions are reused to see uncommitted changes [src/executor/query.rs:973-1006]()
- **Predicate pushdown**: WHERE clause analysis to push filters closer to storage [src/executor/query.rs:1036-1118]()
- **Index selection**: Automatic index selection based on predicates and available indexes
- **Parallel filtering**: `RowFilter` uses thread-local VMs for parallel iteration
- **Zero-copy scanning**: Arena-based row storage avoids cloning when possible

**Sources:** [src/executor/query.rs:963-1424]()

## Result Processing Pipeline

After core execution, results pass through a series of transformation wrappers:

```mermaid
flowchart LR
    Raw["Raw execution result<br/>MVCCScanner or<br/>MemoryResult"]
    
    SetOps["Set Operations<br/>execute_set_operations<br/>UNION/INTERSECT/EXCEPT"]
    
    Distinct["DISTINCT<br/>DistinctResult<br/>Hash-based deduplication"]
    
    OrderBy["ORDER BY<br/>OrderedResult or TopNResult<br/>Radix sort for integers"]
    
    Project["Column projection<br/>ProjectedResult<br/>Remove ORDER BY columns"]
    
    Limit["LIMIT/OFFSET<br/>LimitedResult<br/>Skip and take"]
    
    Final["Final QueryResult<br/>Returned to user"]
    
    Raw --> SetOps
    SetOps --> Distinct
    Distinct --> OrderBy
    OrderBy --> Project
    Project --> Limit
    Limit --> Final
    
    style Raw fill:#f9f9f9
    style Final fill:#f9f9f9
```

### Result Wrapper Types

The execution system uses a series of `QueryResult` wrappers to apply transformations:

| Wrapper Type | Purpose | Implementation | Performance |
|--------------|---------|----------------|-------------|
| `ScannerResult` | Wraps storage scanner | Delegates to `MVCCScanner` | Zero overhead |
| `FilteredResult` | WHERE clause filtering | `RowFilter` with bytecode VM | Compiled, fast |
| `ExprMappedResult` | SELECT projection | `MultiExpressionEval` | Compiled expressions |
| `DistinctResult` | DISTINCT deduplication | `FxHashSet` on row hashes | O(1) per row |
| `OrderedResult` | ORDER BY sorting | Radix sort or comparison sort | O(n) or O(n log n) |
| `TopNResult` | ORDER BY + LIMIT | Bounded heap | O(n log k) |
| `LimitedResult` | LIMIT/OFFSET | Iterator skip/take | O(1) |
| `ProjectedResult` | Column truncation | Index-based projection | O(1) per row |

**Sources:** [src/executor/query.rs:267-719](), [src/executor/result.rs]() (referenced)

### TOP-N Optimization

When ORDER BY is combined with LIMIT, the executor uses a bounded heap instead of full sorting:

**Characteristics:**
- **Complexity**: O(n log k) instead of O(n log n), where k = LIMIT
- **Memory**: Only keeps k rows in heap, not full dataset
- **Speedup**: 5-50x faster for large datasets with small limits
- **Implementation**: `TopNResult::new` [src/executor/query.rs:658-674]()

**Example**: `SELECT * FROM large_table ORDER BY score DESC LIMIT 10` on 1M rows:
- Full sort: O(1M log 1M) ≈ 20M comparisons
- TOP-N heap: O(1M log 10) ≈ 3.3M comparisons
- **~6x speedup**

**Sources:** [src/executor/query.rs:658-674]()

## Optimization Techniques

### Predicate Pushdown

The executor analyzes WHERE clauses to push filtering closer to storage, reducing rows that need to be processed:

```mermaid
graph TD
    Where["WHERE clause<br/>Complex expression"]
    
    Analyze["Analyze predicates<br/>flatten_and_predicates<br/>Extract conjunctions"]
    
    Indexable{"Can use index?<br/>Check for:<br/>- Equality on indexed column<br/>- Range on BTree index<br/>- IN list on Hash index"}
    
    IndexScan["Index scan<br/>table.range_scan<br/>O log n or O 1"]
    
    TableScan["Full table scan<br/>table.scan<br/>All rows"]
    
    Filter["Apply remaining predicates<br/>RowFilter<br/>Bytecode evaluation"]
    
    Where --> Analyze
    Analyze --> Indexable
    
    Indexable -->|"Yes"| IndexScan
    Indexable -->|"No"| TableScan
    
    IndexScan --> Filter
    TableScan --> Filter
    
    style IndexScan fill:#f9f9f9
    style Filter fill:#f9f9f9
```

**Sources:** [src/executor/query.rs:1036-1118](), [src/executor/pushdown.rs]() (referenced)

### Join Filter Pushdown

For JOIN operations, WHERE clause predicates are partitioned and pushed to individual tables:

**Partitioning logic** [src/executor/query.rs:110-151]():
```
partition_where_for_join(where_clause, left_alias, right_alias):
  1. Flatten AND predicates
  2. For each predicate:
     - If references only left table → push to left
     - If references only right table → push to right
     - If references both tables → apply post-join
  3. Return (left_filter, right_filter, cross_table_filter)
```

**Benefits:**
- Reduces rows before join operation
- Enables index usage on individual tables
- Reduces memory for hash join build phase

**Sources:** [src/executor/query.rs:110-151]()

### Radix Sort for ORDER BY

The executor attempts radix sort for ORDER BY on integer columns, achieving O(n) complexity instead of O(n log n):

**Algorithm selection** [src/executor/query.rs:676-704]():
1. Check if all ORDER BY columns have valid indices (no complex expressions)
2. If yes, build `RadixOrderSpec` array
3. `OrderedResult::new_radix` attempts radix sort
4. Falls back to comparison sort if radix sort is not applicable (non-integer types)

**Characteristics:**
- **Best case**: O(n) for integer keys
- **Memory**: In-place sorting
- **Stability**: Stable sort (equal elements maintain order)

**Sources:** [src/executor/query.rs:676-704]()

## Expression Compilation

All expressions (WHERE, SELECT, ORDER BY) are compiled to bytecode before execution:

```mermaid
flowchart LR
    AST["Expression AST<br/>parser::ast::Expression"]
    
    Compile["ExprCompiler<br/>Zero-recursion compilation<br/>Linear instruction sequence"]
    
    Program["Program<br/>Bytecode instructions<br/>Arc-shareable, immutable"]
    
    VM["ExprVM<br/>Stack-based VM<br/>Thread-local for parallel"]
    
    Eval["Evaluation<br/>Per-row execution<br/>Fast bytecode interpretation"]
    
    Result["Value result<br/>Boolean for WHERE<br/>Any type for SELECT"]
    
    AST --> Compile
    Compile --> Program
    Program -->|"Arc clone"| VM
    VM --> Eval
    Eval --> Result
    
    style Program fill:#f9f9f9
    style VM fill:#f9f9f9
```

**Key benefits:**
- **Zero recursion**: Linear instruction sequence eliminates stack overflow risk
- **Thread-safe**: `Arc<Program>` + thread-local VM enables parallel execution
- **Pre-compilation**: Compile once, execute many times (amortized cost)
- **Fast**: Bytecode interpretation faster than AST traversal

For detailed information on expression evaluation, see [Expression Evaluation](#3.2).

**Sources:** [src/executor/query.rs:48](), [src/executor/expression.rs]() (referenced), Diagram 4 (Expression Evaluation System)

## Timeout and Cancellation

The executor supports query timeouts and cancellation through the `TimeoutGuard` mechanism:

**Timeout setup** [src/executor/query.rs:163-176]():
1. `TimeoutGuard::new(ctx)` is created only at top level (`query_depth == 0`)
2. Sets up timeout deadline if `ctx.timeout` is specified
3. Spawns background thread to set cancellation flag on timeout
4. All subqueries and nested operations share the same timeout

**Cancellation checking:**
- `ctx.check_cancelled()` is called at strategic points [src/executor/query.rs:179]()
- Checks if timeout has elapsed or cancellation token is set
- Returns `Error::Timeout` or `Error::Cancelled` if triggered

**Benefits:**
- Prevents runaway queries
- Enables user-initiated cancellation
- Single timeout applies to entire query including subqueries

**Sources:** [src/executor/query.rs:163-176](), [src/executor/query.rs:179]()

## Subquery Caching

The executor maintains thread-local caches for subquery results to avoid redundant execution:

**Cache types:**
- **Scalar subquery cache**: Stores single-value results
- **IN subquery cache**: Stores result sets for IN clauses
- **Semi-join cache**: Stores EXISTS results
- **EXISTS predicate cache**: Caches EXISTS evaluation results
- **EXISTS index/schema caches**: Metadata caches for EXISTS optimization

**Cache clearing** [src/executor/query.rs:165-172]():
```
At top-level query entry (query_depth == 0):
  - clear_scalar_subquery_cache()
  - clear_in_subquery_cache()
  - clear_semi_join_cache()
  - clear_exists_predicate_cache()
  - clear_exists_index_cache()
  - clear_exists_fetcher_cache()
  - clear_exists_schema_cache()
  - clear_exists_pred_key_cache()
```

This ensures caches are fresh for each query while allowing reuse within a single query execution.

For detailed subquery execution strategies, see [Subquery Execution](#3.3).

**Sources:** [src/executor/query.rs:165-172](), [src/executor/context.rs:42-47]() (referenced)

## Related Subsystems

The Query Execution System integrates with and coordinates several other major subsystems:

| Subsystem | Integration Point | Purpose |
|-----------|-------------------|---------|
| [Storage Engine](#4) | `MVCCEngine`, `MVCCTable`, `MVCCScanner` | Provides MVCC-compliant row access with snapshot isolation |
| [Expression Evaluation](#3.2) | `ExprVM`, `Program`, `ExpressionEval` | Compiles and evaluates WHERE/SELECT/ORDER BY expressions |
| [Query Optimizer](#6.1) | `QueryPlanner`, `decide_join_algorithm` | Generates optimized access plans based on statistics |
| [Index System](#4.3) | `BTreeIndex`, `HashIndex`, `BitmapIndex` | Accelerates queries through indexed lookups |
| [Function Registry](#5.2) | `FunctionRegistry` | Provides 101+ built-in functions for expressions |
| [Aggregation Engine](#3.4) | `execute_select_with_aggregation` | Handles GROUP BY, ROLLUP, CUBE |
| [Window Functions](#3.5) | `execute_select_with_windows` | Processes OVER clauses and window frames |
| [CTE Engine](#3.6) | `execute_select_with_ctes` | Executes WITH clauses and recursive CTEs |

**Sources:** [src/executor/query.rs:1-66](), [src/lib.rs:56-73]()