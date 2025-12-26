# Page: DDL Operations

# DDL Operations

<details>
<summary>Relevant source files</summary>

The following files were used as context for generating this wiki page:

- [.gitignore](.gitignore)
- [README.md](README.md)
- [../../../../roadmap.md](../../../../roadmap.md)
- [docs/_config.yml](docs/_config.yml)
- [src/api/database.rs](src/api/database.rs)
- [src/api/transaction.rs](src/api/transaction.rs)
- [src/executor/ddl.rs](src/executor/ddl.rs)
- [src/executor/expression/evaluator_bridge.rs](src/executor/expression/evaluator_bridge.rs)
- [src/executor/expression/mod.rs](src/executor/expression/mod.rs)

</details>



This page documents Data Definition Language (DDL) operations in OxiBase: creating and modifying database schemas, tables, indexes, and views. For data manipulation operations (INSERT, UPDATE, DELETE), see the Query Execution System documentation. For information about data types used in DDL statements, see [Data Types](#5.1).

## DDL Execution Pipeline

DDL statements are executed through the `Executor` which delegates to specialized methods in the DDL module. All DDL operations are recorded to the Write-Ahead Log (WAL) for durability and crash recovery.

**DDL Statement Processing Flow**

```mermaid
graph TB
    SQL["SQL Statement"] --> Parser["Parser<br/>parse_sql()"]
    Parser --> Router{"Statement Type"}
    
    Router -->|CREATE TABLE| CreateTable["execute_create_table()<br/>src/executor/ddl.rs:37"]
    Router -->|DROP TABLE| DropTable["execute_drop_table()<br/>src/executor/ddl.rs:328"]
    Router -->|CREATE INDEX| CreateIndex["execute_create_index()<br/>src/executor/ddl.rs:365"]
    Router -->|DROP INDEX| DropIndex["execute_drop_index()<br/>src/executor/ddl.rs:458"]
    Router -->|ALTER TABLE| AlterTable["execute_alter_table()<br/>src/executor/ddl.rs:503"]
    Router -->|CREATE VIEW| CreateView["execute_create_view()<br/>src/executor/ddl.rs:646"]
    Router -->|DROP VIEW| DropView["execute_drop_view()<br/>src/executor/ddl.rs:669"]
    
    CreateTable --> SchemaBuilder["SchemaBuilder<br/>Build schema from columns"]
    SchemaBuilder --> CheckPK{"PRIMARY KEY<br/>is INTEGER?"}
    CheckPK -->|No| Error["Error: PRIMARY KEY<br/>must be INTEGER"]
    CheckPK -->|Yes| CheckExists{"Table/View<br/>exists?"}
    
    CheckExists -->|Yes, no IF NOT EXISTS| ErrorExists["Error: TableExists"]
    CheckExists -->|Yes, IF NOT EXISTS| Empty["ExecResult::empty()"]
    CheckExists -->|No| CreateOp["MVCCEngine.create_table()"]
    
    CreateOp --> UniqueIdx{"UNIQUE<br/>constraints?"}
    UniqueIdx -->|Yes| CreateIdx["Create unique indexes<br/>format: unique_{table}_{col}"]
    UniqueIdx -->|No| WAL["Record to WAL"]
    CreateIdx --> WAL
    
    CreateIndex --> ValidateCols["Validate columns exist"]
    ValidateCols --> DetermineType["Determine index type<br/>BTREE/HASH/BITMAP"]
    DetermineType --> CreateIdxOp["MVCCTable.create_index()"]
    CreateIdxOp --> WALIdx["Record to WAL"]
    
    AlterTable --> AlterRouter{"Operation Type"}
    AlterRouter -->|ADD COLUMN| AddCol["Add column with DEFAULT"]
    AlterRouter -->|DROP COLUMN| DropCol["Drop column"]
    AlterRouter -->|RENAME COLUMN| RenameCol["Rename column"]
    AlterRouter -->|MODIFY COLUMN| ModifyCol["Modify type/nullable"]
    AlterRouter -->|RENAME TABLE| RenameTable["Rename table"]
    
    AddCol --> WALAlter["Record to WAL"]
    DropCol --> WALAlter
    RenameCol --> WALAlter
    ModifyCol --> WALAlter
    RenameTable --> WALAlter
    
    WAL --> Result["ExecResult"]
    WALIdx --> Result
    WALAlter --> Result
```

Sources: [src/executor/ddl.rs:1-765]()

## CREATE TABLE

The `CREATE TABLE` statement defines a new table with columns, data types, and constraints. OxiBase supports both standard table creation and `CREATE TABLE AS SELECT` for creating tables from query results.

### Basic Syntax

```sql
CREATE TABLE [IF NOT EXISTS] table_name (
    column_name data_type [column_constraint ...],
    ...
    [table_constraint ...]
)
```

**Column Constraints:**
- `PRIMARY KEY` - Designates the primary key (must be INTEGER type)
- `UNIQUE` - Values must be unique across rows
- `NOT NULL` - Column cannot contain NULL values
- `DEFAULT expression` - Default value when not specified
- `CHECK (expression)` - Value must satisfy condition
- `AUTO_INCREMENT` - Auto-generate sequential integers

**Table Constraints:**
- `UNIQUE (col1, col2, ...)` - Multi-column unique constraint

### CREATE TABLE Implementation Details

```mermaid
graph TB
    Start["CREATE TABLE statement"] --> ParseCols["Parse column definitions"]
    ParseCols --> SchemaBuilder["SchemaBuilder.new(table_name)"]
    
    SchemaBuilder --> IterCols["For each column definition"]
    IterCols --> ParseType["parse_data_type()<br/>INTEGER/FLOAT/TEXT/etc"]
    ParseType --> ParseConstraints["Parse constraints"]
    
    ParseConstraints --> CheckPK{"Is PRIMARY KEY?"}
    CheckPK -->|Yes| ValidatePKType{"Type == INTEGER?"}
    ValidatePKType -->|No| PKError["Error: PRIMARY KEY<br/>must be INTEGER"]
    ValidatePKType -->|Yes| AddPK["nullable=false<br/>is_primary_key=true"]
    
    CheckPK -->|No| CheckUnique{"Is UNIQUE?"}
    CheckUnique -->|Yes| TrackUnique["Add to unique_columns"]
    CheckUnique -->|No| ExtractDefault{"Has DEFAULT?"}
    
    ExtractDefault -->|Yes| ParseDefault["Parse DEFAULT expression<br/>format: 'SELECT {expr}'"]
    ExtractDefault -->|No| ExtractCheck{"Has CHECK?"}
    
    ExtractCheck -->|Yes| StoreCheck["Store CHECK expression"]
    ExtractCheck -->|No| AddColumn["schema_builder.add_with_constraints()"]
    
    AddPK --> AddColumn
    TrackUnique --> ExtractDefault
    ParseDefault --> AddColumn
    StoreCheck --> AddColumn
    
    AddColumn --> MoreCols{"More columns?"}
    MoreCols -->|Yes| IterCols
    MoreCols -->|No| TableConstraints["Parse table-level constraints"]
    
    TableConstraints --> BuildSchema["schema_builder.build()"]
    BuildSchema --> CheckTxn{"Active transaction?"}
    
    CheckTxn -->|Yes| TxnCreate["tx.create_table()"]
    CheckTxn -->|No| EngineCreate["engine.create_table()"]
    
    TxnCreate --> CreateUniqueIdx["Create indexes for<br/>UNIQUE constraints"]
    EngineCreate --> CreateUniqueIdx
    
    CreateUniqueIdx --> IndexLoop["For each unique_column"]
    IndexLoop --> CreateIdx["table.create_index()<br/>name: unique_{table}_{col}<br/>unique: true"]
    CreateIdx --> RecordWAL["engine.record_create_index()"]
    RecordWAL --> MoreIdx{"More unique columns?"}
    MoreIdx -->|Yes| IndexLoop
    MoreIdx -->|No| TableUniqueLoop["For multi-column UNIQUE"]
    
    TableUniqueLoop --> CreateMultiIdx["table.create_index()<br/>name: unique_{table}_{i}"]
    CreateMultiIdx --> RecordMultiWAL["engine.record_create_index()"]
    RecordMultiWAL --> MoreMulti{"More table constraints?"}
    MoreMulti -->|Yes| TableUniqueLoop
    MoreMulti -->|No| Done["ExecResult::empty()"]
```

Sources: [src/executor/ddl.rs:36-245](), [src/executor/ddl.rs:716-733]()

### Examples

```sql
-- Simple table with primary key
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    age INTEGER
);

-- Table with constraints
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT NOT NULL,
    price FLOAT DEFAULT 0.0,
    quantity INTEGER CHECK (quantity >= 0),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Multi-column unique constraint
CREATE TABLE user_roles (
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    UNIQUE (user_id, role_id)
);

-- Create table from query
CREATE TABLE high_value_orders AS
SELECT * FROM orders WHERE amount > 1000;
```

### CREATE TABLE AS SELECT

OxiBase supports creating tables from SELECT query results. The schema is inferred from the result columns and data types from the first row.

```mermaid
graph TB
    Start["CREATE TABLE ... AS SELECT"] --> Execute["execute_select()"]
    Execute --> Materialize["Materialize result rows"]
    
    Materialize --> InferSchema["For each result column"]
    InferSchema --> ExtractName["Extract base column name<br/>strip table prefix"]
    ExtractName --> InferType["Infer type from first row"]
    
    InferType --> TypeLogic{"Value type?"}
    TypeLogic -->|Integer| DTInteger["DataType::Integer"]
    TypeLogic -->|Float| DTFloat["DataType::Float"]
    TypeLogic -->|Text| DTText["DataType::Text"]
    TypeLogic -->|Boolean| DTBoolean["DataType::Boolean"]
    TypeLogic -->|Timestamp| DTTimestamp["DataType::Timestamp"]
    TypeLogic -->|Json| DTJson["DataType::Json"]
    TypeLogic -->|Null| DTDefault["DataType::Text (default)"]
    
    DTInteger --> AddNullable["schema_builder.add_nullable()"]
    DTFloat --> AddNullable
    DTText --> AddNullable
    DTBoolean --> AddNullable
    DTTimestamp --> AddNullable
    DTJson --> AddNullable
    DTDefault --> AddNullable
    
    AddNullable --> MoreCols{"More columns?"}
    MoreCols -->|Yes| InferSchema
    MoreCols -->|No| BuildSchema["schema_builder.build()"]
    
    BuildSchema --> CreateTable["engine.create_table()"]
    CreateTable --> InsertRows["Insert materialized rows<br/>within transaction"]
    InsertRows --> Commit["tx.commit()"]
```

Sources: [src/executor/ddl.rs:247-326]()

### PRIMARY KEY Restrictions

**Important:** OxiBase currently only supports INTEGER PRIMARY KEY. This is validated at table creation time and will return an error for other types.

```sql
-- Valid: INTEGER PRIMARY KEY
CREATE TABLE valid (id INTEGER PRIMARY KEY);

-- Invalid: TEXT PRIMARY KEY (will error)
CREATE TABLE invalid (uuid TEXT PRIMARY KEY);
-- Error: PRIMARY KEY column 'uuid' must be INTEGER type
```

Sources: [src/executor/ddl.rs:88-94]()

## DROP TABLE

Removes a table and all its data from the database.

### Syntax

```sql
DROP TABLE [IF EXISTS] table_name
```

### Transaction Behavior

**Warning:** `DROP TABLE` within a transaction has limited rollback support. On rollback, the table schema will be recreated but **data will be lost** because table data is immediately deleted and cannot be recovered.

For recoverable data deletion, use `DELETE FROM table_name` or `TRUNCATE TABLE` instead.

```sql
-- Outside transaction - safe, auto-committed
DROP TABLE users;

-- Within transaction - data loss on rollback
BEGIN;
DROP TABLE orders;  -- Warning: data cannot be recovered on rollback
ROLLBACK;  -- Schema recreated, but data is gone
```

Sources: [src/executor/ddl.rs:328-363]()

## CREATE INDEX

Creates an index on one or more columns to improve query performance. OxiBase supports three index types: BTree, Hash, and Bitmap.

### Syntax

```sql
CREATE [UNIQUE] INDEX [IF NOT EXISTS] index_name
ON table_name (column1, column2, ...)
[USING {BTREE | HASH | BITMAP}]
```

### Index Types

**Index Type Selection:**

```mermaid
graph TB
    CreateIndex["CREATE INDEX statement"] --> ExplicitType{"USING clause<br/>specified?"}
    
    ExplicitType -->|Yes| UseExplicit["Use specified type:<br/>BTREE/HASH/BITMAP"]
    ExplicitType -->|No| AutoSelect["Auto-select based on:<br/>1. Column data type<br/>2. Cardinality<br/>3. Query patterns"]
    
    AutoSelect --> CheckType{"Column type?"}
    CheckType -->|INTEGER/FLOAT/TIMESTAMP| BTree["BTree Index<br/>• Range queries<br/>• ORDER BY<br/>• O(log n)"]
    CheckType -->|TEXT/JSON| Hash["Hash Index<br/>• Equality lookups<br/>• O(1) average<br/>• No range queries"]
    CheckType -->|BOOLEAN| Bitmap["Bitmap Index<br/>• Low cardinality<br/>• Fast AND/OR/NOT<br/>• Minimal space"]
    
    UseExplicit --> BTreeImpl["BTreeIndex<br/>sorted_values: BTreeMap<br/>value_to_rows: HashMap"]
    BTree --> BTreeImpl
    
    UseExplicit --> HashImpl["HashIndex<br/>hash_to_rows<br/>row_to_hash"]
    Hash --> HashImpl
    
    UseExplicit --> BitmapImpl["BitmapIndex<br/>Roaring bitmaps<br/>bitwise operations"]
    Bitmap --> BitmapImpl
    
    BTreeImpl --> Validate["Validate columns exist"]
    HashImpl --> Validate
    BitmapImpl --> Validate
    
    Validate --> CreateOp["table.create_index_with_type()"]
    CreateOp --> RecordWAL["engine.record_create_index()<br/>Persist to WAL"]
```

Sources: [src/executor/ddl.rs:365-456](), [README.md:196-216]()

| Index Type | Best For | Characteristics | Use Cases |
|------------|----------|-----------------|-----------|
| **BTree** | Range queries, sorting | O(log n) lookup, ordered | `created_at BETWEEN ...`<br/>`ORDER BY timestamp` |
| **Hash** | Equality lookups | O(1) average, unordered | `email = 'user@example.com'`<br/>`id = 42` |
| **Bitmap** | Low cardinality | Efficient AND/OR/NOT, compact | `status = 'active'`<br/>`is_deleted = false` |

### Examples

```sql
-- BTree index for range queries (auto-selected for TIMESTAMP)
CREATE INDEX idx_created ON orders(created_at);

-- Hash index for equality lookups (explicit)
CREATE INDEX idx_email ON users(email) USING HASH;

-- Bitmap index for boolean column (auto-selected for BOOLEAN)
CREATE INDEX idx_active ON users(is_active);

-- Unique index (enforces constraint)
CREATE UNIQUE INDEX idx_username ON users(username);

-- Multi-column composite index
CREATE INDEX idx_lookup ON events(user_id, event_type, timestamp);

-- Conditional creation
CREATE INDEX IF NOT EXISTS idx_status ON orders(status);
```

### UNIQUE Indexes

When a column has a `UNIQUE` constraint in `CREATE TABLE`, OxiBase automatically creates a unique index with the naming convention `unique_{table}_{column}`:

```sql
-- This CREATE TABLE statement...
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT UNIQUE
);

-- ...automatically creates this index:
-- CREATE UNIQUE INDEX unique_users_email ON users(email)
```

Sources: [src/executor/ddl.rs:162-178]()

## DROP INDEX

Removes an index from a table.

### Syntax

```sql
DROP INDEX [IF EXISTS] index_name ON table_name
```

### Example

```sql
-- Drop an index
DROP INDEX idx_email ON users;

-- Conditional drop
DROP INDEX IF EXISTS idx_status ON orders;
```

**Note:** The table name is required in OxiBase's DROP INDEX syntax.

Sources: [src/executor/ddl.rs:458-501]()

## ALTER TABLE

Modifies an existing table's structure. All ALTER TABLE operations are transactional and recorded to the WAL.

### Operations

**ALTER TABLE Operations:**

```mermaid
graph LR
    AlterTable["ALTER TABLE table_name"] --> AddColumn["ADD COLUMN<br/>column_name type"]
    AlterTable --> DropColumn["DROP COLUMN<br/>column_name"]
    AlterTable --> RenameColumn["RENAME COLUMN<br/>old TO new"]
    AlterTable --> ModifyColumn["MODIFY COLUMN<br/>column_name type"]
    AlterTable --> RenameTable["RENAME TO<br/>new_table_name"]
    
    AddColumn --> AddImpl["table.create_column_with_default_value()<br/>• Backfill existing rows<br/>• Store default expression"]
    DropColumn --> DropImpl["table.drop_column()<br/>• Remove from schema<br/>• Update all rows"]
    RenameColumn --> RenameImpl["table.rename_column()<br/>• Update schema<br/>• Preserve data"]
    ModifyColumn --> ModifyImpl["table.modify_column()<br/>• Change type/nullable<br/>• Validate data"]
    RenameTable --> RenameTableImpl["tx.rename_table()<br/>• Update schema registry"]
    
    AddImpl --> WAL["Record to WAL"]
    DropImpl --> WAL
    RenameImpl --> WAL
    ModifyImpl --> WAL
    RenameTableImpl --> WAL
```

Sources: [src/executor/ddl.rs:503-644]()

### ADD COLUMN

Adds a new column to an existing table. If a DEFAULT expression is provided, existing rows are backfilled with the default value.

```sql
-- Add nullable column
ALTER TABLE users ADD COLUMN age INTEGER;

-- Add column with default
ALTER TABLE users ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;

-- Add NOT NULL column with default
ALTER TABLE users ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
```

**Implementation Notes:**
- Default expressions are evaluated once at ALTER TABLE time for backfilling existing rows
- The default expression is also stored for use with future INSERT statements
- Expressions like `CURRENT_TIMESTAMP` are evaluated at backfill time, so all existing rows get the same timestamp

Sources: [src/executor/ddl.rs:522-572](), [src/executor/ddl.rs:735-763]()

### DROP COLUMN

Removes a column from the table.

```sql
ALTER TABLE users DROP COLUMN age;
```

Sources: [src/executor/ddl.rs:574-586]()

### RENAME COLUMN

Renames a column while preserving its data and constraints.

```sql
ALTER TABLE users RENAME COLUMN email TO email_address;
```

Sources: [src/executor/ddl.rs:587-603]()

### MODIFY COLUMN

Changes a column's data type or nullability. Data is validated against the new type.

```sql
-- Change type
ALTER TABLE users MODIFY COLUMN age BIGINT;

-- Change nullability
ALTER TABLE users MODIFY COLUMN email TEXT NOT NULL;
```

Sources: [src/executor/ddl.rs:604-626]()

### RENAME TABLE

Renames the entire table.

```sql
ALTER TABLE users RENAME TO customers;
```

Sources: [src/executor/ddl.rs:627-639]()

## CREATE VIEW / DROP VIEW

Views are virtual tables defined by SELECT queries. They can be queried like regular tables but don't store data.

### Syntax

```sql
-- Create view
CREATE [OR REPLACE] VIEW [IF NOT EXISTS] view_name AS
SELECT ...

-- Drop view
DROP VIEW [IF EXISTS] view_name
```

### Examples

```sql
-- Simple view
CREATE VIEW active_users AS
SELECT * FROM users WHERE is_active = true;

-- View with joins and aggregates
CREATE VIEW order_summary AS
SELECT 
    u.name,
    COUNT(o.id) as order_count,
    SUM(o.amount) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name;

-- Query the view
SELECT * FROM active_users WHERE age > 18;

-- Drop view
DROP VIEW IF EXISTS order_summary;
```

**View Implementation:**
- Views are stored in the MVCC engine's view registry
- View names cannot conflict with table names
- Views are expanded inline during query execution
- View definitions are persisted to the WAL

Sources: [src/executor/ddl.rs:646-681]()

## Data Type Mapping

OxiBase's `parse_data_type` function accepts standard SQL type names and aliases, normalizing them to core data types:

| SQL Type Syntax | OxiBase Type | Notes |
|-----------------|--------------|-------|
| `INTEGER`, `INT`, `BIGINT`, `SMALLINT`, `TINYINT` | `DataType::Integer` | 64-bit signed |
| `FLOAT`, `DOUBLE`, `REAL`, `DECIMAL`, `NUMERIC` | `DataType::Float` | 64-bit float |
| `TEXT`, `VARCHAR`, `CHAR`, `STRING`, `CLOB` | `DataType::Text` | UTF-8 string |
| `BOOLEAN`, `BOOL` | `DataType::Boolean` | true/false |
| `TIMESTAMP`, `DATETIME`, `DATE`, `TIME` | `DataType::Timestamp` | Date/time storage |
| `JSON`, `JSONB` | `DataType::Json` | JSON data |
| `BLOB`, `BINARY`, `VARBINARY` | `DataType::Text` | Base64 encoded |

**Type Parsing Logic:**
```sql
-- All these create INTEGER columns:
CREATE TABLE t1 (id INT);
CREATE TABLE t2 (id BIGINT);
CREATE TABLE t3 (id INTEGER);

-- VARCHAR with size is accepted (size ignored):
CREATE TABLE t4 (name VARCHAR(255));  -- Stored as TEXT
```

Sources: [src/executor/ddl.rs:716-733]()

## Constraint Support Matrix

| Constraint | Support Level | Notes |
|------------|---------------|-------|
| `PRIMARY KEY` | ✅ Full | Must be INTEGER type, single column only |
| `UNIQUE` | ✅ Full | Auto-creates unique index, supports multi-column |
| `NOT NULL` | ✅ Full | Enforced at insert/update |
| `DEFAULT` | ✅ Full | Evaluated at schema creation and insert |
| `CHECK` | ⚠️ Partial | Column-level only, stored but not fully enforced |
| `FOREIGN KEY` | ❌ Not supported | Planned for future release |
| `AUTO_INCREMENT` | ✅ Full | Generates sequential integers |

Sources: [src/executor/ddl.rs:76-139]()

## Transaction Support

DDL operations can be executed within transactions, allowing schema changes to be rolled back:

```sql
BEGIN;

-- Create table
CREATE TABLE temp_data (
    id INTEGER PRIMARY KEY,
    value TEXT
);

-- Insert data
INSERT INTO temp_data VALUES (1, 'test');

-- Create index
CREATE INDEX idx_value ON temp_data(value);

-- Rollback everything (table, data, and index)
ROLLBACK;
```

**Transaction Behavior:**
- `CREATE TABLE` - Fully transactional, rollback recreates state
- `DROP TABLE` - **Warning:** Data loss on rollback (schema recreated, data gone)
- `CREATE INDEX` - Fully transactional
- `DROP INDEX` - Fully transactional
- `ALTER TABLE` - Fully transactional
- `CREATE VIEW` - Fully transactional
- `DROP VIEW` - Fully transactional

**Active Transaction Detection:**

```mermaid
graph TB
    DDLStart["DDL Statement"] --> CheckTxn{"Active transaction<br/>in active_transaction?"}
    
    CheckTxn -->|Yes| UseTxn["Use transaction API<br/>tx.create_table()<br/>tx.get_table()<br/>tx.rename_table()"]
    CheckTxn -->|No| UseEngine["Use engine API<br/>engine.create_table()<br/>engine.begin_transaction()"]
    
    UseTxn --> RecordWAL["Record to WAL<br/>(not yet committed)"]
    UseEngine --> AutoCommit["Auto-commit<br/>(immediately durable)"]
    
    RecordWAL --> TxnCommit["Await tx.commit()<br/>or tx.rollback()"]
    TxnCommit --> Commit{"Commit or Rollback?"}
    
    Commit -->|Commit| PersistWAL["Persist all changes<br/>to WAL"]
    Commit -->|Rollback| Undo["Undo schema changes<br/>(data loss for DROP TABLE)"]
    
    AutoCommit --> Done["Changes immediately visible"]
    PersistWAL --> Done
    Undo --> Done
```

Sources: [src/executor/ddl.rs:155-243](), [src/executor/ddl.rs:344-361]()

## Deprecated Features

### COLUMNAR INDEX (Deprecated)

The `CREATE COLUMNAR INDEX` and `DROP COLUMNAR INDEX` syntax is deprecated. Use standard `CREATE INDEX` instead, which automatically selects the optimal index type.

```sql
-- Deprecated (will error):
CREATE COLUMNAR INDEX idx_status ON orders(status);

-- Use this instead:
CREATE INDEX idx_status ON orders(status);
-- Auto-selects Bitmap for BOOLEAN, Hash for TEXT, BTree for numeric types
```

Sources: [src/executor/ddl.rs:683-714]()