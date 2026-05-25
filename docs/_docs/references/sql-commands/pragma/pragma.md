---
layout: default
title: PRAGMA
parent: PRAGMA
grand_parent: SQL Commands
---

# PRAGMA

Sets or gets configuration options.

```sql
-- Set a value
PRAGMA name = value;

-- Get current value
PRAGMA name;
```

#### Supported PRAGMAs

| PRAGMA | Description | Default |
|--------|-------------|---------|
| sync_mode | WAL sync mode (0=None, 1=Normal, 2=Full) | 1 |
| snapshot_interval | Snapshot interval in seconds | 300 |
| keep_snapshots | Number of snapshots to retain | 3 |
| wal_flush_trigger | Operations before WAL flush | 1000 |
| create_snapshot | Manually create a snapshot | - |

See [PRAGMA Commands]({% link _docs/references/pragma-commands.md %}) for detailed documentation.
