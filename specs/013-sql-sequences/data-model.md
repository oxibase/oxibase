# Data Model: SQL Sequences

## Entities

### `SequenceOptions` (Data Structure)
Internal struct representing the defined bounds and behavior of a sequence.

**Fields:**
- `increment_by` (i64): Step size (default 1).
- `start_with` (i64): Initial value (default 1).
- `min_value` (Option<i64>): Lower bound (default 1).
- `max_value` (Option<i64>): Upper bound (default i64::MAX).
- `cycle` (bool): Whether to loop back to min_value/max_value when exhausted (default false).

---

### `SequenceState` (Data Structure)
In-memory representation of an active sequence in the catalog.

**Fields:**
- `current_value` (AtomicI64): The current generated value. Must be highly concurrent.
- `options` (SequenceOptions): The static configuration of the sequence.

---

### `information_schema.sequences` (Virtual Table Schema)
The system view exposed to the user.

**Columns:**
- `sequence_catalog` (String)
- `sequence_schema` (String)
- `sequence_name` (String)
- `data_type` (String)
- `start_value` (String/i64)
- `minimum_value` (String/i64)
- `maximum_value` (String/i64)
- `increment` (String/i64)
- `cycle_option` (String) ("YES" or "NO")

---

### `SessionContext` Updates
The session/execution context requires a new field to track `CURRVAL` isolation.

**New Fields:**
- `sequence_currvals`: A `HashMap<String, i64>` storing the sequence name and the last value retrieved by `NEXTVAL` in *this specific session/execution*.
