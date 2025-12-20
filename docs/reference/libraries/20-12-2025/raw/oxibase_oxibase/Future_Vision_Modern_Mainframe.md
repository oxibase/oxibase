# Page: Future Vision: Modern Mainframe

# Future Vision: Modern Mainframe

<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

- [.gitignore](.gitignore)
- [README.md](README.md)
- [ROADMAP.md](ROADMAP.md)
- [docs/_config.yml](docs/_config.yml)

</details>



## Purpose and Scope

This document describes OxiBase's planned evolution from an embedded SQL database into a distributed "Modern Mainframe" architecture. It outlines the transformation from a single-process MVCC storage engine into a self-managing, unikernel-based distributed system capable of millions of transactions per second.

For information about the current embedded architecture, see [Architecture Overview](#1.2). For current MVCC implementation details, see [MVCC Architecture](#4.1). For current storage engine features, see [Storage Engine](#4).

**Sources**: [README.md:31-80](), [ROADMAP.md:1-115]()

---

## Architectural Philosophy

The Modern Mainframe paradigm rejects the separation of "application server" and "database server" inherited from hardware constraints that no longer exist. Instead, OxiBase positions the database as the active computational core, co-locating logic and data to eliminate network latency and serialization overhead.

### Infrastructure as Data

The fundamental principle: cluster configuration, sharding protocols, access controls, and deployment procedures manifest as transactional rows in system catalog tables. Mutations to the `sys_nodes` table trigger autonomous cluster reconfiguration. This eliminates external orchestration frameworks like Kubernetes.

### Unikernel-First Design

By compiling OxiBase into specialized machine images containing only database and application logic, the system eliminates general-purpose OS overhead. The application becomes synonymous with the kernel, with no intermediary user space layer.

**Sources**: [README.md:39-69]()

---

## Evolution Path: Four Phases

```mermaid
graph LR
    subgraph "Current: Embedded"
        DB1["Database API"]
        MVCC1["MVCCEngine"]
        WAL1["WAL"]
        DB1 --> MVCC1
        MVCC1 --> WAL1
    end
    
    subgraph "Phase 1: Server"
        Wasm["wasmtime integration"]
        SysProc["sys_procedures table"]
        TUI["ratatui TUI"]
        SchemaDiff["Schema Diff Engine"]
    end
    
    subgraph "Phase 2: Async"
        SkipLocked["SELECT FOR UPDATE<br/>SKIP LOCKED"]
        WALRepl["ReplicationStream"]
        Worker["Worker Node<br/>(no storage)"]
    end
    
%% Phase 3
    subgraph "Phase 3: Distributed"
        Raft["openraft consensus"]
        Sharding["Consistent Hashing"]
        GeoShard["Geo-Sharding"]
        SWIM["SWIM gossip"]
    end
    
    subgraph "Phase 4: Autonomous"
        AutoInfra["AWS/GCP API clients"]
        WASINN["wasi-nn ML"]
        GPU["PCI passthrough GPU"]
    end
    
    Current --> Phase1["Phase 1"]
    Phase1 --> Phase2["Phase 2"]
    Phase2 --> Phase3["Phase 3"]
    Phase3 --> Phase4["Phase 4"]
```

**Sources**: [ROADMAP.md:3-115]()

---

## Phase 1: Database as Server

**Goal**: Eliminate separate backend API layers (Deno/Python/Node.js) and external migration tools.

### WebAssembly Stored Procedures

The core innovation embeds `wasmtime` (WebAssembly runtime) directly into the database kernel. User functions compile to Wasm bytecode and execute inside transaction boundaries.

#### Architecture

```mermaid
graph TB
    Client["SQL Client"]
    
    subgraph "OxiBase Kernel"
        Parser["SQL Parser"]
        
        CreateFunc["CREATE FUNCTION handler"]
        
        SysProc["sys_procedures table<br/>Columns:<br/>- name TEXT<br/>- wasm_binary BLOB<br/>- signature TEXT"]
        
        CallHandler["Function Call Handler"]
        
        WasmRuntime["wasmtime::Engine<br/>wasmtime::Store"]
        
        HostFuncs["Host Functions<br/>wasm_query()<br/>wasm_execute()<br/>wasm_begin_txn()"]
        
        MVCCEngine["MVCCEngine"]
        ExprVM["ExprVM"]
    end
    
    Client -->|"CREATE FUNCTION add_user(...)"| Parser
    Parser --> CreateFunc
    CreateFunc -->|"INSERT wasm_binary"| SysProc
    
    Client -->|"SELECT add_user(1, 'Alice')"| Parser
    Parser --> CallHandler
    CallHandler -->|"load wasm_binary"| SysProc
    CallHandler --> WasmRuntime
    
    WasmRuntime -->|"call back"| HostFuncs
    HostFuncs --> MVCCEngine
    HostFuncs --> ExprVM
    
    WasmRuntime -->|"return Value"| Client
```

**Key Implementation Details**:

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Runtime | `wasmtime` crate | Execute Wasm bytecode with sandboxing |
| Storage | `sys_procedures` system table | Store compiled Wasm binaries |
| Interface | Host functions (linker exports) | Zero-copy memory sharing between Wasm and SQL engine |
| Isolation | WASI permissions | Restrict filesystem/network access per function |

**Host Function API** (planned):
```rust
// Callable from Wasm guest code
fn wasm_query(sql: *const u8, sql_len: usize) -> i32;
fn wasm_execute(sql: *const u8, sql_len: usize) -> i32;
fn wasm_begin_txn() -> i32;
fn wasm_commit_txn(txn_id: i32) -> i32;
```

This allows business logic (written in Rust, TypeScript, Python) to run inside the transaction boundary, accessing the `MVCCEngine` without network round-trips.

**Sources**: [ROADMAP.md:7-12](), [README.md:72-75]()

---

### Terminal User Interface (TUI)

A `ratatui`-based terminal interface provides vim-like table navigation, procedure debugging, and real-time performance monitoring.

#### Features

```mermaid
graph LR
    subgraph "TUI Components"
        Nav["Data Navigator<br/>(vim keybindings)"]
        Debugger["Procedure Debugger<br/>(Wasm step-through)"]
        Monitor["Performance Monitor<br/>(IOPS, locks, WAL)"]
    end
    
    subgraph "Backend Hooks"
        WasmtimeDebug["wasmtime debug info"]
        MVCCStats["TransactionRegistry stats"]
        WALMetrics["WAL flush rates"]
    end
    
    Nav --> MVCCStats
    Debugger --> WasmtimeDebug
    Monitor --> MVCCStats
    Monitor --> WALMetrics
```

The debugger leverages `wasmtime`'s debug capabilities to pause execution, inspect stack frames, and view variable states directly in the terminal.

**Sources**: [ROADMAP.md:14-21]()

---

### Declarative Schema Migration

Rejects imperative "up/down" migration scripts. Users define desired end-state schemas; the built-in Schema Diff Engine calculates and executes changes atomically.

#### Schema Diff Engine Architecture

```mermaid
graph TB
    Desired["Desired Schema<br/>(YAML/SQL)"]
    
    Live["Live Schema<br/>(MVCCEngine.schemas)"]
    
    Differ["Schema Diff Engine"]
    
    DAG["Change DAG<br/>- Add column<br/>- Create index<br/>- Add constraint"]
    
    Lock["Catalog Lock<br/>(RwLock)"]
    
    DDLExec["DDL Executor"]
    
    WAL["WAL Manager"]
    
    Desired --> Differ
    Live --> Differ
    
    Differ --> DAG
    DAG --> Lock
    Lock --> DDLExec
    DDLExec --> WAL
```

**Online DDL Capabilities**:
- Non-blocking column additions (append-only schema evolution)
- Index creation with concurrent reads
- Conflict detection for destructive changes

This integrates with the existing `DDLExecutor` at [src/executor/ddl.rs](), extending it to support declarative diffing.

**Sources**: [ROADMAP.md:23-28]()

---

## Phase 2: Async Engine

**Goal**: Eliminate external message brokers (Kafka/RabbitMQ) and enable elastic compute scaling.

### Persistent Queues via SKIP LOCKED

Tables become queues. A "job" is a row with a status column. Multiple consumers grab distinct jobs without blocking.

#### SQL Syntax

```sql
-- Consumer 1 (thread/node)
BEGIN;
SELECT * FROM jobs 
WHERE status = 'pending' 
ORDER BY created_at 
LIMIT 10
FOR UPDATE SKIP LOCKED;

UPDATE jobs SET status = 'processing', worker_id = 'node-1' 
WHERE id IN (...);
COMMIT;
```

#### Implementation Approach

Modifies the MVCC scanner in `VersionStore` to:
1. Check `is_visible()` for transaction isolation
2. Skip rows already locked by other transactions (`uncommitted_writes` tracking)
3. Acquire row locks only for successfully fetched rows

**Code Integration Points**:
- Extend `VersionStore::scan()` at [src/storage/mvcc/version_store.rs]()
- Add lock checking to version visibility logic
- Track lock ownership in `TransactionRegistry`

**Sources**: [ROADMAP.md:34-39]()

---

### WAL Replication for High Availability

```mermaid
graph LR
    subgraph "Leader Node"
        LeaderWAL["WAL Manager"]
        LeaderMVCC["MVCCEngine<br/>(Read-Write)"]
        ReplStream["ReplicationStream"]
    end
    
    subgraph "Follower Node 1"
        F1Recv["WAL Receiver"]
        F1Replay["Replay Engine"]
        F1MVCC["MVCCEngine<br/>(Read-Only)"]
    end
    
    subgraph "Follower Node 2"
        F2Recv["WAL Receiver"]
        F2Replay["Replay Engine"]
        F2MVCC["MVCCEngine<br/>(Read-Only)"]
    end
    
    LeaderMVCC --> LeaderWAL
    LeaderWAL -->|"stream WAL entries"| ReplStream
    ReplStream -->|"TCP"| F1Recv
    ReplStream -->|"TCP"| F2Recv
    
    F1Recv --> F1Replay
    F1Replay --> F1MVCC
    
    F2Recv --> F2Replay
    F2Replay --> F2MVCC
```

**Implementation Plan**:
1. Add `ReplicationStream` trait to `WALManager` at [src/storage/mvcc/wal.rs]()
2. Stream `WALEntry` records over TCP
3. Followers replay entries into their local `VersionStore`
4. Use existing two-phase replay mechanism from crash recovery

This provides read scalability while maintaining single-writer consistency.

**Sources**: [ROADMAP.md:41-45]()

---

### Worker Nodes: Compute-Storage Separation

A new node type with no storage engine. Worker nodes pull data from Data nodes, execute Wasm logic, and return results.

#### Topology

```mermaid
graph TB
    Client["Client"]
    
    Leader["Leader Node<br/>(Raft coordinator)"]
    
    Worker1["Worker Node 1<br/>- wasmtime runtime<br/>- No VersionStore<br/>- Stateless"]
    
    Worker2["Worker Node 2<br/>- wasmtime runtime<br/>- No VersionStore<br/>- Stateless"]
    
    Data1["Data Node 1<br/>- MVCCEngine<br/>- VersionStore<br/>- Indexes"]
    
    Data2["Data Node 2<br/>- MVCCEngine<br/>- VersionStore<br/>- Indexes"]
    
    Client -->|"complex query"| Leader
    Leader -->|"route to worker"| Worker1
    Leader -->|"route to worker"| Worker2
    
    Worker1 -->|"fetch data"| Data1
    Worker1 -->|"fetch data"| Data2
    
    Worker2 -->|"fetch data"| Data1
    Worker2 -->|"fetch data"| Data2
```

**Design Principles**:
- Workers spawn/terminate elastically based on load
- Stateless: can be killed without data loss
- Pull-based: workers fetch data over network RPC
- Compute-intensive: run aggregations, Wasm procedures, joins

**Sources**: [ROADMAP.md:47-56]()

---

## Phase 3: Distributed System

**Goal**: Horizontal scalability with linearizable consistency and geo-replication.

### Multi-Master Consensus via Raft

Transition from simple WAL replication to consensus-based replication using `openraft`.

#### Raft Integration Architecture

```mermaid
graph TB
    subgraph "Raft Group"
        Node1["Node 1 (Leader)<br/>openraft::Raft"]
        Node2["Node 2 (Follower)<br/>openraft::Raft"]
        Node3["Node 3 (Follower)<br/>openraft::Raft"]
    end
    
    Client["Client"]
    
    ConsensusLog["Consensus Log<br/>(replicated)"]
    
    AppLayer["Application Layer"]
    
    subgraph "Per-Node Storage"
        MVCC1["MVCCEngine 1"]
        MVCC2["MVCCEngine 2"]
        MVCC3["MVCCEngine 3"]
    end
    
    Client -->|"write request"| Node1
    Node1 -->|"propose entry"| ConsensusLog
    ConsensusLog -->|"replicate"| Node2
    ConsensusLog -->|"replicate"| Node3
    
    Node2 -->|"ack"| Node1
    Node3 -->|"ack"| Node1
    
    Node1 -->|"commit notification"| AppLayer
    AppLayer --> MVCC1
    
    Node2 -.->|"apply"| MVCC2
    Node3 -.->|"apply"| MVCC3
```

**Key Properties**:
- **Linearizable consistency**: All nodes agree on write order
- **Leader election**: Automatic failover if leader crashes
- **Log compaction**: Snapshot mechanism to prevent unbounded growth

**Implementation Strategy**:
1. Wrap `MVCCEngine` as a Raft state machine
2. Translate SQL writes to Raft log entries
3. Apply committed entries to `VersionStore`

**Sources**: [ROADMAP.md:63-67]()

---

### Sharding via Consistent Hashing

Automatic data distribution across nodes using ring topology.

```mermaid
graph LR
    subgraph "Consistent Hash Ring"
        Node1["Node 1<br/>Range: 0-85"]
        Node2["Node 2<br/>Range: 86-170"]
        Node3["Node 3<br/>Range: 171-255"]
        
        Node1 -->|"ring"| Node2
        Node2 -->|"ring"| Node3
        Node3 -->|"ring"| Node1
    end
    
    Router["Hash Router<br/>hash(primary_key) % 256"]
    
    Table["orders table"]
    
    Table --> Router
    Router -->|"key=100"| Node2
    Router -->|"key=200"| Node3
    Router -->|"key=50"| Node1
```

**Rebalancing Process**:
1. New node joins cluster
2. Cluster calculates shard ownership (consistent hash)
3. Existing nodes stream SSTables (from `VersionStore`) to new node
4. Atomic cutover when data synchronized

This integrates with the existing `VersionStore` structure at [src/storage/mvcc/version_store.rs]().

**Sources**: [ROADMAP.md:69-74]()

---

### Geo-Sharding for Locality

```mermaid
graph TB
    subgraph "EU Region"
        EUData["Data Node EU<br/>Shard: region='eu'<br/>GDPR compliant"]
    end
    
    subgraph "US Region"
        USData["Data Node US<br/>Shard: region='us'"]
    end
    
    subgraph "APAC Region"
        APACData["Data Node APAC<br/>Shard: region='apac'"]
    end
    
    Leader["Leader Node<br/>(Global coordinator)"]
    
    EU_Client["EU Client"]
    US_Client["US Client"]
    
    EU_Client -->|"SELECT * WHERE region='eu'"| Leader
    Leader --> EUData
    
    US_Client -->|"SELECT * WHERE region='us'"| Leader
    Leader --> USData
```

**SQL Syntax**:
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    region TEXT,
    name TEXT
) PARTITION BY LIST (region);

