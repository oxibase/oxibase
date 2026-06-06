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

//! OpenTelemetry Metrics Ingestion System
//!
//! Captures internal metrics from tracing events and persists them into the `system.metrics` table.

use chrono::{DateTime, Utc};
use crossbeam_channel::{Receiver, Sender};
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use crate::core::Value;
use crate::storage::mvcc::engine::MVCCEngine;
use crate::storage::traits::Engine;

thread_local! {
    /// Flag to prevent the metrics flusher thread from tracing its own database operations,
    /// which would cause an infinite loop.
    pub static IS_METRICS_THREAD: RefCell<bool> = const { RefCell::new(false) };
}

/// Represents a captured metric event.
#[derive(Debug, Clone)]
pub struct MetricEvent {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub metric_type: String,
    pub value: f64,
    pub attributes: String, // Stored as JSON string
    pub timestamp: DateTime<Utc>,
}

/// Custom tracing layer that pushes metric events into a crossbeam channel.
/// It looks for tracing events with `metric_name` and `metric_type`.
pub struct SystemMetricsLayer {
    sender: Sender<MetricEvent>,
}

impl SystemMetricsLayer {
    pub fn new(sender: Sender<MetricEvent>) -> Self {
        Self { sender }
    }
}

impl<S> Layer<S> for SystemMetricsLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if IS_METRICS_THREAD.with(|f| *f.borrow()) {
            return;
        }

        let mut visitor = MetricVisitor::default();
        event.record(&mut visitor);

        if let (Some(name), Some(m_type), Some(val)) = (
            visitor.metric_name.clone(),
            visitor.metric_type.clone(),
            visitor.value,
        ) {
            let attributes_str =
                serde_json::to_string(&visitor.attributes).unwrap_or_else(|_| "{}".to_string());

            let metric_event = MetricEvent {
                name,
                description: visitor.description,
                unit: visitor.unit,
                metric_type: m_type,
                value: val,
                attributes: attributes_str,
                timestamp: Utc::now(),
            };

            let _ = self.sender.try_send(metric_event);
        }
    }
}

#[derive(Default)]
struct MetricVisitor {
    metric_name: Option<String>,
    metric_type: Option<String>,
    value: Option<f64>,
    unit: Option<String>,
    description: Option<String>,
    attributes: serde_json::Map<String, serde_json::Value>,
}

impl tracing::field::Visit for MetricVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let key = field.name();
        let val = format!("{:?}", value);

        // Remove surrounding quotes if it's a plain string
        let val_str = if val.starts_with('"') && val.ends_with('"') {
            val[1..val.len() - 1].to_string()
        } else {
            val
        };

        match key {
            "metric_name" => self.metric_name = Some(val_str),
            "metric_type" => self.metric_type = Some(val_str),
            "unit" => self.unit = Some(val_str),
            "description" => self.description = Some(val_str),
            _ => {
                // Ignore standard tracing keys if we don't want them in attributes
                if key != "message" && key != "target" {
                    self.attributes
                        .insert(key.to_string(), serde_json::Value::String(val_str));
                }
            }
        }
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if field.name() == "value" {
            self.value = Some(value);
        } else if let Some(n) = serde_json::Number::from_f64(value) {
            self.attributes
                .insert(field.name().to_string(), serde_json::Value::Number(n));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if field.name() == "value" {
            self.value = Some(value as f64);
        } else {
            self.attributes.insert(
                field.name().to_string(),
                serde_json::Value::Number(serde_json::Number::from(value)),
            );
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "value" {
            self.value = Some(value as f64);
        } else {
            self.attributes.insert(
                field.name().to_string(),
                serde_json::Value::Number(serde_json::Number::from(value)),
            );
        }
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.attributes
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "metric_name" => self.metric_name = Some(value.to_string()),
            "metric_type" => self.metric_type = Some(value.to_string()),
            "unit" => self.unit = Some(value.to_string()),
            "description" => self.description = Some(value.to_string()),
            _ => {
                if field.name() != "message" && field.name() != "target" {
                    self.attributes.insert(
                        field.name().to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
                }
            }
        }
    }
}

