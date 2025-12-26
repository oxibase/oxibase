// Copyright 2025 Stoolap Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Web server implementation using axum

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use oxibase::api::Database;

use crate::web::handlers::{AppState, forum_index, category_page, thread_page, create_thread, create_post, get_threads_fragment, get_posts_fragment};

/// Initialize the forum schema in the database
pub fn init_forum_schema(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let schema_sql = include_str!("../../forum_schema.sql");

    // Split by semicolon and execute each statement
    for statement in schema_sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() && !statement.starts_with("--") {
            db.execute(statement, ())?;
        }
    }

    Ok(())
}

/// Start the web server
pub async fn start_server(db: Database, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(db);

    let app = Router::new()
        // Main routes
        .route("/", get(forum_index))
        .route("/category/:category_id", get(category_page))
        .route("/thread/:thread_id", get(thread_page))

        // HTMX endpoints
        .route("/category/:category_id/thread", post(create_thread))
        .route("/thread/:thread_id/post", post(create_post))
        .route("/category/:category_id/threads", get(get_threads_fragment))
        .route("/thread/:thread_id/posts", get(get_posts_fragment))

        // Static files
        .nest_service("/static", ServeDir::new("static"))

        // Shared state
        .with_state(AppState { db });

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}