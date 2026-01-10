---
layout: default
title: MVCC Implementation
parent: Architecture
nav_order: 4
---

# MVCC Implementation

This document provides a detailed explanation of Oxibase's Multi-Version Concurrency Control (MVCC) implementation, which enables transaction isolation without locking.

## MVCC Overview

Multi-Version Concurrency Control (MVCC) is a concurrency control method used by Oxibase to provide transaction isolation. The key principles are:

1. Maintain full version chains for each row with unlimited history
2. Track deletion status with transaction IDs for proper visibility
3. Each transaction has a consistent view based on visibility rules
4. Reads never block writes, and writes never block reads
5. Implement optimistic concurrency control for conflict detection

### Core Components Overview

```mermaid
graph TB
    subgraph "Transaction Context"
        MVCCTable["MVCCTable<br/>txn_id: i64<br/>cached_schema: Schema"]
        TxnVerStore["Arc&lt;RwLock&lt;TransactionVersionStore&gt;&gt;<br/>local_versions: Int64Map<br/>write_set: FxHashMap"]
    end
    
    subgraph "Global State"
        VersionStore["VersionStore<br/>versions: ConcurrentInt64Map<br/>schema: RwLock&lt;Schema&gt;<br/>indexes: RwLock&lt;FxHashMap&gt;"]
        Arena["RowArena<br/>arena_rows: Vec&lt;RowMetadata&gt;<br/>arena_data: Vec&lt;Value&gt;"]
    end
    
    subgraph "Version Chain"
        V1["VersionChainEntry<br/>version: RowVersion<br/>prev: Option&lt;Arc&lt;...&gt;&gt;<br/>arena_idx: Option&lt;usize&gt;"]
        V2["VersionChainEntry<br/>older version"]
        V3["VersionChainEntry<br/>oldest version"]
        
        V1 -->|"Arc::clone()"| V2
        V2 -->|"Arc::clone()"| V3
    end
    
    MVCCTable -->|"reads from"| VersionStore
    MVCCTable -->|"writes to"| TxnVerStore
    MVCCTable -->|"shares"| TxnVerStore
    
    VersionStore -->|"stores"| V1
    VersionStore -->|"uses"| Arena
    
    V1 -.->|"arena_idx"| Arena
```

## Design Philosophy

Oxibase implements a **true multi-version MVCC** design:

- **Full Version Chains**: Unlimited version history per row linked via `prev` pointers
- **In-Memory Chains**: Version chains built from WAL replay during recovery
- **Immutable Versions**: New versions always created, never modified in place
- **Efficient Persistence**: Only latest version persisted to disk snapshots
- **Automatic Cleanup**: Old versions garbage collected when no longer needed

## Core Components

### Transaction Registry

- Manages transaction lifecycle and state tracking
- Assigns unique transaction IDs using atomic counters
- Tracks active and committed transactions with monotonic sequences
- Supports per-transaction isolation levels without race conditions
- Implements visibility rules for both READ COMMITTED and SNAPSHOT isolation

### Version Store

- Maintains full version chains for each row
- Tracks both creation (`txn_id`) and deletion (`deleted_at_txn_id`) transaction IDs
- Implements efficient concurrent access using concurrent data structures
- Manages B-tree, Hash, and Bitmap indexes
- Provides visibility-aware traversal of version chains

### Row Version Structure

```rust
struct RowVersion {
    txn_id: i64,           // Transaction that created this version
    deleted_at_txn_id: i64, // Transaction that deleted this version (0 if not deleted)
    data: Row,             // Complete row data
    row_id: i64,           // Row identifier
    create_time: i64,      // Timestamp when created
    prev: Option<Box<RowVersion>>, // Previous version in the chain
}
```

The `prev` pointer creates a backward-linked chain from newest to oldest version.

### Version Chains with Arc Pointers

```mermaid
graph LR
    subgraph "versions: ConcurrentInt64Map"
        Head["row_id: 100<br/>↓<br/>VersionChainEntry"]
    end
    
    Head -->|"prev: Arc"| V2["VersionChainEntry<br/>txn_id: 20<br/>deleted_at: 0"]
    V2 -->|"prev: Arc"| V3["VersionChainEntry<br/>txn_id: 10<br/>deleted_at: 0"]
    V3 -.->|"prev: None"| End["None"]
    
    Head2["Different transaction<br/>clones Arc"] -.->|"Arc::clone()"| V2
```

### VersionStore: Global Committed State

