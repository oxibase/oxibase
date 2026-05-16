// Copyright 2026 Oxibase Contributors
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

//! Internal Logging System
//!
//! Captures high-severity tracing logs and persists them into the `system.logs` table.

use chrono::Utc;
use crossbeam_channel::{Receiver, Sender};
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use crate::core::Value;
use crate::storage::mvcc::engine::MVCCEngine;
use crate::storage::traits::Engine;

thread_local! {
    /// Flag to prevent the logging flusher thread from logging its own database operations,
    /// which would cause an infinite loop.
    pub static IS_LOG_FLUSHER: RefCell<bool> = const { RefCell::new(false) };
}

/// Represents a captured log event.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: String,
    pub target: String,
    pub message: String,
    pub timestamp: chrono::DateTime<Utc>,
}

/// Custom tracing layer that pushes high-severity logs into a crossbeam channel.
pub struct InternalLogLayer {
    sender: Sender<LogEntry>,
}

impl InternalLogLayer {
    pub fn new(sender: Sender<LogEntry>) -> Self {
        Self { sender }
    }
}

impl<S> Layer<S> for InternalLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Skip if we are inside the flusher thread
        let is_flusher = IS_LOG_FLUSHER.with(|f| *f.borrow());
        if is_flusher {
            return;
        }

        // Only capture INFO, WARN, ERROR
        let level = event.metadata().level();
        if level > &Level::INFO {
            return;
        }

        let mut visitor = LogVisitor::default();
        event.record(&mut visitor);

        let entry = LogEntry {
            level: level.to_string(),
            target: event.metadata().target().to_string(),
            message: visitor.message,
            timestamp: Utc::now(),
        };

        // Attempt to send, but do not block if the channel is full
        let _ = self.sender.try_send(entry);
    }
}

#[derive(Default)]
struct LogVisitor {
    message: String,
}

impl tracing::field::Visit for LogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Removing surrounding quotes if it's a plain string
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        }
    }
}

/// Start the background flusher thread.
pub fn start_log_flusher(
    engine: Arc<MVCCEngine>,
    receiver: Receiver<LogEntry>,
) -> Arc<std::sync::atomic::AtomicBool> {
    let shutdown_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag_clone = Arc::clone(&shutdown_flag);

    thread::Builder::new()
        .name("oxibase-log-flusher".to_string())
        .spawn(move || {
            // Mark this thread as the log flusher to prevent infinite loops
            IS_LOG_FLUSHER.with(|f| *f.borrow_mut() = true);

            let batch_size = 100;
            let timeout = Duration::from_secs(1);

            loop {
                if flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let mut entries = Vec::new();

                // Wait for the first message with a timeout
                match receiver.recv_timeout(timeout) {
                    Ok(entry) => {
                        entries.push(entry);
                        // Try to gather more up to batch_size
                        while entries.len() < batch_size {
                            match receiver.try_recv() {
                                Ok(entry) => entries.push(entry),
                                Err(_) => break,
                            }
                        }
                    }
                    Err(_) => continue, // Timeout or disconnected
                }

                if entries.is_empty() {
                    continue;
                }

                // Insert into database
                if let Err(e) = insert_log_batch(&engine, &entries) {
                    // We can't use tracing::error! here because it would loop if IS_LOG_FLUSHER wasn't set.
                    // Even though it is set, printing to stderr is safer.
                    eprintln!("Failed to flush internal logs: {}", e);
                }
            }
        })
        .expect("Failed to spawn log flusher thread");

    shutdown_flag
}

fn insert_log_batch(engine: &MVCCEngine, entries: &[LogEntry]) -> crate::core::Result<()> {
    let mut tx = engine.begin_transaction()?;

    // Get the system.logs table
    let mut table = match tx.get_table("system.logs") {
        Ok(t) => t,
        Err(_) => {
            // Table might not exist yet during startup
            tx.rollback()?;
            return Ok(());
        }
    };

    for entry in entries {
        let ts_value = Value::Timestamp(entry.timestamp);
        let level_value = Value::Text(entry.level.clone().into());
        let target_value = Value::Text(entry.target.clone().into());
        let msg_value = Value::Text(entry.message.clone().into());
        let json_value = Value::null_unknown(); // Placeholder for future use

        // id is AUTO_INCREMENT, so we pass Value::null_unknown() for it
        let row = vec![
            Value::null_unknown(), // id
            ts_value,              // timestamp
            level_value,           // level
            target_value,          // target
            msg_value,             // message
            json_value,            // json_fields
        ];

        table.insert(row.into())?;
    }

    tx.commit()?;
    Ok(())
}
