---
title: OLTP Use Case 
layout: default
parent: Getting Started
nav_order: 2
---

# OLTP Microservice 

This guide demonstrates building a simple REST microservice using Oxibase as an
embedded database for Online Transaction Processing (OLTP) workloads. We'll
create a task management API with persistent storage.

## Prerequisites

- Rust installed
- Oxibase crate added to your project
- Basic knowledge of REST APIs and async Rust

## Step 1: Set up the Project

Create a new Rust project:

```bash
cargo new task-service
cd task-service
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
oxibase = "0.3"
tokio = { version = "1", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Step 2: Create the Database Schema


Add to the `main.rs` set up persistent database and create tables:

```rust
use oxibase::api::Database;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create persistent database
        let db = Database::open("file://./task_service.db")?;
        
        // Create tables
        db.execute(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                completed BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            (),
        )?;
        
        Ok(AppState { db: Arc::new(db) })
    }
}
```

## Step 3: Define Data Models

Create structs for request/response handling:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}
```

## Step 4: Implement CRUD Operations

Create functions for database operations:

```rust
use oxibase::Value;

impl AppState {
    pub fn get_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let stmt = self.db.prepare("SELECT id, title, description, completed, created_at, updated_at FROM tasks ORDER BY created_at DESC")?;
        let rows = stmt.query(())?;
        
        let mut tasks = Vec::new();
        for row in rows {
            let row = row?;
            tasks.push(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                completed: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            });
        }
        
        Ok(tasks)
    }
    
    pub fn get_task(&self, id: i64) -> Result<Option<Task>, Box<dyn std::error::Error>> {
        let stmt = self.db.prepare("SELECT id, title, description, completed, created_at, updated_at FROM tasks WHERE id = ?")?;
        let mut rows = stmt.query([Value::Integer(id)])?;
        
        if let Some(row) = rows.next() {
            let row = row?;
            Ok(Some(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                completed: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn create_task(&self, task: CreateTask) -> Result<Task, Box<dyn std::error::Error>> {
        let stmt = self.db.prepare(
            "INSERT INTO tasks (title, description) VALUES (?, ?) RETURNING id, title, description, completed, created_at, updated_at"
        )?;
        
        let mut rows = stmt.query([
            Value::Text(task.title.into()),
            Value::Text(task.description.unwrap_or_default().into()),
        ])?;
        let row = rows.next().ok_or("No row returned")??;
        
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            completed: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
    
    pub fn update_task(&self, id: i64, update: UpdateTask) -> Result<Option<Task>, Box<dyn std::error::Error>> {
        // Build dynamic update query
        let mut set_parts = Vec::new();
        let mut params = Vec::new();

        if let Some(title) = &update.title {
            set_parts.push("title = ?");
            params.push(Value::Text(title.clone().into()));
        }

        if let Some(description) = &update.description {
            set_parts.push("description = ?");
            params.push(Value::Text(description.clone().into()));
        }

        if let Some(completed) = update.completed {
            set_parts.push("completed = ?");
            params.push(Value::Boolean(completed));
        }

        if set_parts.is_empty() {
            return self.get_task(id);
        }

        set_parts.push("updated_at = CURRENT_TIMESTAMP");

        let query = format!(
            "UPDATE tasks SET {} WHERE id = ? RETURNING id, title, description, completed, created_at, updated_at",
            set_parts.join(", ")
        );

        params.push(Value::Integer(id));

        let stmt = self.db.prepare(&query)?;
        let mut rows = stmt.query(params.as_slice())?;
        
        if let Some(row) = rows.next() {
            let row = row?;
            Ok(Some(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                completed: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn delete_task(&self, id: i64) -> Result<bool, Box<dyn std::error::Error>> {
        let stmt = self.db.prepare("DELETE FROM tasks WHERE id = ?")?;
        let changes = stmt.execute([Value::Integer(id)])?;
        Ok(changes > 0)
    }
}
```

## Step 5: Create REST API Routes

Implement HTTP handlers:

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;

async fn get_tasks(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.get_tasks() {
        Ok(tasks) => Ok(Json(json!({ "tasks": tasks }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.get_task(id) {
        Ok(Some(task)) => Ok(Json(json!({ "task": task }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state.create_task(payload) {
        Ok(task) => Ok((StatusCode::CREATED, Json(json!({ "task": task })))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTask>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.update_task(id, payload) {
        Ok(Some(task)) => Ok(Json(json!({ "task": task }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_task_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> StatusCode {
    match state.delete_task(id) {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
```

## Step 6: Set up the Server

Create the main application:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let state = AppState::new()?;
    
    // Build routes
    let app = Router::new()
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:id", 
            get(get_task)
            .put(update_task)
            .delete(delete_task_handler)
        )
        .with_state(state);
    
    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

## Step 7: Test the Microservice

Run the service:

```bash
cargo run
```

Test with curl:

```bash
# Create a task
curl -X POST http://127.0.0.1:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn Oxibase", "description": "Complete the getting started guide"}'

# Get all tasks
curl http://127.0.0.1:3000/tasks

# Get a specific task
curl http://127.0.0.1:3000/tasks/1

# Update a task
curl -X PUT http://127.0.0.1:3000/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'

# Delete a task
curl -X DELETE http://127.0.0.1:3000/tasks/1
```

## Summary

You've built an OLTP microservice with Oxibase as the embedded database,
handling CRUD operations with transactions. The service uses persistent storage
and demonstrates RESTful API design with proper error handling.