CREATE TABLE users_eu PARTITION OF users
    FOR VALUES IN ('eu-west', 'eu-central');
    
CREATE TABLE users_us PARTITION OF users
    FOR VALUES IN ('us-east', 'us-west');
```

**Benefits**:
- Regulatory compliance (GDPR: EU data stays in EU)
- Latency optimization (speed of light)
- Failure isolation (regional outage doesn't affect global cluster)

**Sources**: [ROADMAP.md:76-79]()

---

### SWIM Gossip Protocol

Scalable Weakly-consistent Infection-style Process Group Membership for failure detection.

```mermaid
graph TB
    Node1["Node 1"]
    Node2["Node 2"]
    Node3["Node 3"]
    Node4["Node 4"]
    Node5["Node 5"]
    
    Node1 -->|"ping"| Node2
    Node1 -->|"ping"| Node3
    
    Node2 -->|"ack"| Node1
    Node3 -->|"ack"| Node1
    
    Node4 -.->|"no ack (dead)"| Node1
    
    Node1 -->|"gossip: Node4 dead"| Node2
    Node2 -->|"gossip: Node4 dead"| Node5
```

**Failure Detection**:
1. Nodes send periodic heartbeats
2. If node stops responding, mark as "suspicious"
3. Gossip suspicion to cluster
4. If multiple nodes agree, mark node as dead
5. Trigger automatic rebalancing

No central coordination required (no Zookeeper dependency).

**Sources**: [ROADMAP.md:81-84]()

---

## Phase 4: Autonomous Cloud

**Goal**: Self-managing infrastructure and AI platform capabilities.

### Infrastructure as Data: Auto-Scaling

The database directly manages its own cloud resources.

```mermaid
graph TB
    Monitor["Internal Monitor<br/>CPU > 80%<br/>Disk > 90%"]
    
    SysNodes["sys_nodes table<br/>Columns:<br/>- node_id TEXT<br/>- status TEXT<br/>- instance_type TEXT<br/>- cloud_provider TEXT"]
    
    AutoInfra["Auto-Infra Manager<br/>- AWS SDK client<br/>- GCP SDK client<br/>- Azure SDK client"]
    
    CloudAPI["Cloud Provider API<br/>POST /run-instances"]
    
    NewNode["New OxiBase<br/>Unikernel Node"]
    
    SWIM["SWIM Gossip<br/>Auto-discovery"]
    
    Monitor -->|"INSERT INTO sys_nodes"| SysNodes
    SysNodes -->|"trigger"| AutoInfra
    AutoInfra --> CloudAPI
    CloudAPI -->|"spawn VM"| NewNode
    NewNode -->|"join cluster"| SWIM