```mermaid
graph TB
    subgraph "VersionStore"
        Versions["versions<br/>ConcurrentInt64Map&lt;i64, VersionChainEntry&gt;<br/>(row_id → chain head)"]
        Schema["schema<br/>RwLock&lt;Schema&gt;"]
        Indexes["indexes<br/>RwLock&lt;FxHashMap&lt;String, Arc&lt;dyn Index&gt;&gt;&gt;"]
        Arena["arena<br/>RowArena"]
        RowArenaIndex["row_arena_index<br/>RwLock&lt;Int64Map&lt;usize&gt;&gt;"]
        UncommittedWrites["uncommitted_writes<br/>ConcurrentInt64Map&lt;i64, i64&gt;<br/>(row_id → txn_id)"]
    end
    
    Versions --> Chain["Version chains"]
    Arena --> ArenaRows["arena_rows: Vec&lt;RowMetadata&gt;"]
    Arena --> ArenaData["arena_data: Vec&lt;Value&gt;"]
    
    RowArenaIndex -.->|"maps row_id"| Arena
```

### Arena-Based Storage for Zero-Copy Scans

```mermaid
graph TB
    subgraph "RowArena Structure"
        direction LR
        
        subgraph "arena_rows: Vec&lt;RowMetadata&gt;"
            M0["[0] RowMetadata<br/>row_id: 1<br/>start: 0, end: 3<br/>txn_id: 10"]
            M1["[1] RowMetadata<br/>row_id: 2<br/>start: 3, end: 6<br/>deleted_at: 15"]
            M2["[2] RowMetadata<br/>row_id: 1<br/>start: 6, end: 9<br/>txn_id: 20"]
        end
        
        subgraph "arena_data: Vec&lt;Value&gt;"
            D0["[0] Integer(100)"]
            D1["[1] Text('Alice')"]
            D2["[2] Boolean(true)"]
            D3["[3] Integer(200)"]
            D4["[4] Text('Bob')"]
            D5["[5] Boolean(false)"]
            D6["[6] Integer(150)"]
            D7["[7] Text('Alice')"]
            D8["[8] Boolean(true)"]
        end
    end
    
    M0 -.->|"[start..end]"| D0
    M0 -.-> D1
    M0 -.-> D2
    
    M1 -.-> D3
    M1 -.-> D4
    M1 -.-> D5
    
    M2 -.-> D6
    M2 -.-> D7
    M2 -.-> D8
```

### TransactionVersionStore: Uncommitted Changes

```mermaid
graph TB
    subgraph "TransactionVersionStore"
        LocalVers["local_versions<br/>Int64Map&lt;i64, RowVersion&gt;<br/>(row_id → pending version)"]
        WriteSet["write_set<br/>FxHashMap&lt;i64, WriteSetEntry&gt;<br/>(row_id → read version)"]
        OldRows["old_rows<br/>FxHashMap&lt;i64, Row&gt;<br/>(row_id → pre-update row)"]
    end
    
    subgraph "WriteSetEntry"
        ReadVer["read_version: Option&lt;RowVersion&gt;"]
        ReadSeq["read_version_seq: i64"]
    end
    
    WriteSet --> ReadVer
    WriteSet --> ReadSeq
```

### Conflict Detection Flow

```mermaid
sequenceDiagram
    participant Txn as Transaction
    participant TVS as TransactionVersionStore
    participant VS as VersionStore
    
    Txn->>VS: get_visible_version(row_id=1)
    VS-->>Txn: RowVersion(txn_id=10, seq=50)
    
    Txn->>TVS: put(row_id=1, new_data)
    TVS->>TVS: write_set[1] = {read_version: v10, read_seq: 50}
    TVS->>TVS: local_versions[1] = new_version
    
    Note over Txn: Time passes...
    
    Txn->>TVS: commit()
    TVS->>VS: get_current_sequence()
    VS-->>TVS: current_seq=75
    
    alt read_seq (50) < current_seq (75)
        TVS->>VS: get_visible_version(row_id=1, current_txn)
        VS-->>TVS: RowVersion(txn_id=20, seq=60)
        
        TVS->>TVS: Conflict! (read v10 but v20 was committed)
        TVS-->>Txn: Error: write-write conflict
    else read_seq == current_seq
        TVS->>VS: add_versions_batch(local_versions)
        TVS-->>Txn: Commit successful
    end
```

## Transaction IDs and Timestamps

Oxibase uses monotonic sequences instead of wall-clock timestamps to avoid platform-specific timing issues:

- **Transaction ID**: Unique identifier assigned atomically
- **Begin Sequence**: Monotonic sequence when transaction starts
- **Commit Sequence**: Monotonic sequence when transaction commits
- **Write Sequences**: Track when rows were last modified for conflict detection

This approach solves Windows' 15.6ms timer resolution issue and ensures consistent ordering.

## Isolation Levels

### READ COMMITTED (Default)

- Transactions see committed changes immediately
- No global locks for commits - high concurrency
- Each statement sees the latest committed data
- Suitable for most OLTP workloads

Implementation:
```rust
// In READ COMMITTED, only check if transaction is committed
fn is_directly_visible(&self, version_txn_id: i64) -> bool {
    self.committed_transactions.contains(version_txn_id)
}
```

