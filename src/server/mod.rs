// Copyright 2025 Oxibase Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::api::Database;
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub mod handlers;
pub mod template;

/// The shared application state for the Axum server.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

/// Creates and configures the Axum router for the Auto-API layer.
pub fn create_router(db: Database) -> Router {
    // Initialize system schemas and tables for template rendering. We ignore errors since they might already exist
    let _ = db.execute("CREATE SCHEMA routes", ());
    let _ = db.execute("CREATE SCHEMA templates", ());
    let _ = db.execute(
        "CREATE TABLE routes.definitions (method TEXT, path TEXT, template_name TEXT, context_query TEXT)",
        ()
    );
    let _ = db.execute(
        "CREATE TABLE templates.source (name TEXT, content TEXT)",
        (),
    );

    let state = AppState { db: Arc::new(db) };

    Router::new()
        // Define wildcards for the Auto-API
        .route(
            "/api/rpc/{procedure_name}",
            axum::routing::post(handlers::invoke_procedure),
        )
        .route(
            "/api/{table}",
            get(handlers::get_table)
                .post(handlers::insert_row)
                .patch(handlers::update_row)
                .delete(handlers::delete_row),
        )
        .fallback(handlers::dynamic_route_handler)
        .with_state(state)
        // Add middleware for logging and CORS
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
