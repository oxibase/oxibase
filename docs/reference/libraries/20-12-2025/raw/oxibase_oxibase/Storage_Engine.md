# Page: Storage Engine

# Storage Engine

<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

- [src/storage/mvcc/btree_index.rs](src/storage/mvcc/btree_index.rs)
- [src/storage/mvcc/engine.rs](src/storage/mvcc/engine.rs)
- [src/storage/mvcc/hash_index.rs](src/storage/mvcc/hash_index.rs)
- [src/storage/mvcc/table.rs](src/storage/mvcc/table.rs)
- [src/storage/mvcc/version_store.rs](src/storage/mvcc/version_store.rs)

</details>



The Storage Engine is the foundation of Oxibase's data persistence and transaction management. It implements Multi-Version Concurrency Control (MVCC) for snapshot isolation, manages row versioning and visibility, coordinates multiple index types, and handles Write-Ahead Logging (WAL) for crash recovery.

For information about how SQL queries are executed on top of the storage layer, see [Query Execution System](#3). For details on the expression evaluation that filters rows, see [Expression Evaluation](#3.2). For specifics on MVCC transaction semantics and isolation levels, see [MVCC Architecture](#4.1).

## Architecture Overview

The storage engine is organized into three primary layers: the Engine layer (orchestration), the Table layer (transaction-aware access), and the Storage layer (version management and indexing).

```mermaid
graph TB
    subgraph "Engine Layer - Orchestration"
        MVCCEngine["MVCCEngine<br/>src/storage/mvcc/engine.rs<br/>• Table lifecycle<br/>• Transaction registry<br/>• Persistence manager<br/>• Schema catalog"]
        
        TransactionRegistry["TransactionRegistry<br/>• Transaction ID allocation<br/>• Commit sequence numbers<br/>• Visibility checking"]
        
        PersistenceManager["PersistenceManager<br/>• WAL management<br/>• Snapshot creation<br/>• Recovery coordination"]
    end
    
    subgraph "Table Layer - Transaction-aware Access"
        MVCCTable["MVCCTable<br/>src/storage/mvcc/table.rs<br/>• Transaction isolation<br/>• Row operations<br/>• Index selection<br/>• Commit/rollback"]
        
        TransactionVersionStore["TransactionVersionStore<br/>• Uncommitted changes<br/>• Write-set tracking<br/>• Conflict detection"]
    end
    
    subgraph "Storage Layer - Version Management"
        VersionStore["VersionStore<br/>src/storage/mvcc/version_store.rs<br/>• Version chains<br/>• Visibility filtering<br/>• Arena-based scanning<br/>• Zone maps"]
        
        RowArena["RowArena<br/>• Contiguous memory storage<br/>• 50x+ scan speedup<br/>• Zero-copy reads"]
        
        VersionChainEntry["VersionChainEntry<br/>• Current version<br/>• Arc prev pointer<br/>• Arena index"]
    end
    
    subgraph "Index Layer"
        BTreeIndex["BTreeIndex<br/>src/storage/mvcc/btree_index.rs<br/>• Range queries O(log n + k)<br/>• Cached min/max"]
        
        HashIndex["HashIndex<br/>src/storage/mvcc/hash_index.rs<br/>• Equality O(1)<br/>• ahash for TEXT/JSON"]
        
        BitmapIndex["BitmapIndex<br/>• Boolean columns<br/>• Bitwise AND/OR/NOT"]
        
        MultiColumnIndex["MultiColumnIndex<br/>• Composite key lookups<br/>• Prefix matching"]
    end
    
    MVCCEngine -->|creates| MVCCTable
    MVCCEngine -->|manages| TransactionRegistry
    MVCCEngine -->|uses| PersistenceManager
    
    MVCCTable -->|reads/writes| TransactionVersionStore
    MVCCTable -->|delegates to| VersionStore
    
    TransactionVersionStore -->|commits to| VersionStore
    
    VersionStore -->|stores versions in| RowArena
    VersionStore -->|maintains| VersionChainEntry
    VersionStore -->|updates| BTreeIndex
    VersionStore -->|updates| HashIndex
    VersionStore -->|updates| BitmapIndex
    VersionStore -->|updates| MultiColumnIndex
    
    TransactionRegistry -.checks visibility.-> VersionStore
```

**Sources:** [src/storage/mvcc/engine.rs:254-279](), [src/storage/mvcc/table.rs:36-45](), [src/storage/mvcc/version_store.rs:157-193]()

## Core Components and Responsibilities

The storage engine consists of four main classes that handle different aspects of data storage and retrieval:

| Component | File | Primary Responsibilities |
|-----------|------|--------------------------|
| `MVCCEngine` | [engine.rs:254-279]() | Database lifecycle, table creation/deletion, transaction coordination, schema management, persistence orchestration |
| `MVCCTable` | [table.rs:36-45]() | Per-transaction table view, row insert/update/delete, index lookup optimization, commit/rollback execution |
| `VersionStore` | [version_store.rs:157-193]() | Version chain management, visibility checking, arena-based scanning, row counting, index population |
| `TransactionVersionStore` | [version_store.rs:1354-1414]() | Uncommitted changes buffer, write-set conflict tracking, local version cache |

**Sources:** [src/storage/mvcc/engine.rs:254-316](), [src/storage/mvcc/table.rs:36-78](), [src/storage/mvcc/version_store.rs:157-234]()

## Storage Engine Data Flow

This diagram shows how a typical transaction operation flows through the storage layers, from the public API down to the physical storage structures.

```mermaid
sequenceDiagram
    participant Client
    participant Transaction as "Transaction<br/>(MvccTransaction)"
    participant Engine as "MVCCEngine"
    participant Table as "MVCCTable"
    participant TxnStore as "TransactionVersionStore"
    participant VerStore as "VersionStore"
    participant Index as "BTreeIndex/HashIndex"
    participant WAL as "WALManager"
    
    Client->>Engine: begin_transaction()
    Engine->>TransactionRegistry: allocate txn_id
    Engine->>Transaction: MvccTransaction(txn_id)
    
    Client->>Transaction: execute("INSERT INTO users VALUES (...)")
    Transaction->>Engine: get_table_for_transaction("users")
    Engine->>Table: MVCCTable::new_with_shared_store()
    
    Transaction->>Table: insert(row)
    Table->>TxnStore: add_local_version(row_version)
    Note over TxnStore: Buffered in memory<br/>Not yet visible
    
    Client->>Transaction: commit()
    Transaction->>Table: commit()
    
    Table->>Index: update indexes (old_value, new_value)
    Table->>TxnStore: commit()
    TxnStore->>VerStore: add_versions_batch()
    
    VerStore->>RowArena: insert_row()
    Note over VerStore: Versions now visible<br/>to new transactions
    
    VerStore->>WAL: record_operation()
    WAL-->>VerStore: LSN
    
    Table-->>Transaction: Ok
    Transaction->>TransactionRegistry: mark_committed(commit_seq)
    Transaction-->>Client: Ok
```

**Sources:** [src/storage/mvcc/engine.rs:1185-1227](), [src/storage/mvcc/table.rs:722-804](), [src/storage/mvcc/version_store.rs:292-367]()

## Version Chain Management

Oxibase implements MVCC by maintaining linked version chains for each row. Each `VersionChainEntry` contains a `RowVersion` and an `Arc` pointer to the previous version, enabling O(1) chain cloning for snapshot isolation.

```mermaid
graph LR
    subgraph "versions: ConcurrentInt64Map<VersionChainEntry>"
        row1["row_id: 1"]
        row2["row_id: 2"]
        row3["row_id: 3"]
    end
    
    subgraph "Version Chain for row_id=1"
        v3["VersionChainEntry<br/>txn_id: 30<br/>deleted_at: 30<br/>arena_idx: None"]
        v2["VersionChainEntry<br/>txn_id: 20<br/>deleted_at: 0<br/>arena_idx: Some(15)"]
        v1["VersionChainEntry<br/>txn_id: 10<br/>deleted_at: 0<br/>arena_idx: Some(5)"]
        
        v3 -->|"prev: Arc"| v2
        v2 -->|"prev: Arc"| v1
        v1 -->|"prev: None"| end1["Initial version"]
    end
    
    subgraph "RowArena - Contiguous Memory"
        arena_idx_5["idx 5: row_id=1, txn=10, data=[...]"]
        arena_idx_15["idx 15: row_id=1, txn=20, data=[...]"]
        arena_other["Other rows..."]
    end
    
    row1 --> v3
    v1 -.arena_idx: Some(5).-> arena_idx_5
    v2 -.arena_idx: Some(15).-> arena_idx_15
```

Key data structures:

- **`versions: ConcurrentInt64Map<VersionChainEntry>`** [version_store.rs:171]() - Maps `row_id` to the head of its version chain
- **`VersionChainEntry`** [version_store.rs:112-119]() - Linked list node with `version: RowVersion`, `prev: Option<Arc<...>>`, and `arena_idx: Option<usize>`
- **`RowArena`** [version_store.rs:187]() - Contiguous memory storage providing 50x+ faster full table scans via zero-copy reads

The use of `Arc` for the `prev` pointer enables cheap snapshot cloning—creating a snapshot of a table is O(1) because the chain is reference-counted rather than deep-copied.

**Sources:** [src/storage/mvcc/version_store.rs:110-119](), [src/storage/mvcc/version_store.rs:169-193](), [src/storage/mvcc/version_store.rs:292-367]()

## Transaction Version Store

The `TransactionVersionStore` buffers uncommitted changes within a transaction, maintaining write-set information for conflict detection during commit.

| Field | Type | Purpose |
|-------|------|---------|
| `local_versions` | `Int64Map<RowVersion>` | Uncommitted inserts/updates/deletes |
| `write_set` | `Int64Map<WriteSetEntry>` | Conflict detection: tracks read version and sequence number |
| `version_store` | `Arc<VersionStore>` | Reference to global committed storage |
| `txn_id` | `i64` | Transaction identifier |

**Commit process** [table.rs:722-804]():
1. Check uniqueness constraints on all indexes
2. Update indexes with (old_value, new_value) pairs
3. Flush `local_versions` to `VersionStore` in batch
4. Mark zone maps as stale (for optimizer statistics)

**Rollback process** [table.rs:806-809]():
1. Discard all entries in `local_versions`
2. Clear `write_set` tracking data

**Sources:** [src/storage/mvcc/version_store.rs:1354-1414](), [src/storage/mvcc/table.rs:722-809]()

## Index Architecture

Oxibase provides three index types, each optimized for different query patterns. The `MVCCTable` automatically selects the optimal index type based on column data types.

```mermaid
graph TB
    subgraph "Index Selection Logic"
        auto_select["MVCCTable::auto_select_index_type()<br/>table.rs:105-130"]
    end
    
    subgraph "Index Type Decision Tree"
        check_bool["Is column BOOLEAN?"]
        check_text["Is column TEXT/JSON?"]
        check_numeric["Is column INTEGER/FLOAT/TIMESTAMP?"]
        
        bitmap_idx["IndexType::Bitmap<br/>Fast AND/OR/NOT<br/>Low cardinality"]
        hash_idx["IndexType::Hash<br/>O(1) equality<br/>Avoids O(strlen)"]
        btree_idx["IndexType::BTree<br/>O(log n) range queries<br/>Cached min/max"]
    end
    
    subgraph "Index Implementations"
        BTreeIndexImpl["BTreeIndex<br/>btree_index.rs:74-113<br/>• BTreeMap: sorted values<br/>• AHashMap: O(1) equality<br/>• FxHashMap: row->value"]
        
        HashIndexImpl["HashIndex<br/>hash_index.rs:105-127<br/>• ahash: fast hashing<br/>• SmallVec: inline storage<br/>• Collision handling"]
        
        BitmapIndexImpl["BitmapIndex<br/>• Roaring bitmaps<br/>• Bitwise operations<br/>• Compressed storage"]
    end
    
    auto_select --> check_bool
    check_bool -->|Yes, single column| bitmap_idx
    check_bool -->|No| check_text
    check_text -->|Yes| hash_idx
    check_text -->|No| check_numeric
    check_numeric -->|Yes| btree_idx
    
    bitmap_idx --> BitmapIndexImpl
    hash_idx --> HashIndexImpl
    btree_idx --> BTreeIndexImpl
```

**Index selection rules** [table.rs:105-130]():
- **BOOLEAN** → `BitmapIndex` for fast bitwise AND/OR/NOT operations
- **TEXT/JSON** → `HashIndex` to avoid O(strlen) comparisons in B-tree nodes
- **INTEGER/FLOAT/TIMESTAMP** → `BTreeIndex` for range query support
- **Mixed types** → `BTreeIndex` as safe default

**Sources:** [src/storage/mvcc/table.rs:95-130](), [src/storage/mvcc/btree_index.rs:74-113](), [src/storage/mvcc/hash_index.rs:105-127]()

## BTreeIndex Internal Structure

The `BTreeIndex` maintains dual data structures for optimal performance across different query types:

| Data Structure | Type | Purpose | Complexity |
|----------------|------|---------|------------|
| `sorted_values` | `RwLock<BTreeMap<Value, RowIdSet>>` | Range queries, MIN/MAX | O(log n + k) |
| `value_to_rows` | `RwLock<AHashMap<Value, RowIdSet>>` | Equality lookups | O(1) |
| `row_to_value` | `RwLock<FxHashMap<i64, Value>>` | Removal by row_id | O(1) |
| `cached_min` | `RwLock<Option<Value>>` | MIN aggregate | O(1) |
| `cached_max` | `RwLock<Option<Value>>` | MAX aggregate | O(1) |

The dual-index strategy (BTreeMap + HashMap) trades memory (~2x for unique values) for optimal query performance: O(1) equality via hash lookup and O(log n + k) range queries via sorted iteration.

**Sources:** [src/storage/mvcc/btree_index.rs:74-113](), [src/storage/mvcc/btree_index.rs:298-397]()

## HashIndex Characteristics

The `HashIndex` is designed for high-cardinality TEXT columns where equality queries dominate:

**Advantages:**
- O(1) exact match via `ahash` (faster than SipHash)
- Avoids O(strlen) string comparisons in B-tree traversal
- `SmallVec<[i64; 4]>` reduces allocations for unique indexes

**Limitations:**
- Does **NOT** support range queries [hash_index.rs:403-414]()
- Does **NOT** support ORDER BY optimization
- Requires exact match on all indexed columns (no partial key lookups)

**Triple-lock write pattern** [hash_index.rs:236-295]():
```
add() acquires:
  1. hash_to_rows: Write
  2. row_to_hash: Write
  3. hash_to_values: Write
```

This ensures atomicity but serializes concurrent writes. Read operations (`find`) only acquire a single read lock for minimal contention.

**Sources:** [src/storage/mvcc/hash_index.rs:105-127](), [src/storage/mvcc/hash_index.rs:219-296](), [src/storage/mvcc/hash_index.rs:371-401]()

## Row Arena: Zero-Copy Scanning

The `RowArena` stores row data in contiguous memory to eliminate per-row allocation overhead during full table scans:

**Performance characteristics** [version_store.rs:749-817]():
- **50x+ speedup** for full table scans vs. per-row cloning
- Pre-acquire locks once per scan (O(1) instead of O(N))
- Direct slice access via `unsafe` bounds-checked reads
- Cache locality from contiguous layout

**Arena structure:**
```
arena_rows: Vec<RowMetadata>  // [start, end, row_id, txn_id, create_time, deleted_at]
arena_data: Vec<Value>         // Flattened row data
row_arena_index: Int64Map<usize>  // row_id -> arena_idx
```

**Usage pattern** [version_store.rs:756-816]():
1. Acquire `arena_rows` and `arena_data` read guards once
2. Iterate version chains checking visibility
3. For visible versions, read directly from arena via `arena_idx`
4. Release locks after iteration completes
5. Sort results by `row_id` (if needed)

**Sources:** [src/storage/mvcc/version_store.rs:187-192](), [src/storage/mvcc/version_store.rs:749-817](), [src/storage/mvcc/arena.rs]() (referenced but not provided)

## Persistence and Recovery

The storage engine provides crash consistency through Write-Ahead Logging (WAL) and periodic snapshots:

**WAL Manager responsibilities:**
- Sequential writes with CRC32 checksums [engine.rs:494-521]()
- Two-phase recovery: Phase 1 identifies committed transactions, Phase 2 applies their changes
- Compression (optional) for reduced I/O

**Snapshot system:**
- Binary format with magic bytes (`0x50414E53` = "SNAP") [engine.rs:46-147]()
- Atomic 3-phase writes: temp file → sync → rename
- Tracks source LSN for incremental recovery [engine.rs:365-417]()

**Recovery flow** [engine.rs:487-521]():
1. Read snapshot metadata to get checkpoint LSN
2. Load table snapshots (fastest recovery path)
3. Replay WAL entries from checkpoint LSN forward
4. Apply only entries from committed transactions (two-phase)
5. Populate indexes in single pass after replay completes

**Sources:** [src/storage/mvcc/engine.rs:324-360](), [src/storage/mvcc/engine.rs:487-521](), [src/storage/mvcc/engine.rs:54-107]()

## Schema Management

The `MVCCEngine` maintains the schema catalog and handles DDL operations:

**Schema storage:**
- `schemas: Arc<RwLock<FxHashMap<String, Schema>>>` [engine.rs:260]()
- Lowercase table names for case-insensitive lookups
- Schema changes recorded to WAL [engine.rs:1085-1096]()

**ALTER TABLE operations** [engine.rs:845-1077]():
- `AddColumn`: Normalizes existing rows, adds default values [table.rs:137-163]()
- `DropColumn`: Truncates row data [table.rs:157-160]()
- `RenameColumn`: Updates schema, preserves data
- `ModifyColumn`: Type coercion with validation
- `RenameTable`: Updates all references atomically

**Schema normalization** [table.rs:137-163]():
When reading rows, `MVCCTable` normalizes row data to match the current schema, handling columns added/dropped via ALTER TABLE by filling defaults or truncating.

**Sources:** [src/storage/mvcc/engine.rs:254-279](), [src/storage/mvcc/engine.rs:845-1077](), [src/storage/mvcc/table.rs:137-163]()