### SNAPSHOT Isolation

- Transactions see a consistent snapshot from when they started
- Write-write conflict detection prevents lost updates
- Lock-free commits with optimistic concurrency control
- High throughput with strong consistency guarantees

Implementation:
```rust
// In SNAPSHOT, check if version was committed before viewer began
fn is_visible(&self, version_txn_id: i64, viewer_txn_id: i64) -> bool {
    if let Some(commit_ts) = self.committed_transactions.get(version_txn_id) {
        let viewer_begin_ts = self.get_transaction_begin_seq(viewer_txn_id);
        commit_ts <= viewer_begin_ts
    } else {
        false
    }
}
```

## Visibility Rules

### Version Chain Traversal

Visibility is determined by traversing the version chain:
```rust
// Traverse the version chain from newest to oldest
let mut current = version_ptr;
while let Some(version) = current {
    if registry.is_visible(version.txn_id, txn_id) {
        // Check deletion visibility
        if version.deleted_at_txn_id != 0
            && registry.is_visible(version.deleted_at_txn_id, txn_id) {
            return None; // Deleted
        }
        return Some(version); // Found visible version
    }
    current = version.prev.as_ref();
}
```

### Row Visibility

A row is visible to a transaction if:
1. A version exists in the chain that was created by a visible transaction, AND
2. That version was NOT deleted, OR the deletion is not visible

### Transaction-Specific Isolation

Each transaction maintains its own isolation level:
```rust
// Set isolation level for specific transaction
registry.set_transaction_isolation_level(txn_id, level);

// Get isolation level for visibility checks
let isolation_level = registry.get_isolation_level(txn_id);
```

## Concurrency Control

### SNAPSHOT Isolation Conflicts

Write-write conflict detection during commit:

```rust
// Check for conflicts before commit
if version_store.check_write_conflict(&written_rows, begin_seq) {
    return Err(OxibaseError::Transaction(
        "transaction aborted due to write-write conflict".to_string()
    ));
}

// Set write sequences after successful commit
version_store.set_write_sequences(&written_rows, commit_seq);
```

### Lock-Free Commit Process

SNAPSHOT commits use optimistic concurrency control:
1. Check for write-write conflicts
2. Generate commit sequence
3. Apply changes to version stores (creating new versions)
4. Set write sequences atomically
5. Mark transaction as committed

No global mutex needed - conflicts detected through version checks.

### Read Path: Query Execution

```mermaid

flowchart TD
    Query["SELECT * FROM users<br/>WHERE email = 'alice@example.com'"]
    
    Query --> TryPK("try_pk_lookup<br/>Is WHERE on PK?")
    
    TryPK -->|Yes| PKLookup["get_visible_version(row_id)<br/>O(1) hash lookup"]
    TryPK -->|No| TryIndex("try_index_lookup<br/>Is WHERE on indexed column?")
    
    TryIndex -->|Yes| IndexPath["Index lookup"]
    TryIndex -->|No| FullScan["Full table scan"]
    
    IndexPath --> HashIndex("Index type?")
    HashIndex -->|Hash| HashFind["hash_to_rows.get(hash)<br/>O(1)"]
    HashIndex -->|BTree| BTreeFind["sorted_values.range()<br/>O(log n + k)"]
    HashIndex -->|Bitmap| BitmapFind["bitmap.and_count()<br/>O(n/64)"]
    
    HashFind --> RowIDs["Vec&lt;i64&gt; row_ids"]
    BTreeFind --> RowIDs
    BitmapFind --> RowIDs
    
    RowIDs --> BatchGet["get_visible_versions_batch(row_ids)"]
    
    PKLookup --> CheckVis["Check visibility"]
    BatchGet --> CheckVis
    FullScan --> ArenaIter["get_all_visible_rows_arena()"]
    
    ArenaIter --> CheckVis
    
    CheckVis --> Normalize["normalize_row_to_schema()<br/>(handle ALTER TABLE)"]
    Normalize --> Results["Vec&lt;(i64, Row)&gt;"]

```

### Write Path: Insert, Update, Delete

