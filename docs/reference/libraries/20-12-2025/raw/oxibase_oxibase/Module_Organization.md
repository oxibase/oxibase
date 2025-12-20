# Page: Module Organization

# Module Organization

<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

- [.github/workflows/ci.yml](.github/workflows/ci.yml)
- [.gitignore](.gitignore)
- [Cargo.toml](Cargo.toml)
- [README.md](README.md)
- [ROADMAP.md](ROADMAP.md)
- [docs/_config.yml](docs/_config.yml)
- [src/lib.rs](src/lib.rs)

</details>



This page documents the codebase structure of OxiBase, including the module hierarchy, dependencies between modules, key types exported from each module, and how the modules map to system architecture layers.

For information about building and testing the codebase, see [Building and Testing](#7.1). For high-level system architecture concepts, see [Architecture Overview](#1.2).

---

## Module Structure

OxiBase is organized into eight top-level modules under `src/`, each serving a distinct architectural role. The codebase follows a strict layered architecture where higher-level modules depend on lower-level modules but never the reverse.

```mermaid
graph TB
    api["api/
    Database, Transaction, Rows
    User-facing API layer"]
    
    executor["executor/
    Query execution pipeline
    Planner, Optimizer, Executor"]
    
    functions["functions/
    101+ built-in functions
    scalar/, aggregate/, window/"]
    
    parser["parser/
    SQL lexer and parser
    AST generation"]
    
    storage["storage/
    MVCC storage engine
    mvcc/, index/, expressions/"]
    
    core["core/
    Fundamental types
    Value, Row, Schema, Error"]
    
    common["common/
    Shared utilities
    BufferPool, Maps, Version"]
    
    bin["bin/
    CLI application
    oxibase.rs"]
    
    api --> executor
    api --> storage
    
    executor --> functions
    executor --> parser
    executor --> storage
    executor --> core
    
    functions --> core
    
    parser --> core
    
    storage --> core
    storage --> common
    
    bin --> api
```

**Sources:** [README.md:85-100](), [src/lib.rs:66-73]()

---

## Module Layers and Dependencies

The modules are organized into distinct architectural layers, with strict dependency rules enforcing separation of concerns.

### Layer 1: Foundation (No Dependencies)

**`common/` - Shared Utilities**

Provides reusable data structures and utilities with no dependencies on other OxiBase modules.

| Component | Purpose |
|-----------|---------|
| `BufferPool` | Memory pooling for zero-allocation row processing |
| `Int64Map`, `UInt64Map`, `UsizeMap` | Specialized hash maps for integer keys |
| `ConcurrentInt64Map` | Thread-safe integer map variants |
| `SemVer` | Semantic versioning for persistence format |
| `PoolStats` | Buffer pool statistics tracking |

**Sources:** [src/lib.rs:82-85]()

---

**`core/` - Fundamental Types**

Defines the core data model used throughout the system.

```mermaid
graph LR
    subgraph "core/"
        Value["Value
        Integer/Float/Text/
        Boolean/Timestamp/JSON"]
        
        Row["Row
        Vec&lt;Value&gt;
        Represents a single row"]
        
        Schema["Schema
        Table metadata
        Column definitions"]
        
        DataType["DataType
        Type system
        INTEGER/FLOAT/TEXT/etc"]
        
        Error["Error
        oxibase::Error
        thiserror-based"]
        
        IndexType["IndexType
        BTree/Hash/Bitmap"]
        
        Operator["Operator
        Eq/Lt/Gt/And/Or/etc"]
    end
    
    Row --> Value
    Schema --> DataType
```

**Key Exports:**
- `Value` - Enum representing all supported SQL types
- `Row` - Struct containing a vector of Values
- `Schema` / `SchemaBuilder` / `SchemaColumn` - Table schema definitions
- `DataType` - Type enumeration (INTEGER, FLOAT, TEXT, BOOLEAN, TIMESTAMP, JSON)
- `Error` / `Result` - Error type used throughout codebase
- `IndexType` - Index type enumeration (BTree, Hash, Bitmap)
- `IsolationLevel` - Transaction isolation levels
- `Operator` - Binary and unary operators

**Sources:** [src/lib.rs:76-79]()

---

### Layer 2: Core Systems (Depends on Foundation)

**`parser/` - SQL Parser**

Tokenizes and parses SQL statements into Abstract Syntax Trees (AST).

```mermaid
graph LR
    SQL["SQL String
    'SELECT * FROM users'"]
    
    Lexer["Lexer
    Token stream"]
    
    Parser["Parser
    AST generation"]
    
    AST["SelectStatement
    FromClause/WhereExpr/etc"]
    
    SQL --> Lexer --> Parser --> AST
```

**Dependencies:** `core` (for DataType, Operator, Value constants)

The parser module converts SQL text into structured AST nodes that can be processed by the planner and executor. Each SQL statement type (SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, etc.) has a corresponding AST struct.

**Sources:** [README.md:89](), [src/lib.rs:72]()

---

**`storage/` - MVCC Storage Engine**

Implements multi-version concurrency control, persistence, and indexing.

```mermaid
graph TB
    subgraph "storage/"
        subgraph "mvcc/"
            MVCCEngine["MVCCEngine
            Transaction coordinator"]
            
            VersionStore["VersionStore
            Version chains
            Arena-based storage"]
            
            TransactionRegistry["TransactionRegistry
            begin_seq/commit_seq
            Visibility checking"]
            
            MvccTransaction["MvccTransaction
            Transaction handle"]
            
            MVCCTable["MVCCTable
            Transaction-aware table API"]
        end
        
        subgraph "index/"
            BTreeIndex["BTreeIndex
            Range queries"]
            
            HashIndex["HashIndex
            Equality lookups"]
            
            BitmapIndex["BitmapIndex
            Low cardinality"]
        end
        
        subgraph "Persistence"
            WALManager["WALManager
            Write-ahead log"]
            
            PersistenceManager["PersistenceManager
            Snapshots & recovery"]
        end
        
        subgraph "expressions/"
            Expression["Expression trait
            Filter predicates"]
            
            ExprVM["ExprVM
            Bytecode evaluation"]
        end
    end
    
    MVCCEngine --> VersionStore
    MVCCEngine --> TransactionRegistry
    MVCCEngine --> WALManager
    MVCCEngine --> PersistenceManager
    
    MvccTransaction --> MVCCTable
    MVCCTable --> VersionStore
    
    VersionStore --> BTreeIndex
    VersionStore --> HashIndex
    VersionStore --> BitmapIndex
    
    MVCCTable --> Expression
    Expression --> ExprVM
```

**Key Types Exported:**

| Category | Types |
|----------|-------|
| **MVCC Core** | `MVCCEngine`, `MvccTransaction`, `MVCCTable`, `VersionStore`, `TransactionRegistry` |
| **Indexes** | `BTreeIndex`, `HashIndex`, `BitmapIndex` (via `storage::index`) |
| **Persistence** | `WALManager`, `PersistenceManager`, `CheckpointMetadata`, `WALEntry` |
| **Expressions** | `Expression` trait, `ComparisonExpr`, `AndExpr`, `OrExpr`, `ExprVM` |
| **Traits** | `Engine`, `Transaction`, `Table`, `Scanner`, `Index` |
| **Version Data** | `RowVersion`, `TransactionVersionStore`, `WriteSetEntry` |

**Dependencies:** `core` (Value, Row, Schema), `common` (BufferPool, maps)

**Sources:** [src/lib.rs:88-125](), [README.md:97-99]()

---

**`functions/` - Built-in SQL Functions**

Implements 101+ SQL functions organized by category.

```mermaid
graph TB
    FunctionRegistry["FunctionRegistry
    Lookup by name"]
    
    subgraph "functions/"
        subgraph "scalar/"
            String["String Functions
            UPPER/LOWER/CONCAT/
            SUBSTRING/LENGTH/etc"]
            
            Math["Math Functions
            ABS/ROUND/CEIL/FLOOR/
            SQRT/POWER/etc"]
            
            DateTime["Date/Time Functions
            NOW/EXTRACT/DATE_TRUNC/
            DATE_ADD/etc"]
            
            JSON["JSON Functions
            JSON_EXTRACT/JSON_TYPE/
            JSON_VALID/etc"]
        end
        
        subgraph "aggregate/"
            Agg["Aggregate Functions
            COUNT/SUM/AVG/MIN/MAX/
            STDDEV/STRING_AGG/etc"]
        end
        
        subgraph "window/"
            Window["Window Functions
            ROW_NUMBER/RANK/
            LAG/LEAD/NTILE/etc"]
        end
    end
    
    FunctionRegistry --> String
    FunctionRegistry --> Math
    FunctionRegistry --> DateTime
    FunctionRegistry --> JSON
    FunctionRegistry --> Agg
    FunctionRegistry --> Window
```

**Key Types:**
- `FunctionRegistry` - Central registry for function lookup
- `ScalarFunction` trait - Single-row functions
- `AggregateFunction` trait - Aggregating functions (COUNT, SUM, etc.)
- `WindowFunction` trait - Window/analytical functions
- `FunctionSignature` / `FunctionInfo` - Function metadata
- Concrete implementations: `AbsFunction`, `UpperFunction`, `CountFunction`, `RowNumberFunction`, etc.

**Dependencies:** `core` (Value, DataType)

**Sources:** [src/lib.rs:127-139](), [README.md:94-96]()

---

### Layer 3: Query Processing (Depends on Core + Storage)

**`executor/` - Query Execution Engine**

Orchestrates query planning, optimization, and execution.

```mermaid
graph TB
    subgraph "executor/"
        QueryPlanner["QueryPlanner
        Logical plan generation"]
        
        Optimizer["Cost-based Optimizer
        Predicate pushdown
        Join ordering"]
        
        Executor["Executor
        Physical execution
        Parallel processing"]
        
        subgraph "Specialized Executors"
            QueryExec["Query Executor
            SELECT processing"]
            
            AggExec["Aggregation Executor
            GROUP BY/ROLLUP/CUBE"]
            
            WindowExec["Window Executor
            OVER clauses"]
            
            SubqueryExec["Subquery Executor
            EXISTS/IN/Correlated"]
            
            CTEExec["CTE Executor
            WITH/RECURSIVE"]
        end
        
        ExecContext["ExecutionContext
        Runtime state"]
        
        QueryCache["QueryCache
        Semantic caching"]
        
        ColumnStatsCache["ColumnStatsCache
        Statistics for optimizer"]
    end
    
    QueryPlanner --> Optimizer
    Optimizer --> Executor
    
    Executor --> QueryExec
    Executor --> AggExec
    Executor --> WindowExec
    Executor --> SubqueryExec
    Executor --> CTEExec
    
    Executor --> ExecContext
    Optimizer --> QueryCache
    Optimizer --> ColumnStatsCache
```

**Key Types:**
- `Executor` - Main execution engine
- `QueryPlanner` - Converts parsed AST to logical plans
- `ExecutionContext` - Runtime context for query execution
- `AccessPlan` / `JoinPlan` - Physical execution plans
- `QueryCache` / `CachedQueryPlan` - Semantic query result caching
- `ColumnStatsCache` - Statistics for cost-based optimization
- `ExecResult` / `ExecutorMemoryResult` - Execution results

**Dependencies:** `parser` (AST), `functions` (function registry), `storage` (MVCC, indexes), `core` (Value, Row)

**Sources:** [src/lib.rs:142-145](), [README.md:92]()

---

### Layer 4: Public API (Depends on All Layers)

**`api/` - User-Facing Database API**

Provides the public interface for application developers.

```mermaid
graph TB
    subgraph "api/"
        Database["Database
        Primary entrypoint
        open_in_memory/open"]
        
        Transaction["Transaction
        ACID transaction handle
        execute/query/commit/rollback"]
        
        Statement["Statement
        Prepared statement
        Parameterized queries"]
        
        Rows["Rows
        Iterator over results
        Result streaming"]
        
        ResultRow["ResultRow
        Single row accessor
        get&lt;T&gt; with type conversion"]
        
        Params["Params trait
        Named/positional parameters"]
        
        FromValue["FromValue trait
        Type conversion from Value"]
        
        FromRow["FromRow trait
        Struct deserialization"]
    end
    
    Database --> Transaction
    Database --> Rows
    
    Transaction --> Statement
    Transaction --> Rows
    
    Statement --> Rows
    
    Rows --> ResultRow
    ResultRow --> FromValue
    
    Params -.implemented by.-> NamedParams
    Params -.implemented by.-> TupleParams
```

**Key Types:**
- `Database` - Main database handle with `open_in_memory()` and `open()` constructors
- `Transaction` (as `ApiTransaction` in lib.rs) - Transaction handle
- `Rows` - Iterator over query results
- `ResultRow` - Single row with typed accessors
- `Statement` - Prepared statement
- `Params` / `NamedParams` - Parameter binding
- `FromValue` / `ToParam` - Type conversion traits
- `FromRow` - Automatic struct deserialization

**Dependencies:** `executor` (query execution), `storage` (MVCC), `parser` (SQL parsing), `core` (Value, Row, Schema)

**Sources:** [src/lib.rs:148-151](), [README.md:87]()

---

### Binary Crate

**`bin/oxibase.rs` - CLI Application**

Command-line REPL and query execution tool.

```mermaid
graph LR
    CLI["CLI Arguments
    --db/--query/-q"]
    
    REPL["Interactive REPL
    rustyline-based"]
    
    Direct["Direct Query Mode
    Execute and exit"]
    
    Database["Database API
    api::Database"]
    
    CLI --> REPL
    CLI --> Direct
    
    REPL --> Database
    Direct --> Database
```

**Features:**
- Interactive REPL with history (uses `rustyline`)
- Direct query execution with `-q` flag
- Pretty-printed table output (uses `comfy-table`)
- DSN-based configuration (memory:// or file:///)

**Dependencies:** `api` module, plus CLI-specific dependencies (`clap`, `rustyline`, `comfy-table`)

**Conditional Compilation:** Only built when `cli` feature is enabled (default)

**Sources:** [Cargo.toml:19-22](), [Cargo.toml:95-96]()

---

## Module Dependency Graph

This diagram shows all module dependencies and how they respect architectural layers.

```mermaid
graph TB
    subgraph "Layer 4 - Public API"
        api["api"]
        bin["bin/oxibase"]
    end
    
    subgraph "Layer 3 - Query Processing"
        executor["executor"]
    end
    
    subgraph "Layer 2 - Core Systems"
        parser["parser"]
        storage["storage"]
        functions["functions"]
    end
    
    subgraph "Layer 1 - Foundation"
        core["core"]
        common["common"]
    end
    
    subgraph "External Dependencies"
        ext1["thiserror
        anyhow"]
        
        ext2["parking_lot
        dashmap
        crossbeam
        rayon"]
        
        ext3["serde
        serde_json
        chrono"]
        
        ext4["clap
        rustyline
        comfy-table"]
    end
    
    bin --> api
    bin --> ext4
    
    api --> executor
    api --> storage
    
    executor --> parser
    executor --> functions
    executor --> storage
    executor --> core
    
    functions --> core
    
    parser --> core
    
    storage --> core
    storage --> common
    
    core --> ext1
    core --> ext3
    
    storage --> ext2
    storage --> ext3
    
    common --> ext2
    
    style api fill:#e1f5ff
    style executor fill:#fff4e1
    style storage fill:#f3e5f5
    style core fill:#e8f5e9
```

**Key Principles:**

1. **No Circular Dependencies** - All dependencies flow downward through layers
2. **Core as Foundation** - The `core` module has minimal dependencies and is imported by all other modules
3. **Storage Independence** - Storage layer does not depend on query execution
4. **API Isolation** - Public API is the only layer that depends on execution and storage together

**Sources:** [src/lib.rs:66-151]()

---

## Key Type Re-exports

The root `lib.rs` file re-exports key types from each module to provide a convenient public API. This allows users to write `use oxibase::{Database, Value, Row}` instead of `use oxibase::api::Database; use oxibase::core::{Value, Row}`.

### Public API Exports

| Category | Re-exported Types | Source Module |
|----------|------------------|---------------|
| **Core Types** | `Value`, `Row`, `Schema`, `DataType`, `Error`, `Result` | `core` |
| **Database API** | `Database`, `Transaction`, `Rows`, `Statement`, `ResultRow` | `api` |
| **Storage Traits** | `Engine`, `Transaction`, `Table`, `Scanner`, `Index` | `storage` |
| **MVCC Types** | `MVCCEngine`, `MvccTransaction`, `VersionStore`, `TransactionRegistry` | `storage::mvcc` |
| **Index Types** | `BTreeIndex`, `HashIndex`, `BitmapIndex` | `storage::index` |
| **Expressions** | `Expression`, `ComparisonExpr`, `AndExpr`, `OrExpr` | `storage::expressions` |
| **Functions** | `FunctionRegistry`, `ScalarFunction`, `AggregateFunction`, `WindowFunction` | `functions` |
| **Executor** | `Executor`, `QueryPlanner`, `QueryCache`, `ExecutionContext` | `executor` |
| **Persistence** | `WALManager`, `PersistenceManager`, `WALEntry`, `SyncMode`, `Config` | `storage::persistence` |
| **Utilities** | `BufferPool`, `Int64Map`, `SemVer` | `common` |

**Sources:** [src/lib.rs:75-151]()

---

## Storage Module Deep Dive

The `storage/` module is the most complex, containing multiple sub-modules for different aspects of the storage engine.

```mermaid
graph TB
    subgraph "storage/"
        subgraph "mvcc/"
            engine["engine.rs
            MVCCEngine"]
            
            transaction["transaction.rs
            MvccTransaction"]
            
            version_store["version_store.rs
            VersionStore
            Version chains"]
            
            registry["registry.rs
            TransactionRegistry"]
            
            table["table.rs
            MVCCTable"]
        end
        
        subgraph "index/"
            btree["btree.rs
            BTreeIndex"]
            
            hash["hash.rs
            HashIndex"]
            
            bitmap["bitmap.rs
            BitmapIndex"]
            
            index_trait["mod.rs
            Index trait"]
        end
        
        subgraph "persistence/"
            wal["wal.rs
            WALManager"]
            
            persistence_mgr["persistence_manager.rs
            PersistenceManager"]
            
            snapshot["snapshot.rs
            Snapshot logic"]
            
            recovery["recovery.rs
            2-phase WAL replay"]
        end
        
        subgraph "expressions/"
            expr_trait["mod.rs
            Expression trait"]
            
            comparison["comparison.rs
            ComparisonExpr"]
            
            logical["logical.rs
            AndExpr/OrExpr"]
            
            vm["vm.rs
            ExprVM bytecode"]
        end
        
        config["config.rs
        Config/PersistenceConfig"]
        
        traits["traits.rs
        Engine/Transaction/Table/Scanner"]
    end
    
    engine --> version_store
    engine --> registry
    engine --> wal
    engine --> persistence_mgr
    
    transaction --> table
    table --> version_store
    
    version_store --> btree
    version_store --> hash
    version_store --> bitmap
    
    persistence_mgr --> wal
    persistence_mgr --> snapshot
    recovery --> wal
    
    vm --> expr_trait
    comparison --> expr_trait
    logical --> expr_trait
```

**File Organization:**

| File Path | Primary Types | Purpose |
|-----------|---------------|---------|
| `storage/mod.rs` | Module re-exports | Public interface of storage module |
| `storage/traits.rs` | `Engine`, `Transaction`, `Table`, `Scanner`, `Index` | Core storage abstractions |
| `storage/config.rs` | `Config`, `PersistenceConfig`, `SyncMode` | Configuration types |
| `storage/mvcc/engine.rs` | `MVCCEngine` | Transaction coordinator |
| `storage/mvcc/transaction.rs` | `MvccTransaction` | Transaction handle |
| `storage/mvcc/version_store.rs` | `VersionStore` | Multi-version storage |
| `storage/mvcc/registry.rs` | `TransactionRegistry` | Transaction ID management |
| `storage/mvcc/table.rs` | `MVCCTable` | Transaction-aware table API |
| `storage/index/btree.rs` | `BTreeIndex` | B-tree index implementation |
| `storage/index/hash.rs` | `HashIndex` | Hash index with `ahash` |
| `storage/index/bitmap.rs` | `BitmapIndex` | Roaring bitmap index |
| `storage/persistence/wal.rs` | `WALManager`, `WALEntry` | Write-ahead log |
| `storage/persistence/persistence_manager.rs` | `PersistenceManager` | Snapshot coordination |
| `storage/persistence/recovery.rs` | Recovery logic | 2-phase WAL replay |
| `storage/expressions/mod.rs` | `Expression` trait | Expression evaluation interface |
| `storage/expressions/vm.rs` | `ExprVM` | Zero-recursion bytecode VM |

**Sources:** [README.md:97-99](), [src/lib.rs:88-125]()

---

## Functions Module Organization

The `functions/` module is organized by function category with a central registry.

```mermaid
graph TB
    subgraph "functions/"
        registry["registry.rs
        FunctionRegistry
        Singleton registry"]
        
        traits["traits.rs
        ScalarFunction
        AggregateFunction
        WindowFunction"]
        
        subgraph "scalar/"
            string["string.rs
            UPPER/LOWER/CONCAT/etc
            23 functions"]
            
            math["math.rs
            ABS/ROUND/SQRT/etc
            28 functions"]
            
            datetime["datetime.rs
            NOW/EXTRACT/DATE_ADD/etc
            24 functions"]
            
            json["json.rs
            JSON_EXTRACT/JSON_TYPE/etc
            7 functions"]
            
            other["other.rs
            COALESCE/NULLIF/CAST/etc
            9 functions"]
        end
        
        subgraph "aggregate/"
            basic_agg["basic.rs
            COUNT/SUM/AVG/MIN/MAX"]
            
            stats["statistics.rs
            STDDEV/VARIANCE"]
            
            string_agg["string_agg.rs
            STRING_AGG"]
            
            array_agg["array_agg.rs
            ARRAY_AGG"]
        end
        
        subgraph "window/"
            ranking["ranking.rs
            ROW_NUMBER/RANK/DENSE_RANK/NTILE"]
            
            offset["offset.rs
            LAG/LEAD/FIRST_VALUE/LAST_VALUE"]
            
            distribution["distribution.rs
            PERCENT_RANK/CUME_DIST"]
        end
    end
    
    registry --> traits
    
    string --> traits
    math --> traits
    datetime --> traits
    json --> traits
    other --> traits
    
    basic_agg --> traits
    stats --> traits
    string_agg --> traits
    array_agg --> traits
    
    ranking --> traits
    offset --> traits
    distribution --> traits
```

**Function Categories:**

| Category | Count | Examples |
|----------|-------|----------|
| **String** | 23 | `UPPER`, `LOWER`, `CONCAT`, `SUBSTRING`, `LENGTH`, `TRIM`, `REPLACE` |
| **Math** | 28 | `ABS`, `ROUND`, `CEIL`, `FLOOR`, `SQRT`, `POWER`, `SIN`, `COS`, `RAND` |
| **Date/Time** | 24 | `NOW`, `EXTRACT`, `DATE_TRUNC`, `DATE_ADD`, `DATE_SUB`, `YEAR`, `MONTH` |
| **JSON** | 7 | `JSON_EXTRACT`, `JSON_TYPE`, `JSON_VALID`, `JSON_KEYS` |
| **Other** | 9 | `COALESCE`, `NULLIF`, `CAST`, `IF`, `GREATEST`, `LEAST` |
| **Aggregate** | 18 | `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, `STDDEV`, `STRING_AGG` |
| **Window** | 11 | `ROW_NUMBER`, `RANK`, `DENSE_RANK`, `LAG`, `LEAD`, `NTILE` |

**Total:** 101+ functions

**Sources:** [README.md:324-343](), [src/lib.rs:127-139]()

---

## Dependency Management

### External Dependencies

OxiBase uses carefully selected external crates for specific functionality.

**Core Dependencies:**

| Crate | Purpose | Used By |
|-------|---------|---------|
| `thiserror` | Error derive macros | `core::Error` |
| `anyhow` | Error context | Throughout codebase |
| `serde` / `serde_json` | Serialization | `core::Value`, `storage::persistence` |
| `chrono` | Date/time handling | `core::Value::Timestamp`, date functions |

**Concurrency:**

| Crate | Purpose | Used By |
|-------|---------|---------|
| `parking_lot` | Fast locks (RwLock, Mutex) | `storage::mvcc`, `storage::persistence` |
| `dashmap` | Concurrent HashMap | `executor::QueryCache` |
| `crossbeam` | Lock-free data structures | Channel communication |
| `rayon` | Data parallelism | `executor` (parallel scans) |

**Storage/Indexing:**

| Crate | Purpose | Used By |
|-------|---------|---------|
| `ahash` | Fast hashing (~30 GB/s) | `storage::index::HashIndex` |
| `roaring` | Compressed bitmaps | `storage::index::BitmapIndex` |
| `radsort` | O(n) integer sorting | Optimizer statistics |

**Persistence:**

| Crate | Purpose | Used By |
|-------|---------|---------|
| `crc32fast` | Checksums | `storage::persistence::WALManager` |
| `lz4_flex` | Compression | `storage::persistence` |

**CLI (Optional):**

| Crate | Purpose | Enabled By |
|-------|---------|------------|
| `clap` | Argument parsing | `cli` feature |
| `rustyline` | Interactive REPL | `cli` feature |
| `comfy-table` | Table formatting | `cli` feature |

**Sources:** [Cargo.toml:30-79]()

---

## Feature Flags

OxiBase uses Cargo features to enable optional functionality.

```mermaid
graph LR
    default["default
    Enables: cli"]
    
    cli["cli
    Requires: clap, rustyline,
    comfy-table, dirs"]
    
    pg_server["pg-server
    Requires: tokio
    Status: Not yet implemented"]
    
    simd["simd
    Future: SIMD optimizations
    Status: Placeholder"]
    
    default --> cli
```

**Feature Configuration:**

| Feature | Status | Purpose | Dependencies Added |
|---------|--------|---------|-------------------|
| `default` | Active | Includes CLI by default | (delegates to `cli`) |
| `cli` | Optional | Command-line interface | `clap`, `rustyline`, `comfy-table`, `dirs` |
| `pg-server` | Future | PostgreSQL wire protocol server | `tokio` |
| `simd` | Future | SIMD optimizations for query execution | None yet |

**Usage:**
```toml
# Include only the library (no CLI)
[dependencies]
oxibase = { version = "0.1", default-features = false }

# Include CLI (default)
[dependencies]
oxibase = "0.1"

# Future: Include pg-server
[dependencies]
oxibase = { version = "0.1", features = ["pg-server"] }
```

**Sources:** [Cargo.toml:94-98](), [Cargo.toml:19-28]()

---

## Build Profiles

OxiBase defines custom build profiles optimized for different use cases.

| Profile | LTO | Codegen Units | Debug Symbols | Use Case |
|---------|-----|---------------|---------------|----------|
| `release` | Full | 1 | Yes | Production deployment with profiling |
| `bench` | Full | 1 | No | Benchmarking |
| `ci` | Thin | 16 | No | Fast CI/CD builds with minimal disk usage |

**Release Profile Configuration:**
```toml
[profile.release]
lto = true              # Full link-time optimization
codegen-units = 1       # Single codegen unit for max optimization
panic = "abort"         # Smaller binary, faster panics
opt-level = 3           # Maximum optimization
debug = true            # Keep debug symbols for profiling
```

**CI Profile Configuration:**
```toml
[profile.ci]
inherits = "release"
lto = "thin"            # Faster than full LTO
codegen-units = 16      # Parallel compilation
debug = false           # No debug symbols for tests
```

**Sources:** [Cargo.toml:100-117]()

---

## Code Organization Best Practices

The OxiBase codebase follows these organizational principles:

### 1. Strict Layering
- Lower layers never import from higher layers
- Each module has a clear architectural role
- Circular dependencies are prevented by design

### 2. Trait-Based Abstractions
- Core abstractions defined in `storage/traits.rs`: `Engine`, `Transaction`, `Table`, `Scanner`, `Index`
- Multiple implementations possible (currently only MVCC, but extensible)
- Functions use traits (`ScalarFunction`, `AggregateFunction`, `WindowFunction`)

### 3. Module Privacy
- Each module exposes a clean public API via `mod.rs`
- Internal implementation details kept private
- Re-exports in `lib.rs` provide convenient top-level access

### 4. Type Safety
- Strong typing throughout (no `Any` types)
- `Value` enum for runtime type flexibility
- Compile-time guarantees where possible

### 5. Zero-Cost Abstractions
- `BufferPool` for allocation reuse
- `Arc<T>` for cheap cloning of immutable data
- Arena-based row storage for zero-copy scanning
- Bytecode VM (`ExprVM`) instead of recursive evaluation

**Sources:** [src/lib.rs:1-152]()

---

## Module Size Metrics

Approximate lines of code per module (excluding tests and comments):

```mermaid
graph LR
    storage["storage/
    ~8000 LOC"]
    
    executor["executor/
    ~3500 LOC"]
    
    functions["functions/
    ~3000 LOC"]
    
    parser["parser/
    ~2000 LOC"]
    
    api["api/
    ~800 LOC"]
    
    core["core/
    ~600 LOC"]
    
    common["common/
    ~400 LOC"]
    
    style storage fill:#e8f5e9
    style executor fill:#fff4e1
    style functions fill:#f3e5f5
```

**Largest Modules:**
1. `storage/` - Most complex module with MVCC, indexes, persistence (~8000 LOC)
2. `executor/` - Query planning and execution (~3500 LOC)
3. `functions/` - 101+ function implementations (~3000 LOC)
4. `parser/` - SQL lexer and parser (~2000 LOC)

The storage module is intentionally the largest because it contains the most complex subsystems: transaction management, version storage, multiple index implementations, WAL, snapshots, and recovery logic.

**Sources:** [README.md:85-100]()