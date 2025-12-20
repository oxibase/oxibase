# Roadmap

### Phase 1: The "Database as Server" (Logic & Usability)

*Goal: Eliminate the need for a separate backend API layer (Deno/Lua/Python) and external migration tools.*

**1. Stored Procedures (Wasm Integration)**

* **The Build:** Embed `wasmtime` into the Oxibase kernel.
* **The Mechanism:** Create a `sys_procedures` table. When a user runs `CREATE FUNCTION`, we compile the code to Wasm and store the binary blob in that table.
* **The Interface:** Implement "Host Functions" allowing Wasm to call back into the SQL engine (zero-copy memory sharing).
* **Result:** You can write business logic (Rust/TS) that runs *inside* the transaction.

**2. The TUI (Terminal User Interface)**

* **The Build:** Use `ratatui` (Rust).
* **The Features:**
* **Data Navigator:** Vim-like traversal of tables and rows.
* **Procedure Debugger:** A step-through debugger for the stored Wasm procedures. Since we control the runtime, we can pause execution, inspect stack frames, and view variable states directly in the terminal.
* **Performance Monitor:** Real-time graphs of IOPS, lock contention, and WAL flush rates.



**3. Declarative Schema Migration (Infrastructure as Data)**

* **The Philosophy:** Rejection of imperative "up/down" scripts.
* **The Mechanism:** Users define the *desired* end-state of a table in a generic format (YAML/SQL). Oxibase calculates the diff between the live schema and the desired schema, generates the DAG of changes, and executes them safely.
* **The Tech:** A built-in "Schema Diff Engine" that locks the catalog, detects conflicts, and performs online DDL changes (no downtime column additions).


### Phase 2: The Async Engine (Scale-Out Preliminaries)

*Goal: Eliminate the need for external Message Brokers (Kafka/RabbitMQ) and separate Compute clusters.*

**4. Persistent Queues (SKIP LOCKED)**

* **The Build:** Modify the MVCC scanner to support `SELECT ... FOR UPDATE SKIP LOCKED`.
* **The Mechanism:** This allows multiple consumer threads (or nodes) to grab distinct jobs from a table without blocking each other.
* **The Vision:** Tables *are* queues. A "job" is just a row with a status column.

**5. Master-Read-Read (WAL Replication)**

* **The Build:** Implement a `ReplicationStream` trait in `wal_manager.rs`.
* **The Topology:** One Read-Write leader. Multiple Read-Only followers.
* **The Mechanism:** The Leader streams WAL entries over TCP. Followers replay the WAL to their local MemTable/SSTables. This provides High Availability (HA) for reads.

**6. The Worker Node (Separation of Compute & Storage)**

* **The Concept:** A new node type that *has no storage engine*.
* **The Flow:**
* Client sends a heavy calculation request to the Cluster.
* The Leader routes the request to a "Worker Node."
* The Worker Node pulls data from the "Data Node" (via network), runs the Wasm logic, and returns the result.


* **Result:** Elastic scaling of logic independent of storage size.


### Phase 3: The Distributed System (Consensus & Geo-Scale)

*Goal: Achieve infinite horizontal scale and global resilience.*

**7. Multi-Master (Consensus)**

* **The Build:** Integrate `openraft`.
* **The Mechanism:** Transition from simple WAL shipping to a Consensus Log. Every write is proposed to a Raft group.
* **The Guarantee:** Linearizable consistency. No more "split-brain" scenarios.

**8. Data Rebalancing (Sharding)**

* **The Logic:** Implement Consistent Hashing (e.g., Ring Topology).
* **The Automation:** When a new Data Node joins, the cluster automatically calculates which shard ranges it owns. Existing nodes background-stream the relevant SSTables to the new node.
* **Zero Downtime:** The switch-over happens atomically once data is synchronized.

**9. Geo-Sharding (Locality)**

* **The Feature:** `PARTITION BY REGION`.
* **The Logic:** Rows tagged with `region='eu-west'` are physically stored only on nodes tagged `eu-west`.
* **The Benefit:** Regulatory compliance (GDPR) and speed of light latency optimizations for local users.

**10. Gossip Protocol**

* **The Build:** Implement SWIM (Scalable Weakly-consistent Infection-style Process Group Membership).
* **The Usage:** Nodes "gossip" heartbeats. If a node stops gossiping, the cluster marks it dead and triggers rebalancing automatically. No central "Zookeeper" required.


### Phase 4: The Autonomous Cloud (The "Singularity")

*Goal: The database manages its own physical existence and evolves into an AI platform.*

**11. Auto-Infra Management (The DB is the Terraform)**

* **The Concept:** The Unikernel includes lightweight API clients for AWS/GCP/Azure.
* **The Trigger:** Monitor internal metrics (e.g., "CPU > 80%").
* **The Action:** The Leader executes `POST /run-instances` to AWS EC2 directly to spawn a new Oxibase Unikernel node. It bootstraps itself and joins the cluster via Gossip.
* **Result:** Self-replicating infrastructure.

**12. In-Database ML (Inference)**

* **The Interface:** SQL extensions. `SELECT PREDICT(model_id, input_data) FROM stream`.
* **The Runtime:** Integrate `wasi-nn` (WebAssembly Neural Network interface). This allows loading ONNX or TensorFlow Lite models directly into the kernel memory.
* **Use Case:** Real-time fraud detection within the transaction scope.

**13. GPU Inference**

* **The Hard Part:** Unikernels usually lack proprietary GPU drivers (Nvidia).
* **The Solution:** PCI passthrough. We map the GPU memory space directly to the Oxibase address space.
* **The Benefit:** Zero-latency memory transfer between the Database Buffer Pool and the GPU VRAM. No copying data over PCIe to user-space and back.

**14. GPU Training**

* **The Endgame:** Federated learning.
* **The Mechanism:** The cluster orchestrates a training job where each Data Node trains a local model on its shard of data (using its local GPU) and sends only the weight updates (gradients) to the Leader to aggregate.
* **Result:** Training massive models on petabytes of data without ever moving the data across the network.