```

**Workflow**:
1. `Monitor` detects high load (CPU > 80%)
2. Inserts row into `sys_nodes`: `status='pending_creation'`
3. `AutoInfra` background thread polls `sys_nodes`
4. Executes cloud API: `aws ec2 run-instances --image-id oxibase-unikernel`
5. New node boots, reads cluster config from consensus
6. Joins via SWIM gossip
7. Updates `sys_nodes`: `status='active'`

**Self-Termination**:
```sql
-- User deletes row
DELETE FROM sys_nodes WHERE node_id = 'node-xyz';

-- AutoInfra intercepts, calls:
-- aws ec2 terminate-instances --instance-ids i-xyz
```

This eliminates Terraform, Ansible, and Kubernetes operators.

**Sources**: [ROADMAP.md:91-96](), [README.md:55-60]()

---

### In-Database Machine Learning

#### wasi-nn Integration

```mermaid
graph TB
    SQLQuery["SELECT PREDICT(model_id, features)<br/>FROM transactions"]
    
    Parser["SQL Parser"]
    
    PredictFunc["PREDICT() function"]
    
    ModelTable["sys_models table<br/>- model_id<br/>- onnx_binary BLOB"]
    
    WasiNN["wasi-nn runtime<br/>ONNX/TensorFlow Lite"]
    
    GPU["GPU (optional)<br/>PCI passthrough"]
    
    Result["Prediction result"]
    
    SQLQuery --> Parser
    Parser --> PredictFunc
    PredictFunc -->|"load model"| ModelTable
    PredictFunc --> WasiNN
    WasiNN -.->|"accelerate"| GPU
    WasiNN --> Result
