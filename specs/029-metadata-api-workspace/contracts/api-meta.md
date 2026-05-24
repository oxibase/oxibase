# Metadata API Contract

Base Path: `/api/meta`

## Endpoints

### 1. List Schema Objects (Tables, Views, etc.)
- **URL**: `/api/meta/{object_type}` (e.g., `/tables`, `/views`, `/functions`)
- **Method**: `GET`
- **Response** (200 OK):
  ```json
  [
    {
      "schema": "public",
      "name": "users",
      "type": "table"
    }
  ]
  ```

### 2. Get Table Columns
- **URL**: `/api/meta/columns?table_id={schema}.{table}`
- **Method**: `GET`
- **Response** (200 OK):
  ```json
  [
    {
      "name": "id",
      "data_type": "INT",
      "is_nullable": false
    },
    {
      "name": "username",
      "data_type": "VARCHAR",
      "is_nullable": false
    }
  ]
  ```

### 3. Create Table
- **URL**: `/api/meta/tables`
- **Method**: `POST`
- **Body**:
  ```json
  {
    "schema": "public",
    "name": "new_table",
    "columns": [
      {
        "name": "id",
        "data_type": "INT"
      }
    ]
  }
  ```
- **Response** (201 Created):
  ```json
  {
    "status": "success",
    "message": "Table created successfully"
  }
  ```

### 4. Drop Table
- **URL**: `/api/meta/tables/{schema}.{name}`
- **Method**: `DELETE`
- **Response** (200 OK):
  ```json
  {
    "status": "success",
    "message": "Table dropped successfully"
  }
  ```

### 5. Add Column
- **URL**: `/api/meta/columns`
- **Method**: `POST`
- **Body**:
  ```json
  {
    "table_id": "public.users",
    "column": {
      "name": "email",
      "data_type": "VARCHAR"
    }
  }
  ```
- **Response** (200 OK):
  ```json
  {
    "status": "success",
    "message": "Column added successfully"
  }
  ```