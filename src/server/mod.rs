use crate::api::Database;
use axum::{routing::get, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub mod handlers;

/// The shared application state for the Axum server.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

/// Creates and configures the Axum router for the Auto-API layer.
pub fn create_router(db: Database) -> Router {
    let state = AppState { db: Arc::new(db) };

    Router::new()
        // Define wildcards for the Auto-API
        .route(
            "/api/:table",
            get(handlers::get_table)
                .post(handlers::insert_row)
                .patch(handlers::update_row)
                .delete(handlers::delete_row),
        )
        .with_state(state)
        // Add middleware for logging and CORS
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