```

**Use Cases**:
- Real-time fraud detection within transaction scope
- Recommendation systems (query-time inference)
- Anomaly detection in streaming data

**Zero-Copy GPU Access**:
For unikernels, GPU memory can be mapped directly into the OxiBase address space via PCI passthrough. No copying data over PCIe to user-space and back.

**Sources**: [ROADMAP.md:98-102](), [ROADMAP.md:104-109]()

---

### Federated Learning (Phase 4 Endgame)

```mermaid
graph TB
    subgraph "Data Node 1<br/>(US-West)"
        Shard1["User data shard 1"]
        GPU1["Local GPU"]
        LocalTrain1["Local Training"]
    end
    
    subgraph "Data Node 2<br/>(EU-West)"
        Shard2["User data shard 2"]
        GPU2["Local GPU"]
        LocalTrain2["Local Training"]
    end
    
    subgraph "Data Node 3<br/>(APAC)"
        Shard3["User data shard 3"]
        GPU3["Local GPU"]
        LocalTrain3["Local Training"]
    end
    
    Leader["Leader Node<br/>Gradient Aggregator"]
    
    GlobalModel["Global Model<br/>(sys_models table)"]
    
    Shard1 --> LocalTrain1
    LocalTrain1 --> GPU1
    GPU1 -->|"gradients only"| Leader
    
    Shard2 --> LocalTrain2
    LocalTrain2 --> GPU2
    GPU2 -->|"gradients only"| Leader
    
    Shard3 --> LocalTrain3
    LocalTrain3 --> GPU3
    GPU3 -->|"gradients only"| Leader
    
    Leader --> GlobalModel
