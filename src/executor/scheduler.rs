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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use cron::Schedule;

use crate::core::{DataType, Row, Value};
use crate::executor::Executor;
use crate::storage::jobs::{SYS_CRON, SYS_CRON_RUNS};
use crate::storage::traits::Engine;

/// Background worker for scheduling jobs using CRON expressions
pub struct JobScheduler {
    executor: Executor,
    shutdown_flag: Arc<AtomicBool>,
}

impl JobScheduler {
    /// Start a new scheduler worker
    pub fn start(executor: Executor) -> Arc<AtomicBool> {
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&shutdown_flag);

        let scheduler = Self {
            executor,
            shutdown_flag: flag_clone,
        };

        thread::Builder::new()
            .name("oxibase-job-scheduler".to_string())
            .spawn(move || {
                scheduler.run_loop();
            })
            .expect("Failed to spawn job scheduler thread");

        shutdown_flag
    }

    fn run_loop(&self) {
        tracing::info!("Job scheduler background thread started");

        // Loop runs once every second
        let interval = Duration::from_secs(1);

        while !self.shutdown_flag.load(Ordering::Relaxed) {
            let start = SystemTime::now();

            if let Err(e) = self.evaluate_and_run_jobs() {
                tracing::error!("Job scheduler error: {}", e);
            }

            // Sleep until the next second starts
            if let Ok(elapsed) = start.elapsed() {
                if elapsed < interval {
                    thread::sleep(interval - elapsed);
                }
            } else {
                thread::sleep(interval);
            }
        }

        tracing::info!("Job scheduler background thread shutting down");
    }

    fn evaluate_and_run_jobs(&self) -> crate::core::Result<()> {
        let tx = self.executor.engine.begin_transaction()?;

        // Ensure system.cron exists before proceeding
        if !self.executor.engine.table_exists(SYS_CRON)? {
            return Ok(());
        }

        let table = tx.get_table(SYS_CRON)?;
        let mut scanner = table.scan(&[], None)?;

        let mut jobs_to_run = Vec::new();
        let now = Utc::now();

        while scanner.next() {
            let row = scanner.row();

            // Schema: id, name, schedule, command, active
            if let (
                Some(Value::Integer(id)),
                Some(Value::Text(name)),
                Some(Value::Text(schedule_str)),
                Some(Value::Text(command)),
                Some(Value::Boolean(active)),
            ) = (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4))
            {
                if !active {
                    continue;
                }

                if let Ok(schedule) = schedule_str.as_ref().parse::<Schedule>() {
                    // Find the most recent past time this should have run
                    // Since we check every second, we see if it was scheduled to run precisely now or in the last second
                    // cron::Schedule calculates future iterators easily, so we check if now is included
                    if schedule.includes(now) {
                        jobs_to_run.push((*id, name.to_string(), command.to_string()));
                    }
                }
            }
        }
        drop(scanner);
        drop(tx); // Release the read transaction

        for (job_id, name, command) in jobs_to_run {
            self.execute_job(job_id, &name, &command);
        }

        Ok(())
    }

    fn execute_job(&self, job_id: i64, name: &str, command: &str) {
        let _span = tracing::info_span!("job.execute", job_id = job_id, job_name = name).entered();
        tracing::debug!("Executing scheduled job '{}' (ID: {})", name, job_id);

        let start_time = Utc::now();

        // Execute the command in an internal context
        let result = self.executor.execute_internal_sql(command);

        let end_time = Utc::now();

        let (status, message) = match result {
            Ok(_) => ("SUCCESS", None),
            Err(e) => ("FAILED", Some(e.to_string())),
        };

        // Log the run
        if let Err(e) = self.log_run(job_id, status, message, start_time, end_time) {
            tracing::error!("Failed to log job execution for '{}': {}", name, e);
        }
    }

    fn log_run(
        &self,
        job_id: i64,
        status: &str,
        message: Option<String>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> crate::core::Result<()> {
        let mut tx = self.executor.engine.begin_transaction()?;
        let mut table = tx.get_table(SYS_CRON_RUNS)?;

        let values = vec![
            Value::Null(DataType::Integer), // ID
            Value::Integer(job_id),
            Value::text(status),
            if let Some(msg) = message {
                Value::text(msg)
            } else {
                Value::Null(DataType::Text)
            },
            Value::Timestamp(start_time),
            Value::Timestamp(end_time),
        ];

        let row = Row::from_values(values);
        table.insert(row)?;
        tx.commit()?;

        Ok(())
    }
}