```mermaid
flowchart TD
    Insert["MVCCTable::insert(row)"]
    Update["MVCCTable::update(filter, updates)"]
    Delete["MVCCTable::delete(filter)"]
    
    Insert --> ExtractPK["extract_row_pk(row)<br/>Auto-increment if needed"]
    ExtractPK --> CheckUnique["check_unique_constraints(row)"]
    CheckUnique --> AddLocal["txn_versions.put(row_id, version)"]
    
    Update --> ScanRows["scan(filter)<br/>Get matching row_ids"]
    ScanRows --> BatchGetUpdate["get_visible_versions_for_update()<br/>Read + track in write_set"]
    BatchGetUpdate --> ApplyUpdates["Apply SET clauses"]
    ApplyUpdates --> BatchPut["Batch: txn_versions.put(...)"]
    
    Delete --> ScanDel["scan(filter)"]
    ScanDel --> MarkDeleted["txn_versions.delete(row_id)"]
    
    AddLocal --> LocalStore["TransactionVersionStore<br/>local_versions"]
    BatchPut --> LocalStore
    MarkDeleted --> LocalStore
    
    LocalStore -.->|"Later: commit()"| FlushGlobal["Flush to VersionStore"]
    FlushGlobal --> UpdateIndexes["Update all indexes"]
    UpdateIndexes --> ZoneMap["Mark zone_maps_stale"]
```

## Version Chain Management

### Updates

When a row is updated:
- A new version is created with the updating transaction's ID
- The new version's `prev` pointer links to the current version
- The version chain grows backward in time
- All historical versions remain accessible

### Deletions

When a row is deleted:
- A new version is created with `DeletedAtTxnID` set
- The deletion version links to the previous version
- Data is preserved in the deletion version
- The row appears deleted to transactions that see this version

### Version Chain Example

```
[Newest] -> Version 3 (TxnID=300, DeletedAt=400)
              |
              v
            Version 2 (TxnID=200)
              |
              v
            Version 1 (TxnID=100) -> [Oldest]
```

### Row Normalization for Schema Evolution

```mermaid
graph LR
    OldRow["Old Row (2 columns)<br/>[100, 'Alice']"]
    NewSchema["Current Schema (3 columns)<br/>id, name, age"]
    
    OldRow --> Normalize["normalize_row_to_schema()"]
    NewSchema --> Normalize
    
    Normalize --> Check["row.len() vs schema.len()"]
    
    Check -->|"row.len() < schema"| AddDefaults["Append default values<br/>or NULLs"]
    Check -->|"row.len() > schema"| Truncate["Truncate extra columns"]
    Check -->|"Equal"| NoOp["Return as-is"]
    
    AddDefaults --> NewRow["[100, 'Alice', NULL]"]
    Truncate --> NewRow2["Truncated row"]
    NoOp --> NewRow3["Original row"]
```

## Performance Optimizations

### Lock-Free Data Structures

- `SegmentInt64Map`: High-performance concurrent maps
- Atomic operations for counters and flags
- Minimal mutex usage in hot paths

### Object Pooling

- Transaction objects
- Table objects  
- Version maps
- Reduces GC pressure in high-throughput scenarios

### Optimized Visibility Checks

- Fast path for own-transaction visibility
- Direct visibility check for READ COMMITTED
- Batch processing for bulk operations

### Memory Management

- Version chains built on-demand from WAL
- Automatic cleanup of old versions no longer needed
- Periodic cleanup of deleted rows
- Cold data eviction to disk

## Garbage Collection

### Version Chain Cleanup

Old versions in chains are cleaned up when:
1. No active transaction can see them
2. A newer version is visible to all active transactions

```rust
// Find oldest version still needed
let mut current = version;
let mut last_needed = None;
while let Some(v) = current {
    for txn_id in &active_transactions {
        if registry.is_visible(v.txn_id, *txn_id) {
            last_needed = Some(v);
            break;
        }
    }
    current = v.prev.as_ref();
}
// Disconnect older versions (Rust's ownership handles cleanup)
if let Some(last) = last_needed {
    last.prev = None; // Drop older versions
}
```

### Deleted Row Cleanup

Deleted rows are removed based on:
1. Retention period (age-based)
2. Transaction visibility (no active transaction can see them)
3. Safety checks to prevent removing visible data

## Key Implementation Files

- `src/storage/mvcc/engine.rs` - MVCC engine coordinating all components
- `src/storage/mvcc/table.rs` - Table operations with MVCC support
- `src/storage/mvcc/transaction.rs` - Transaction implementation and conflict detection
- `src/storage/mvcc/version_store.rs` - Row version storage and management

## Best Practices

1. **Choose Appropriate Isolation**: Use READ COMMITTED unless you need snapshot consistency
2. **Keep Transactions Short**: Long transactions delay garbage collection
3. **Handle Conflicts**: Implement retry logic for SNAPSHOT conflicts
4. **Monitor Deleted Rows**: Ensure garbage collection keeps up with deletions
5. **Batch Operations**: Group related changes in single transactions

## Future Improvements

Potential enhancements to the current design:
1. **Time Travel Queries**: Query data as of specific timestamps
2. **Additional Isolation Levels**: REPEATABLE READ, SERIALIZABLE
3. **Read-Set Tracking**: Detect read-write conflicts for SERIALIZABLE
4. **Savepoint Support**: Transaction savepoints for partial rollbacks
5. **Version Compression**: Delta encoding for version chains
