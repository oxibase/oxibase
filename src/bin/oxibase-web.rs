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

//! Oxibase Web Server - Forum application web server
//!
//! This binary runs a web server that serves a forum application
//! directly from an Oxibase database using axum, askama, and htmx.

use clap::Parser;
use oxibase::api::Database;

/// Oxibase Web Server
#[derive(Parser, Debug)]
#[command(name = "oxibase-web")]
#[command(author = "Oxibase Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Web server for Oxibase forum application")]
#[command(
    long_about = "Oxibase Web Server serves a forum application directly from an Oxibase database.\n\
    Uses axum for routing, askama for templating, and htmx for dynamic interactions."
)]
struct Args {
    /// Database path (file://<path> or memory://)
    #[arg(short = 'd', long = "db", default_value = "memory://")]
    db_path: String,

    /// Port to bind the web server to
    #[arg(short = 'p', long = "port", default_value = "3000")]
    port: u16,

    /// Initialize forum schema if it doesn't exist
    #[arg(long = "init-forum", default_value = "false")]
    init_forum: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Open the database
    let db = Database::open(&args.db_path)?;

    // Initialize forum schema if requested
    if args.init_forum {
        println!("Initializing forum schema...");
        oxibase_web::init_forum_schema(&db)?;
        println!("Forum schema initialized.");
    }

    // Start the web server
    println!("Starting Oxibase Web Server on port {}", args.port);
    println!("Database: {}", args.db_path);
    println!("Visit http://localhost:{}/ to access the forum", args.port);

    oxibase_web::start_server(db, args.port).await?;

    Ok(())
}