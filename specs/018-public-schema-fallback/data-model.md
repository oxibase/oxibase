# Data Model: Public Schema Fallback

## Changes to System Tables (`system.procedures`, `system.functions`, `system.triggers`)
No physical schema changes are required for the system tables themselves, as the `schema` column is already defined as `TEXT`. However, the *data* stored in these tables will change:
- `schema` will no longer be `NULL`. It will be populated with `"public"` (or the active session schema).

## Changes to In-Memory Models (`MVCCEngine`)

### 1. `views`
- **Old Type**: `RwLock<FxHashMap<String, Arc<ViewDefinition>>>`
- **New Type**: `RwLock<FxHashMap<String, FxHashMap<String, Arc<ViewDefinition>>>>`
- **Description**: The outer `FxHashMap` uses the schema name as the key (defaulting to `"public"`). The inner `FxHashMap` uses the view name as the key.

### 2. `sequences`
- **Old Type**: `Arc<RwLock<FxHashMap<String, Arc<crate::core::SequenceState>>>>`
- **New Type**: `Arc<RwLock<FxHashMap<String, FxHashMap<String, Arc<crate::core::SequenceState>>>>>`
- **Description**: The outer `FxHashMap` uses the schema name as the key (defaulting to `"public"`). The inner `FxHashMap` uses the sequence name as the key.

## Changes to In-Memory Metadata Structs (`StoredProcedure`, `StoredFunction`, `StoredTrigger`)
No structure changes required. The `schema` field remains `Option<String>`, but the application logic will ensure it is instantiated as `Some(...)` rather than `None`.
