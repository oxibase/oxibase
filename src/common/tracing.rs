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

//! OpenTelemetry Tracing and Internal Ingestion System
//!
//! Captures tracing spans and persists them into the `system.traces` table.
//! Optionally exports telemetry to OTLP compatible endpoints.

use chrono::{DateTime, Utc};
use crossbeam_channel::{Receiver, Sender};
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{Id, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use crate::core::Value;
use crate::storage::mvcc::engine::MVCCEngine;
use crate::storage::traits::Engine;

thread_local! {
    /// Flag to prevent the telemetry flusher thread from tracing its own database operations,
    /// which would cause an infinite loop.
    pub static IS_TELEMETRY_THREAD: RefCell<bool> = const { RefCell::new(false) };
}

/// Represents a captured tracing span event.
#[derive(Debug, Clone)]
pub struct SpanEvent {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub target: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub attributes: Vec<(String, String)>, // Defer JSON stringification
}

/// Custom tracing layer that pushes span events into a crossbeam channel.
pub struct SystemTraceLayer {
    sender: Sender<SpanEvent>,
}

impl SystemTraceLayer {
    pub fn new(sender: Sender<SpanEvent>) -> Self {
        Self { sender }
    }
}

impl<S> Layer<S> for SystemTraceLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &tracing::span::Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if IS_TELEMETRY_THREAD.with(|f| *f.borrow()) {
            return;
        }

        let span = ctx.span(id).expect("Span not found");
        let mut visitor = AttributeVisitor::default();
        attrs.record(&mut visitor);

        let mut ext = span.extensions_mut();

        let span_id_str = id.into_u64().to_string();
        let trace_id = format!("trace-{}", span_id_str); // Fallback

        ext.insert::<(
            Instant,
            DateTime<Utc>,
            String,
            String,
            Vec<(String, String)>,
            String, // trace_id
            String, // span_id
        )>((
            Instant::now(),
            Utc::now(),
            attrs.metadata().target().to_string(),
            attrs.metadata().name().to_string(),
            visitor.attributes,
            trace_id,
            span_id_str,
        ));
    }

    fn on_record(&self, id: &Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        if IS_TELEMETRY_THREAD.with(|f| *f.borrow()) {
            return;
        }

        let span = ctx.span(id).expect("Span not found");

        let attributes = {
            let mut ext = span.extensions_mut();
            if let Some(data) = ext.get_mut::<(
                Instant,
                DateTime<Utc>,
                String,
                String,
                Vec<(String, String)>,
                String, // trace_id
                String, // span_id
            )>() {
                std::mem::take(&mut data.4)
            } else {
                return;
            }
        };

        let mut visitor = AttributeVisitor { attributes };
        values.record(&mut visitor);

        let mut ext = span.extensions_mut();
        if let Some(data) = ext.get_mut::<(
            Instant,
            DateTime<Utc>,
            String,
            String,
            Vec<(String, String)>,
            String, // trace_id
            String, // span_id
        )>() {
            data.4 = visitor.attributes;
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        if IS_TELEMETRY_THREAD.with(|f| *f.borrow()) {
            return;
        }

        let end_time = Utc::now();
        let end_instant = Instant::now();

        let span = match ctx.span(&id) {
            Some(s) => s,
            None => return,
        };

        let parent_span_id = span.parent().map(|p| p.id().into_u64().to_string());

        let ext = span.extensions();
        if let Some(data) = ext.get::<(
            Instant,
            DateTime<Utc>,
            String,
            String,
            Vec<(String, String)>,
            String,
            String,
        )>() {
            let (start_instant, start_time, target, name, attributes, trace_id, span_id) = data;
            let duration_ms = end_instant.duration_since(*start_instant).as_millis() as u64;

            let final_attrs = attributes.clone();

            // Extract OTel trace ID and span ID if present in the tracing-opentelemetry layer
            let trace_id = trace_id.clone();
            let span_id = span_id.clone();

            // Pass attributes directly, flusher thread will format as JSON
            let attributes = final_attrs;

            let entry = SpanEvent {
                trace_id,
                span_id,
                parent_span_id,
                name: name.clone(),
                target: target.clone(),
                start_time: *start_time,
                end_time,
                duration_ms,
                attributes,
            };

            let _ = self.sender.try_send(entry);
        }
    }
}

