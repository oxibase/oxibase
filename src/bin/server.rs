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

//! Oxibase Server - PostgreSQL wire protocol server

use std::sync::Arc;

use clap::Parser;
use oxibase::api::Database;

use oxibase::api::pgwire::OxibaseBackendFactory;
use pgwire::tokio::process_socket;

/// Oxibase Server - PostgreSQL wire protocol server
#[derive(Parser, Debug)]
#[command(name = "server")]
#[command(author = "Oxibase Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "PostgreSQL wire protocol server for Oxibase")]
struct Args {
    /// Database path (file://<path> or memory://)
    #[arg(short = 'd', long = "db", default_value = "memory://")]
    db_path: String,

    /// Server host
    #[arg(long = "host", default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(short = 'p', long = "port", default_value = "5433")]
    port: u16,

    /// Maximum number of connections
    #[arg(long = "max-connections", default_value = "100")]
    max_connections: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Open database
    let db = Database::open(&args.db_path)?;

    // Create backend factory
    let factory = Arc::new(OxibaseBackendFactory::new(db));

    // No authentication for now

    // Bind to address
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("Oxibase server listening on {}", addr);
    println!("Database: {}", args.db_path);
    println!("Max connections: {}", args.max_connections);
    println!("Press Ctrl+C to stop");

    // Handle connections
    loop {
        let (socket, peer_addr) = listener.accept().await?;
        println!("New connection from {}", peer_addr);

        let factory_ref = factory.clone();

        tokio::spawn(async move {
            if let Err(e) = process_socket(socket, None, factory_ref).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}