```

**Key Properties**:
- **Data never moves**: Each node trains on its local shard
- **Gradient aggregation**: Leader combines weight updates using federated averaging
- **Privacy preserving**: Satisfies GDPR (raw data never leaves region)
- **Petabyte scale**: Train on full dataset without network bottlenecks

**Sources**: [ROADMAP.md:111-115]()

---

## Current vs Future State Comparison

| Aspect | Current (Embedded) | Future (Mainframe) |
|--------|-------------------|-------------------|
| **Deployment** | Single process, library linking | Distributed unikernel cluster |
| **Logic Location** | Application code (Rust/Python/JS) | Wasm stored procedures inside DB |
| **Scaling** | Vertical (bigger machine) | Horizontal (add nodes dynamically) |
| **Orchestration** | Manual (systemd/Docker) | Self-managing (`sys_nodes` table) |
| **Consistency** | ACID (single node) | Linearizable (Raft consensus) |
| **Replication** | WAL snapshots (manual restore) | Continuous Raft replication |
| **Sharding** | None (single database file) | Automatic consistent hashing |
| **Queues** | External (Redis/RabbitMQ) | Built-in (`SELECT FOR UPDATE SKIP LOCKED`) |
| **Schema Migration** | External tools (Flyway/Liquibase) | Declarative diff engine |
| **ML Inference** | External service | In-database (`wasi-nn`) |
| **Infrastructure** | External (Terraform/K8s) | Self-provisioning (cloud API clients) |

**Sources**: [README.md:22-27](), [README.md:39-80]()

---

## Code Entity Mapping: Current → Future

The following table maps existing code modules to their future distributed equivalents:

| Current Module | Future Component | Status |
|----------------|------------------|--------|
| `src/api/database.rs` | Leader Node API | Extends to route queries |
| `src/storage/mvcc/engine.rs` (`MVCCEngine`) | Data Node Storage | Becomes sharded instance |
| `src/storage/mvcc/wal.rs` (`WALManager`) | `ReplicationStream` | Add streaming interface |
| `src/storage/mvcc/transaction_registry.rs` | Distributed Transaction Coordinator | Add 2PC for cross-shard txns |
| `src/executor/query.rs` (`QueryExecutor`) | Worker Node Executor | Becomes stateless |
| `src/parser/` | Unchanged | Same SQL parser |
| (new) | `wasmtime::Engine` | Stored procedure runtime |
| (new) | `openraft::Raft` | Consensus layer |
| (new) | `sys_procedures` table | Wasm binary storage |
| (new) | `sys_nodes` table | Cluster topology |
| (new) | `sys_models` table | ML model storage |
| (new) | `ratatui` TUI | Developer interface |
| (new) | SWIM gossip | Failure detection |
| (new) | Auto-Infra Manager | Cloud API integration |

**Sources**: [src/api/database.rs](), [src/storage/mvcc/engine.rs](), [src/storage/mvcc/wal.rs](), [src/executor/query.rs]()

---

## Implementation Timeline

Based on the phased roadmap:

```mermaid
gantt
    title "OxiBase Evolution Timeline"
    dateFormat YYYY-MM
    
    section "Phase 1: Server"
    Wasm Integration         :2025-01, 3M
    sys_procedures table     :2025-02, 1M
    TUI Development          :2025-03, 2M
    Schema Diff Engine       :2025-04, 2M
    
    section "Phase 2: Async"
    SKIP LOCKED              :2025-05, 1M
    WAL Replication          :2025-06, 2M
    Worker Nodes             :2025-07, 2M
    
    section "Phase 3: Distributed"
    Raft Integration         :2025-09, 3M
    Sharding                 :2025-12, 3M
    Geo-Sharding             :2026-03, 2M
    SWIM Gossip              :2026-04, 1M
    
    section "Phase 4: Autonomous"
    Auto-Infra Manager       :2026-06, 3M
    wasi-nn Integration      :2026-09, 2M
    GPU Passthrough          :2026-11, 2M
    Federated Learning       :2027-01, 3M