/// Start the background flusher thread.
pub fn start_metrics_flusher(
    engine: Arc<MVCCEngine>,
    receiver: Receiver<MetricEvent>,
) -> Arc<std::sync::atomic::AtomicBool> {
    let shutdown_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag_clone = Arc::clone(&shutdown_flag);

    thread::Builder::new()
        .name("oxibase-metrics-flusher".to_string())
        .spawn(move || {
            // Mark this thread as the metrics flusher to prevent infinite loops
            IS_METRICS_THREAD.with(|f| *f.borrow_mut() = true);

            let batch_size = 100;
            let timeout = Duration::from_secs(1);

            let mut last_pool_check = std::time::Instant::now();
            let mut last_small_stats = crate::common::buffer_pool::global::small().stats();
            let mut last_medium_stats = crate::common::buffer_pool::global::medium().stats();
            let mut last_large_stats = crate::common::buffer_pool::global::large().stats();

            loop {
                if flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let mut entries = Vec::new();

                // Wait for the first message with a timeout
                if let Ok(entry) = receiver.recv_timeout(timeout) {
                    entries.push(entry);
                    while entries.len() < batch_size {
                        match receiver.try_recv() {
                            Ok(entry) => entries.push(entry),
                            Err(_) => break,
                        }
                    }
                }

                if last_pool_check.elapsed() >= Duration::from_secs(1) {
                    last_pool_check = std::time::Instant::now();

                    let current_small = crate::common::buffer_pool::global::small().stats();
                    let current_medium = crate::common::buffer_pool::global::medium().stats();
                    let current_large = crate::common::buffer_pool::global::large().stats();

                    let pools = [
                        ("small", &current_small, &mut last_small_stats),
                        ("medium", &current_medium, &mut last_medium_stats),
                        ("large", &current_large, &mut last_large_stats),
                    ];

                    for (pool_name, current_stats, last_stats) in pools {
                        let gets_delta =
                            current_stats.get_count.saturating_sub(last_stats.get_count);
                        let created_delta = current_stats
                            .buffers_created
                            .saturating_sub(last_stats.buffers_created);

                        let misses = created_delta;
                        let hits = gets_delta.saturating_sub(created_delta);

                        if hits > 0 {
                            entries.push(MetricEvent {
                                name: "buffer_pool_hits".to_string(),
                                description: Some(format!("Hits for {} buffer pool", pool_name)),
                                unit: Some("count".to_string()),
                                metric_type: "counter".to_string(),
                                value: hits as f64,
                                attributes: format!(r#"{{"pool": "{}"}}"#, pool_name),
                                timestamp: Utc::now(),
                            });
                        }

                        if misses > 0 {
                            entries.push(MetricEvent {
                                name: "buffer_pool_misses".to_string(),
                                description: Some(format!("Misses for {} buffer pool", pool_name)),
                                unit: Some("count".to_string()),
                                metric_type: "counter".to_string(),
                                value: misses as f64,
                                attributes: format!(r#"{{"pool": "{}"}}"#, pool_name),
                                timestamp: Utc::now(),
                            });
                        }

                        *last_stats = current_stats.clone();
                    }
                }

                if entries.is_empty() {
                    continue;
                }

                // Insert into database
                if let Err(e) = insert_metric_batch(&engine, &entries) {
                    eprintln!("Failed to flush internal metrics: {}", e);
                }
            }
        })
        .expect("Failed to spawn metrics flusher thread");

    shutdown_flag
}

fn insert_metric_batch(engine: &MVCCEngine, entries: &[MetricEvent]) -> crate::core::Result<()> {
    let mut tx = engine.begin_transaction()?;

    // Get the system.metrics table
    let mut table = match tx.get_table("system.metrics") {
        Ok(t) => t,
        Err(_) => {
            tx.rollback()?;
            return Ok(());
        }
    };

    for entry in entries {
        let name_val = Value::Text(entry.name.clone().into());
        let desc_val = entry
            .description
            .clone()
            .map_or(Value::Null(crate::core::DataType::Text), |d| {
                Value::Text(d.into())
            });
        let unit_val = entry
            .unit
            .clone()
            .map_or(Value::Null(crate::core::DataType::Text), |u| {
                Value::Text(u.into())
            });
        let type_val = Value::Text(entry.metric_type.clone().into());
        let val_val = Value::Float(entry.value);
        let attr_val = Value::Text(entry.attributes.clone().into());
        let ts_val = Value::Timestamp(entry.timestamp);

        let row = vec![
            Value::null_unknown(), // id AUTO_INCREMENT
            name_val,              // name
            desc_val,              // description
            unit_val,              // unit
            type_val,              // metric_type
            val_val,               // value
            attr_val,              // attributes
            ts_val,                // timestamp
        ];

        table.insert(row.into())?;
    }

    tx.commit()?;
    Ok(())
}
