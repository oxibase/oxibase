# Data Model

This feature doesn't introduce changes to the engine's core data model (schema, rows, values). The "entities" here are structural for documentation purposes.

## Entities (Documentation Structure)

### 1. Command Categories
- DDL (Data Definition Language)
- DML (Data Manipulation Language)
- DQL (Data Query Language)
- DCL (Data Control Language)
- TCL (Transaction Control Language)
- Utility/Pragma

### 2. SQL Command Documentation Page
- **Attributes:**
  - `name`: Command name (e.g., "SELECT")
  - `category`: Link to the parent category
  - `railroad_diagram_id`: Identifier used by the JS library to render the specific diagram
  - `syntax_description`: Textual explanation of the grammar
  - `examples`: Usage examples