```

**Sources**: [ROADMAP.md:3-115]()

---

## Migration Path for Users

Current users of the embedded library can migrate incrementally:

### Step 1: Library → Stored Procedures
```rust
// Before: Application logic in Rust
let result = db.query("SELECT * FROM users WHERE active = true")?;
for row in result {
    send_email(row.get("email")?);
}

// After: Logic in Wasm stored procedure
db.execute(r#"
    CREATE FUNCTION notify_active_users() RETURNS INTEGER
    LANGUAGE wasm
    AS $$ /* Wasm binary */ $$
"#)?;

db.execute("SELECT notify_active_users()")?;
```

### Step 2: Single Node → Replicated Cluster
```sql
-- Create follower nodes
INSERT INTO sys_nodes (node_id, role, endpoint)
VALUES 
    ('follower-1', 'read-replica', 'https://10.0.1.10:5432'),
    ('follower-2', 'read-replica', 'https://10.0.1.11:5432');

-- Writes still go to leader, reads distributed automatically
```

### Step 3: Enable Consensus
```sql
-- Promote cluster to multi-master
ALTER CLUSTER SET consensus_mode = 'raft';

-- Now linearizable writes across all nodes
```

**Sources**: [ROADMAP.md:3-12]()

---

## Open Research Questions

Several technical challenges remain:

1. **Wasm-to-SQL zero-copy memory sharing**: How to avoid serialization overhead when Wasm calls back into SQL engine?
2. **Cross-shard transactions**: Implement two-phase commit while maintaining MVCC snapshot isolation guarantees.
3. **Unikernel GPU drivers**: PCI passthrough for Nvidia GPUs without proprietary kernel modules.
4. **Adaptive sharding**: Automatically detect hotspots and split/merge shards based on load.
5. **Consensus performance**: Can Raft keep up with millions of transactions per second? May need custom consensus protocol.

**Sources**: [README.md:41-49]()