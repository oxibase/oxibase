# Page: Common Table Expressions (CTEs)

# Common Table Expressions (CTEs)

<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

- [src/executor/cte.rs](src/executor/cte.rs)
- [src/executor/query.rs](src/executor/query.rs)

</details>



## Purpose and Scope

This page describes the Common Table Expression (CTE) execution system in OxiBase, which implements SQL `WITH` clauses. CTEs provide a way to define temporary named result sets that can be referenced within the scope of a query.

This page covers:
- Basic and chained CTE execution
- Recursive CTE processing with iteration limits
- CTE registry architecture for result materialization
- CTE inlining optimization to preserve index access
- Semi-join reduction for CTE-table JOINs

For information about subquery execution more broadly, see [Subquery Execution](#3.3). For JOIN execution details, see [Query Execution Pipeline](#3.1).

**Sources:** [src/executor/cte.rs:1-30]()

## Architecture Overview

The CTE execution system consists of three main components: the registry for storing materialized results, the execution engine that processes WITH clauses, and optimization layers that improve performance.

```mermaid
graph TB
    subgraph "Query Entry Point"
        SelectStmt["SelectStatement<br/>with: Option&lt;WithClause&gt;"]
    end
    
    subgraph "CTE Detection & Routing"
        HasCte["Executor::has_cte()<br/>src/executor/cte.rs:1172"]
        ExecuteSelect["Executor::execute_select<br/>src/executor/query.rs:155"]
        ExecuteWithCtes["Executor::execute_select_with_ctes<br/>src/executor/cte.rs:115"]
    end
    
    subgraph "Optimization Layer"
        TryInline["try_inline_ctes()<br/>Single-use CTE detection<br/>src/executor/cte.rs:1278"]
        InlinedStmt["Rewritten SelectStatement<br/>CTEs converted to subqueries"]
    end
    
    subgraph "CTE Registry"
        Registry["CteRegistry<br/>materialized: FxHashMap<br/>shared: Option&lt;Arc&lt;Map&gt;&gt;<br/>src/executor/cte.rs:53"]
        Store["store(name, columns, rows)<br/>src/executor/cte.rs:72"]
        Get["get(name) -> (columns, rows)<br/>src/executor/cte.rs:81"]
        Data["data() -> Arc&lt;Map&gt;<br/>src/executor/cte.rs:96"]
    end
    
    subgraph "CTE Execution"
        ExecuteCte["execute_cte_query()<br/>Materialize CTE result<br/>src/executor/cte.rs:189"]
        RecursiveCte["execute_recursive_cte_with_columns()<br/>UNION ALL iteration<br/>src/executor/cte.rs:250"]
    end
    
    subgraph "Main Query Execution"
        ExecuteMain["execute_main_query_with_ctes()<br/>src/executor/cte.rs:370"]
        CtxWithCtes["ExecutionContext::with_cte_data()<br/>Arc&lt;Map&gt; for subquery access"]
    end
    
    SelectStmt --> ExecuteSelect
    ExecuteSelect --> HasCte
    HasCte -->|"has WITH"| ExecuteWithCtes
    HasCte -->|"no WITH"| ExecuteSelect
    
    ExecuteWithCtes --> TryInline
    TryInline -->|"can inline"| InlinedStmt
    InlinedStmt --> ExecuteSelect
    TryInline -->|"must materialize"| Registry
    
    ExecuteWithCtes --> ExecuteCte
    ExecuteCte -->|"is_recursive"| RecursiveCte
    ExecuteCte -->|"not recursive"| ExecuteSelect
    
    ExecuteCte --> Store
    RecursiveCte --> Store
    Store --> Registry
    
    ExecuteWithCtes --> ExecuteMain
    Registry --> Data
    Data --> CtxWithCtes
    CtxWithCtes --> ExecuteMain
    
    ExecuteMain --> Get
```

**Diagram: CTE Execution Architecture**

The system follows a check-optimize-materialize-execute pattern. When a `SELECT` statement with a `WITH` clause is detected, it first attempts CTE inlining (converting single-use CTEs to subqueries). If inlining is not possible, CTEs are materialized in order, stored in the registry, and the main query executes with CTE data available in the execution context.

**Sources:** [src/executor/cte.rs:53-111](), [src/executor/cte.rs:115-186](), [src/executor/query.rs:190-193]()

## CTE Registry

The `CteRegistry` stores materialized CTE results during query execution. It uses a lazy `Arc` caching strategy to enable efficient sharing with nested execution contexts.

### Registry Structure

| Component | Type | Purpose |
|-----------|------|---------|
| `materialized` | `FxHashMap<String, (Vec<String>, Vec<Row>)>` | Stores CTE name → (columns, rows) mappings |
| `shared` | `Option<Arc<CteDataMap>>` | Cached Arc for cheap cloning to execution contexts |

```mermaid
graph LR
    subgraph "CteRegistry Internal State"
        Materialized["materialized<br/>FxHashMap"]
        Shared["shared<br/>Option&lt;Arc&gt;"]
    end
    
    subgraph "Storage Operations"
        Store["store(name, cols, rows)<br/>Invalidates cached Arc"]
        Get["get(name)<br/>Lookup by lowercase name"]
        Data["data()<br/>Clone Arc or create new"]
    end
    
    subgraph "Consumers"
        ExecutionContext["ExecutionContext<br/>with_cte_data()"]
        SubqueryExec["Subquery execution<br/>Needs CTE access"]
        JoinExec["JOIN execution<br/>CTE awareness"]
    end
    
    Store --> Materialized
    Store -.invalidates.-> Shared
    Get --> Materialized
    Data --> Shared
    Shared -.if None.-> Materialized
    
    Data --> ExecutionContext
    ExecutionContext --> SubqueryExec
    ExecutionContext --> JoinExec
```

**Diagram: CTE Registry Data Flow**

The registry maintains a cache of the Arc-wrapped data map. When `store()` is called, the cached Arc is invalidated. The first call to `data()` creates a new Arc, which is then reused for subsequent calls until the registry is modified again. This avoids repeated cloning of the entire CTE data map when creating nested execution contexts.

**Sources:** [src/executor/cte.rs:53-111]()

### Registry API

```
CteRegistry::new() -> Self
```
Creates an empty registry.

```
store(&mut self, name: &str, columns: Vec<String>, rows: Vec<Row>)
```
Stores a materialized CTE result. The name is converted to lowercase for case-insensitive lookup. Invalidates the cached Arc.

```
get(&self, name: &str) -> Option<(&Vec<String>, &Vec<Row>)>
```
Retrieves CTE results by name (case-insensitive).

```
data(&mut self) -> Arc<CteDataMap>
```
Returns an Arc-wrapped reference to the internal map. Creates and caches the Arc on first call after modification.

**Sources:** [src/executor/cte.rs:64-105]()

## CTE Execution Flow

CTEs are executed in dependency order, with results materialized before the main query runs. The system supports both simple and chained CTEs (where one CTE references another).

```mermaid
graph TB
    Start["execute_select_with_ctes()"]
    
    subgraph "Optimization Phase"
        CheckInline["try_inline_ctes()<br/>Check single-use CTEs"]
        CanInline{"All CTEs<br/>single-use?"}
        RewriteQuery["Convert CTEs to subqueries<br/>Remove WITH clause"]
        ExecuteInlined["execute_select()<br/>on rewritten query"]
    end
    
    subgraph "Materialization Phase"
        CreateRegistry["Create CteRegistry"]
        ForEachCte["For each CTE in order"]
        CheckRecursive{"is_recursive?"}
        ExecuteRecursive["execute_recursive_cte_with_columns()<br/>UNION ALL iteration"]
        ExecuteNormal["execute_cte_query()<br/>Normal SELECT"]
        ApplyAliases["Apply column aliases<br/>if specified"]
        StoreCte["registry.store(name, cols, rows)"]
    end
    
    subgraph "Main Query Phase"
        ShareData["registry.data() -> Arc"]
        CreateCtx["ctx.with_cte_data(Arc)"]
        ExecuteMain["execute_main_query_with_ctes()"]
        CheckCteTables{"Main query<br/>refs CTE?"}
        FastPath["execute_query_on_cte_result()<br/>Direct CTE filtering"]
        NormalPath["execute_select()<br/>with CTE context"]
    end
    
    Start --> CheckInline
    CheckInline --> CanInline
    CanInline -->|Yes| RewriteQuery
    RewriteQuery --> ExecuteInlined
    ExecuteInlined --> Return["Return QueryResult"]
    
    CanInline -->|No| CreateRegistry
    CreateRegistry --> ForEachCte
    ForEachCte --> CheckRecursive
    CheckRecursive -->|Yes| ExecuteRecursive
    CheckRecursive -->|No| ExecuteNormal
    ExecuteRecursive --> ApplyAliases
    ExecuteNormal --> ApplyAliases
    ApplyAliases --> StoreCte
    StoreCte -->|More CTEs| ForEachCte
    StoreCte -->|Done| ShareData
    
    ShareData --> CreateCtx
    CreateCtx --> ExecuteMain
    ExecuteMain --> CheckCteTables
    CheckCteTables -->|Yes| FastPath
    CheckCteTables -->|No| NormalPath
    FastPath --> Return
    NormalPath --> Return
```

**Diagram: CTE Execution Flow**

The execution flow has three phases:
1. **Optimization**: Attempts to inline single-use CTEs as subqueries
2. **Materialization**: Executes each CTE in order, storing results in the registry
3. **Main Query**: Executes the main query with CTE data available in the context

**Sources:** [src/executor/cte.rs:115-186](), [src/executor/cte.rs:370-437]()

### CTE Query Execution

When executing a CTE query, the system handles several cases:

1. **Simple CTE**: Standard SELECT query materialized once
2. **CTE referencing another CTE**: Uses registry lookup to access previously materialized CTEs
3. **CTE with JOIN**: Special handling when joining CTEs together
4. **Recursive CTE**: Iterative execution with UNION ALL (see next section)

```mermaid
graph TB
    ExecuteCte["execute_cte_query()"]
    
    CheckRef{"Refs other CTE?"}
    GetCte["registry.get(cte_name)"]
    ExecOnCte["execute_query_on_cte_result()<br/>Filter/project CTE data"]
    
    CheckJoin{"Has JOIN?"}
    CheckJoinCte{"Either side<br/>is CTE?"}
    ExecJoinCte["try_execute_join_with_ctes()<br/>CTE-aware JOIN logic"]
    
    CreateCtx["ctx.with_cte_data(registry.data())"]
    ExecNormal["execute_select()<br/>Normal query path"]
    
    Materialize["Materialize QueryResult<br/>Collect all rows"]
    Return["Return (columns, rows)"]
    
    ExecuteCte --> CheckRef
    CheckRef -->|Yes| GetCte
    GetCte --> ExecOnCte
    ExecOnCte --> Return
    
    CheckRef -->|No| CheckJoin
    CheckJoin -->|Yes| CheckJoinCte
    CheckJoinCte -->|Yes| ExecJoinCte
    ExecJoinCte --> Materialize
    Materialize --> Return
    
    CheckJoinCte -->|No| CreateCtx
    CheckJoin -->|No| CreateCtx
    CreateCtx --> ExecNormal
    ExecNormal --> Materialize
```

**Diagram: CTE Query Execution Logic**

The system optimizes CTE-to-CTE queries by directly operating on materialized data rather than re-scanning tables. For JOINs involving CTEs, specialized logic handles semi-join reduction (see later section).

**Sources:** [src/executor/cte.rs:189-247]()

## Recursive CTEs

Recursive CTEs use iterative execution with `UNION ALL` to compute results that depend on previous iterations. The system enforces a maximum iteration limit to prevent infinite loops.

### Recursive CTE Structure

A recursive CTE must have the form:
```
WITH RECURSIVE cte_name AS (
  <anchor_member>      -- Initial query
  UNION ALL
  <recursive_member>   -- References cte_name
)
```

The anchor member runs once to produce the initial working set. The recursive member runs iteratively, each time referencing the results from the previous iteration, until no new rows are produced or the maximum iteration limit is reached.

```mermaid
graph TB
    Start["execute_recursive_cte_with_columns()"]
    
    subgraph "Validation"
        CheckUnion{"Has UNION ALL?"}
        CheckAllUnion{"All set ops<br/>UNION ALL?"}
        Error["Error: Recursive CTE<br/>must use UNION ALL"]
    end
    
    subgraph "Anchor Execution"
        BuildAnchor["Build anchor SelectStatement<br/>First query before UNION ALL"]
        ExecAnchor["execute_select(anchor_stmt)"]
        ApplyAliases["Apply column aliases<br/>if provided"]
        CheckEmpty{"Anchor rows<br/>empty?"}
        ReturnEmpty["Return (columns, [])"]
    end
    
    subgraph "Recursive Iteration"
        InitWorking["working_rows = anchor_rows<br/>all_rows = anchor_rows"]
        IterLoop["For iteration in 0..MAX_ITERATIONS"]
        CheckWorking{"working_rows<br/>empty?"}
        CreateTemp["Create temp CteRegistry<br/>Copy existing CTEs<br/>Add current CTE with working_rows"]
        ExecRecursive["For each UNION ALL member:<br/>execute_cte_query()"]
        Collect["Collect new_rows<br/>from all members"]
        CheckNew{"new_rows<br/>empty?"}
        Extend["all_rows.extend(new_rows)<br/>working_rows = new_rows"]
        Continue["Continue iteration"]
        MaxReached["MAX_ITERATIONS reached"]
    end
    
    Return["Return (columns, all_rows)"]
    
    Start --> CheckUnion
    CheckUnion -->|No| Error
    CheckUnion -->|Yes| CheckAllUnion
    CheckAllUnion -->|No| Error
    CheckAllUnion -->|Yes| BuildAnchor
    
    BuildAnchor --> ExecAnchor
    ExecAnchor --> ApplyAliases
    ApplyAliases --> CheckEmpty
    CheckEmpty -->|Yes| ReturnEmpty
    CheckEmpty -->|No| InitWorking
    
    InitWorking --> IterLoop
    IterLoop --> CheckWorking
    CheckWorking -->|Yes| Return
    CheckWorking -->|No| CreateTemp
    CreateTemp --> ExecRecursive
    ExecRecursive --> Collect
    Collect --> CheckNew
    CheckNew -->|Yes| Return
    CheckNew -->|No| Extend
    Extend --> Continue
    Continue --> IterLoop
    IterLoop -.max reached.-> MaxReached
    MaxReached --> Return
```

**Diagram: Recursive CTE Execution Flow**

The execution follows these steps:
1. Validate that all set operations are `UNION ALL` (non-recursive `UNION` is not supported)
2. Execute the anchor member to get initial rows
3. Iterate: Add current working set to temporary registry, execute recursive members, collect new rows
4. Stop when no new rows are produced or `MAX_ITERATIONS` (10,000) is reached

**Sources:** [src/executor/cte.rs:250-367]()

### Iteration Control

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_ITERATIONS` | 10,000 | Prevents infinite loops in recursive CTEs |

The system tracks three row sets during iteration:
- `all_rows`: Accumulated results from all iterations (returned at the end)
- `working_rows`: Results from the previous iteration (used as input to current iteration)
- `new_rows`: Results from the current iteration (becomes next `working_rows`)

**Sources:** [src/executor/cte.rs:261-366]()

## CTE Inlining Optimization

Single-use, non-recursive CTEs are converted to subqueries to preserve index access and enable optimizations like LIMIT pushdown. This is similar to PostgreSQL 12+ behavior where CTEs are no longer optimization barriers by default.

### Inlining Eligibility

A CTE can be inlined if ALL of the following conditions are met:

| Condition | Reason |
|-----------|--------|
| Not recursive | Recursive CTEs must be materialized for iteration |
| No column aliases | Column renaming requires materialization |
| Doesn't reference other CTEs | CTE chaining creates data dependencies |
| Used exactly once in FROM/JOIN | Multi-use benefits from materialization |
| Not used in WHERE clause | WHERE subqueries have different execution paths |

```mermaid
graph TB
    Start["try_inline_ctes(stmt, with_clause)"]
    
    subgraph "CTE Analysis"
        BuildMap["Build cte_defs map<br/>name -> CTE definition"]
        CheckRecursive{"Any CTE<br/>is_recursive?"}
        CheckAliases{"Any CTE has<br/>column_names?"}
        CheckChaining{"Any CTE<br/>refs other CTE?"}
        ReturnNone1["Return None<br/>Cannot inline"]
    end
    
    subgraph "Usage Analysis"
        InitCounts["table_ref_counts = {}<br/>where_ref_counts = {}"]
        CountTable["count_cte_references_in_expr<br/>in stmt.table_expr"]
        CountWhere["count_cte_references_in_expr<br/>in stmt.where_clause"]
        
        CheckUsage["For each CTE:<br/>Check ref counts"]
        WhereRefs{"Used in<br/>WHERE?"}
        MultiUse{"table_refs<br/>> 1?"}
        ReturnNone2["Return None<br/>Cannot inline"]
    end
    
    subgraph "Inlining Transformation"
        CreateNew["Create new SelectStatement<br/>with = None"]
        InlineRefs["inline_cte_references()<br/>Replace CTE refs with SubquerySource"]
        ReturnInlined["Return Some(new_stmt)"]
    end
    
    Start --> BuildMap
    BuildMap --> CheckRecursive
    CheckRecursive -->|Yes| ReturnNone1
    CheckRecursive -->|No| CheckAliases
    CheckAliases -->|Yes| ReturnNone1
    CheckAliases -->|No| CheckChaining
    CheckChaining -->|Yes| ReturnNone1
    CheckChaining -->|No| InitCounts
    
    InitCounts --> CountTable
    CountTable --> CountWhere
    CountWhere --> CheckUsage
    CheckUsage --> WhereRefs
    WhereRefs -->|Yes| ReturnNone2
    WhereRefs -->|No| MultiUse
    MultiUse -->|Yes| ReturnNone2
    MultiUse -->|No| CreateNew
    
    CreateNew --> InlineRefs
    InlineRefs --> ReturnInlined
```

**Diagram: CTE Inlining Decision Logic**

The inlining optimization performs a two-phase analysis:
1. **CTE Properties**: Check for recursive CTEs, column aliases, and cross-CTE dependencies
2. **Usage Patterns**: Count references in FROM/JOIN vs WHERE clauses

If all CTEs pass the eligibility checks, they are converted to `SubquerySource` expressions and the `WITH` clause is removed.

**Sources:** [src/executor/cte.rs:1278-1364]()

### Inlining Transformation

The `inline_cte_references()` function recursively walks the query AST, replacing CTE references with subquery sources:

```mermaid
graph LR
    Before1["CteReference<br/>name: 'my_cte'<br/>alias: Some('c')"]
    After1["SubquerySource<br/>subquery: CTE's SelectStatement<br/>alias: Some('c')"]
    
    Before2["TableSource<br/>name: 'my_cte'<br/>alias: Some('c')"]
    After2["SubquerySource<br/>subquery: CTE's SelectStatement<br/>alias: Some('c')"]
    
    Before1 -.inline_cte_references.-> After1
    Before2 -.inline_cte_references.-> After2
```

**Diagram: CTE Reference Transformation**

The transformation preserves aliases to maintain correct column resolution in the rest of the query. After inlining, the query executes through the normal subquery path, which can leverage indexes and other optimizations.

**Sources:** [src/executor/cte.rs:1493-1557]()

### Performance Impact

| Scenario | Without Inlining | With Inlining |
|----------|------------------|---------------|
| CTE with index on filtered column | Full table scan (indexes lost in materialization) | Index scan (subquery preserves indexes) |
| CTE with LIMIT in main query | Materialize all CTE rows, then LIMIT | LIMIT pushed through subquery |
| Single-use CTE | Allocate storage for materialized data | Zero additional memory allocation |

**Sources:** [src/executor/cte.rs:126-134]()

## Semi-Join Reduction for CTE JOINs

When joining a CTE with a regular table on an indexed column, the system extracts join keys from the CTE to filter the table scan via an `IN` clause. This avoids materializing the entire table and leverages indexes.

### Semi-Join Applicability

Semi-join reduction is applied when:
1. One side is a CTE, the other is a regular table
2. The join is an `INNER JOIN`
3. The CTE has ≤ 500 rows (configurable threshold)
4. The join condition is an equality on a single column

```mermaid
graph TB
    Start["execute_cte_join_with_semijoin_reduction()"]
    
    subgraph "CTE Side"
        CteCols["CTE columns, rows<br/>Already materialized"]
        ExtractKeys["Extract join key column<br/>from condition"]
        FindIdx["Find column index<br/>in CTE columns"]
        CollectVals["Collect distinct non-NULL values<br/>from CTE rows[idx]"]
        CheckSize{"CTE size<br/><= MAX_SEMIJOIN_SIZE?"}
        UseOptimization["Build IN filter"]
        SkipOptimization["Skip optimization<br/>Hash join faster"]
    end
    
    subgraph "Table Side"
        TableExpr["Regular table expression"]
        BuildFilter["Build filter:<br/>table_col IN (v1, v2, ...)"]
        ExecFiltered["execute_table_expression_with_filter()<br/>IN clause can use index"]
        ExecNormal["execute_table_expression()<br/>Full table scan"]
        Materialize["Materialize table rows"]
    end
    
    subgraph "JOIN Execution"
        OrderSides{"CTE on left<br/>or right?"}
        ReturnLeft["Return (cte_cols, cte_rows,<br/>table_cols, table_rows)"]
        ReturnRight["Return (table_cols, table_rows,<br/>cte_cols, cte_rows)"]
    end
    
    Start --> CteCols
    CteCols --> ExtractKeys
    ExtractKeys --> FindIdx
    FindIdx --> CollectVals
    CollectVals --> CheckSize
    CheckSize -->|Yes| UseOptimization
    CheckSize -->|No| SkipOptimization
    
    UseOptimization --> BuildFilter
    SkipOptimization --> ExecNormal
    
    BuildFilter --> ExecFiltered
    ExecFiltered --> Materialize
    ExecNormal --> Materialize
    
    Materialize --> OrderSides
    OrderSides -->|Left| ReturnLeft
    OrderSides -->|Right| ReturnRight
```

**Diagram: Semi-Join Reduction Flow**

The optimization transforms:
```
SELECT * FROM cte JOIN table ON cte.id = table.cte_id
```
into:
```
SELECT * FROM cte JOIN table WHERE table.cte_id IN (values from cte.id)
```

This allows the table scan to use an index on `cte_id` (if one exists) instead of scanning all rows.

**Sources:** [src/executor/cte.rs:721-817]()

### Join Key Extraction

The system uses qualifier-aware key extraction to correctly identify which column belongs to which side of the join:

```mermaid
graph TB
    Condition["JOIN condition:<br/>u.id = h.user_id"]
    
    Extract["extract_join_key_columns_with_qualifiers()"]
    
    CheckInfix{"Operator<br/>is '='?"}
    ParseLeft["Parse left:<br/>QualifiedIdentifier(u, id)"]
    ParseRight["Parse right:<br/>QualifiedIdentifier(h, user_id)"]
    
    MatchQuals["Match qualifiers<br/>to known aliases"]
    CheckMatch{"Qualifiers<br/>match CTE/table?"}
    
    ReturnKeys["Return:<br/>(table_col: 'user_id',<br/>cte_col: 'id')"]
    ReturnNone["Return (None, None)"]
    
    Condition --> Extract
    Extract --> CheckInfix
    CheckInfix -->|Yes| ParseLeft
    ParseLeft --> ParseRight
    ParseRight --> MatchQuals
    MatchQuals --> CheckMatch
    CheckMatch -->|Yes| ReturnKeys
    CheckMatch -->|No| ReturnNone
    CheckInfix -->|No| ReturnNone
```

**Diagram: Qualifier-Aware Key Extraction**

The function `extract_join_key_columns_with_qualifiers()` correctly handles both `cte.col = table.col` and `table.col = cte.col` by using the qualifiers (table aliases) to determine which column belongs to which side.

**Sources:** [src/executor/cte.rs:822-871]()

### Performance Characteristics

| CTE Size | Regular Table Size | Index on Join Column | Strategy | Complexity |
|----------|-------------------|---------------------|----------|------------|
| ≤ 500 rows | Any | Yes | Semi-join with index | O(cte_size + log(table_size)) |
| ≤ 500 rows | Any | No | Semi-join with scan | O(cte_size + table_size) |
| > 500 rows | Any | Any | Hash join | O(cte_size + table_size) |

The 500-row threshold (`MAX_SEMIJOIN_SIZE`) balances the cost of:
- Building the IN filter from CTE values
- Potential index lookups vs full table hash join

For small CTEs, the IN filter is cheap to build and can leverage indexes. For large CTEs, hash join on the full table is faster.

**Sources:** [src/executor/cte.rs:746-761]()

## Integration with Query Execution

CTEs integrate with the broader query execution system through the `ExecutionContext`, which carries CTE data to nested queries and subqueries.

```mermaid
graph TB
    subgraph "Top-Level Query"
        ParseQuery["Parse SQL with WITH clause"]
        ExecuteSelect["Executor::execute_select()"]
        CheckCte["has_cte(stmt)"]
        RouteExecution{"Has WITH?"}
    end
    
    subgraph "CTE Materialization"
        ExecWithCtes["execute_select_with_ctes()"]
        MaterializeCtes["For each CTE:<br/>execute_cte_query()"]
        StoreRegistry["CteRegistry::store()"]
        ShareData["registry.data() -> Arc&lt;Map&gt;"]
    end
    
    subgraph "Context Creation"
        BaseCtx["ExecutionContext"]
        WithCteData["ctx.with_cte_data(Arc)"]
        NestedCtx["New context with<br/>cte_data: Some(Arc)"]
    end
    
    subgraph "Main Query & Subqueries"
        ExecMain["execute_main_query_with_ctes()"]
        CheckCteRef{"Query refs CTE?"}
        GetCte["ctx.get_cte(name)"]
        ExecOnCte["execute_query_on_cte_result()"]
        
        CheckSubquery{"Has subquery?"}
        PassCtx["Pass context to subquery"]
        SubqueryCanAccessCte["Subquery can access CTEs<br/>via ctx.get_cte()"]
    end
    
    ParseQuery --> ExecuteSelect
    ExecuteSelect --> CheckCte
    CheckCte --> RouteExecution
    RouteExecution -->|Yes| ExecWithCtes
    RouteExecution -->|No| BaseCtx
    
    ExecWithCtes --> MaterializeCtes
    MaterializeCtes --> StoreRegistry
    StoreRegistry --> ShareData
    ShareData --> WithCteData
    WithCteData --> NestedCtx
    
    NestedCtx --> ExecMain
    ExecMain --> CheckCteRef
    CheckCteRef -->|Yes| GetCte
    GetCte --> ExecOnCte
    
    ExecMain --> CheckSubquery
    CheckSubquery -->|Yes| PassCtx
    PassCtx --> SubqueryCanAccessCte
```

**Diagram: CTE Integration with Execution Context**

The `ExecutionContext` carries CTE data through multiple execution layers:
1. CTEs are materialized and stored in the registry
2. The registry data is wrapped in an `Arc` for cheap sharing
3. The Arc is attached to the execution context via `with_cte_data()`
4. Subqueries and nested queries receive the context and can access CTEs via `get_cte()`

This design allows CTEs to be referenced from:
- The main query's FROM clause
- JOIN expressions
- Subqueries in WHERE/SELECT/HAVING clauses
- Nested CTEs (CTEs referencing other CTEs)

**Sources:** [src/executor/cte.rs:370-437](), [src/executor/query.rs:754-763]()

### Context CTE Access

The `ExecutionContext` provides the `get_cte()` method for looking up CTE data:

```
impl ExecutionContext {
    pub fn get_cte(&self, name: &str) -> Option<(&Vec<String>, &Vec<Row>)>
}
```

This method:
1. Checks if `cte_data` is present in the context
2. Performs a case-insensitive lookup by converting `name` to lowercase
3. Returns references to the CTE's columns and rows (avoiding clones)

**Sources:** [src/executor/context.rs]() (referenced in cte.rs), [src/executor/query.rs:754-763]()

### CTE Query Execution Path

When the main query references a CTE as a table source:

```mermaid
graph LR
    TableExpr["table_expr:<br/>Expression::TableSource<br/>name: 'my_cte'"]
    
    CheckCtx["ctx.get_cte('my_cte')"]
    
    Found{"CTE found?"}
    
    ExecOnCte["execute_query_on_cte_result()<br/>Apply WHERE/SELECT/ORDER BY<br/>to CTE rows"]
    
    CheckView["engine.get_view_lowercase()"]
    
    IsView{"View found?"}
    
    ExecView["execute_view_query()"]
    
    RegularTable["execute_simple_table_scan()"]
    
    TableExpr --> CheckCtx
    CheckCtx --> Found
    Found -->|Yes| ExecOnCte
    Found -->|No| CheckView
    CheckView --> IsView
    IsView -->|Yes| ExecView
    IsView -->|No| RegularTable
```

**Diagram: Table Resolution Order**

The executor checks table sources in this priority order:
1. CTE from execution context (if available)
2. View from storage engine
3. Regular table

This allows CTEs to shadow views and tables with the same name, following SQL standard scoping rules.

**Sources:** [src/executor/query.rs:754-770]()