#[derive(Default)]
struct AttributeVisitor {
    attributes: Vec<(String, String)>,
}

impl tracing::field::Visit for AttributeVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let key = field.name().to_string();
        let val = format!("{:?}", value);
        // Remove surrounding quotes if it's a plain string
        let val_str = if val.starts_with('"') && val.ends_with('"') {
            val[1..val.len() - 1].to_string()
        } else {
            val
        };
        self.attributes.push((key, val_str));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.attributes
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.attributes
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.attributes
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.attributes
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.attributes
            .push((field.name().to_string(), value.to_string()));
    }
}

/// Start the background flusher thread.
pub fn start_trace_flusher(
    engine: Arc<MVCCEngine>,
    receiver: Receiver<SpanEvent>,
) -> (
    Arc<std::sync::atomic::AtomicBool>,
    std::thread::JoinHandle<()>,
) {
    let shutdown_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag_clone = Arc::clone(&shutdown_flag);

    let handle = thread::Builder::new()
        .name("oxibase-trace-flusher".to_string())
        .spawn(move || {
            // Mark this thread as the telemetry flusher to prevent infinite loops
            IS_TELEMETRY_THREAD.with(|f| *f.borrow_mut() = true);
            crate::common::logging::IS_LOG_FLUSHER.with(|f| *f.borrow_mut() = true);
            crate::common::metrics::IS_METRICS_THREAD.with(|f| *f.borrow_mut() = true);

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
                if let Err(e) = insert_trace_batch(&engine, &entries) {
                    eprintln!("Failed to flush internal traces: {}", e);
                }
            }
        })
        .expect("Failed to spawn trace flusher thread");

    (shutdown_flag, handle)
}

fn insert_trace_batch(engine: &MVCCEngine, entries: &[SpanEvent]) -> crate::core::Result<()> {
    let mut tx = engine.begin_transaction()?;

    // Get the system.traces table
    let mut table = match tx.get_table("system.traces") {
        Ok(t) => t,
        Err(_) => {
            tx.rollback()?;
            return Ok(());
        }
    };

    for entry in entries {
        let trace_id_val = Value::Text(entry.trace_id.clone().into());
        let span_id_val = Value::Text(entry.span_id.clone().into());
        let parent_span_id_val = entry
            .parent_span_id
            .clone()
            .map_or(Value::Null(crate::core::DataType::Text), |id| {
                Value::Text(id.into())
            });
        let name_val = Value::Text(entry.name.clone().into());
        let span_kind_val = Value::Text("INTERNAL".into());
        let start_time_val = Value::Timestamp(entry.start_time);
        let end_time_val = Value::Timestamp(entry.end_time);
        let duration_ms_val = Value::Float(entry.duration_ms as f64);
        let status_code_val = Value::Text("OK".into());
        let status_message_val = Value::Null(crate::core::DataType::Text);
        let attributes_str = if entry.attributes.is_empty() {
            "{}".to_string()
        } else {
            let mut map = serde_json::Map::new();
            for (k, v) in &entry.attributes {
                map.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
        };
        let attributes_val = Value::Text(attributes_str.into());
        let events_val = Value::Null(crate::core::DataType::Text);

        let row = vec![
            Value::null_unknown(), // id AUTO_INCREMENT
            trace_id_val,          // trace_id
            span_id_val,           // span_id
            parent_span_id_val,    // parent_span_id
            name_val,              // name
            span_kind_val,         // span_kind
            start_time_val,        // start_time
            end_time_val,          // end_time
            duration_ms_val,       // duration_ms
            status_code_val,       // status_code
            status_message_val,    // status_message
            attributes_val,        // attributes
            events_val,            // events
        ];

        table.insert(row.into())?;
    }

    tx.commit()?;
    Ok(())
}
