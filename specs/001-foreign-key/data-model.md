# Data Model: Foreign Key Constraints

## AST Extensions

New types to represent referential actions:
```rust
pub enum ReferentialAction {
    Restrict,
    Cascade,
    SetNull,
    NoAction,
}
```

Additions to `TableConstraint` (or equivalent AST node):
```rust
pub enum TableConstraint {
    // ... existing constraints ...
    ForeignKey {
        name: Option<String>,
        column: String, // Single column supported
        foreign_table: String,
        foreign_column: String,
        on_delete: ReferentialAction,
        on_update: ReferentialAction,
    }
}
```

## Schema Extensions

The `Table` definition within the `Schema` needs to track its constraints:

```rust
pub struct ForeignKeyMetadata {
    pub column_id: ColumnId,
    pub referenced_table_id: TableId,
    pub referenced_column_id: ColumnId,
    pub on_delete: ReferentialAction,
    pub on_update: ReferentialAction,
}

// In the Table schema definition:
pub struct TableSchema {
    // ... existing fields
    pub foreign_keys: Vec<ForeignKeyMetadata>,
    
    // Reverse mapping for fast lookups when modifying the referenced table
    // E.g., if this table is referenced by other tables, we need to know who they are
    // to apply CASCADE or RESTRICT
    pub referenced_by: Vec<TableId>, 
}
```

## Validation Logic Flow

1. **Insert into Referencing Table**:
   - Read value of FK column.
   - If not NULL, perform a primary key/unique index lookup on the `referenced_table`.
   - If not found -> Return `Error::ReferentialIntegrityViolation`.

2. **Delete from Referenced Table**:
   - Identify all tables in `referenced_by`.
   - For each referencing table:
     - Scan/Index lookup for rows where FK column == deleted row's PK.
     - If matches found:
       - If `ON DELETE RESTRICT/NO ACTION` -> Return `Error::ReferentialIntegrityViolation`.
       - If `ON DELETE SET NULL` -> Update matching rows, setting FK column to NULL.
       - If `ON DELETE CASCADE` -> Recursively delete matching rows.

3. **Update Referenced Table**:
   - Similar to Delete, but applying `ON UPDATE` actions.