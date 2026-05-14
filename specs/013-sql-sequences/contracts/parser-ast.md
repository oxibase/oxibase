# Interface Contracts: SQL Sequences

## Parser & AST Extensions

### AST Node: `CreateSequence`
```rust
pub struct CreateSequence {
    pub name: String,
    pub if_not_exists: bool,
    pub start_with: Option<i64>,
    pub increment_by: Option<i64>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub cycle: bool,
}
```

### AST Node: `AlterSequence`
```rust
pub struct AlterSequence {
    pub name: String,
    pub if_exists: bool,
    pub restart_with: Option<i64>,
    pub increment_by: Option<i64>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub cycle: Option<bool>,
}
```

### AST Node: `DropSequence`
```rust
pub struct DropSequence {
    pub name: String,
    pub if_exists: bool,
}
```

## Internal Execution APIs

The Execution/Session context exposed to scalar functions needs access to these sequence routines:

```rust
impl Catalog {
    /// Increments the sequence and returns the new value safely across threads.
    /// Returns an error if the sequence hits max_value and CYCLE is false.
    pub fn nextval(&self, seq_name: &str) -> Result<i64>;
    
    /// Overrides the sequence value.
    pub fn setval(&self, seq_name: &str, value: i64, is_called: bool) -> Result<i64>;
}

impl SessionState {
    /// Stores the last generated value for `currval` isolation
    pub fn set_currval(&mut self, seq_name: &str, value: i64);
    
    /// Retrieves the last generated value. Fails if nextval was not called in this session.
    pub fn get_currval(&self, seq_name: &str) -> Result<i64>;
}
```
