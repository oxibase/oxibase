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

pub mod dap;
pub mod handlers;
pub mod meta;
pub mod template;

/// The shared application state for the Axum server.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub debug_controller: Arc<crate::common::debug::DebugController>,
}

/// Creates and configures the Axum router for the Auto-API layer.
pub fn create_router(db: Database) -> Router {
    // Initialize system schemas and tables for template rendering. We ignore errors since they might already exist
    let _ = db.execute("CREATE SCHEMA interface", ());

    let _ = db.execute(
        "CREATE TABLE interface.routes (method TEXT, path TEXT, template_name TEXT, context_query TEXT)",
        (),
    );
    let _ = db.execute(
        "CREATE TABLE interface.templates (name TEXT, content TEXT)",
        (),
    );
    let _ = db.execute(
        "CREATE TABLE interface.templates (name TEXT, content TEXT)",
        (),
    );

    let state = AppState {
        db: Arc::new(db),
        debug_controller: Arc::new(crate::common::debug::DebugController::new()),
    };

    Router::new()
        // Define wildcards for the Auto-API
        .route(
            "/api/rpc/{procedure_name}",
            axum::routing::post(handlers::invoke_procedure),
        )
        .route(
            "/api/data/{table}",
            get(handlers::get_table)
                .post(handlers::insert_row)
                .patch(handlers::update_row)
                .delete(handlers::delete_row),
        )
        .route("/api/sql", axum::routing::post(handlers::execute_sql))
        .route(
            "/workspace/static/js/dap-client.js",
            get(dap::serve_dap_client),
        )
        .route("/workspace/dap-ws", get(dap::dap_ws_handler))
        .route("/api/meta/schemas", get(meta::list_schemas))
        .route(
            "/api/meta/tables",
            get(meta::list_tables).post(meta::create_table),
        )
        .route(
            "/api/meta/tables/{table}",
            axum::routing::delete(meta::drop_table),
        )
        .route("/api/meta/views", get(meta::list_views))
        .route(
            "/api/meta/columns",
            get(meta::list_columns).post(meta::add_column),
        )
        .route("/api/meta/functions", get(meta::list_functions))
        .route("/api/meta/procedures", get(meta::list_functions))
        .route("/api/meta/indexes", get(meta::list_indexes))
        .route("/api/meta/constraints", get(meta::list_constraints))
        .route("/api/meta/triggers", get(meta::list_triggers))
        .fallback(handlers::dynamic_route_handler)
        .with_state(state)
        // Add middleware for logging and CORS
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
