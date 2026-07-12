//! Background job processing.
//!
//! Long-running or deferred tasks are dispatched to background workers.
//! Uses `PostgreSQL` as the job queue for durability, with `tokio::spawn`
//! for in-process execution. Jobs are retried with exponential backoff.
//!
//! Ownership: Infrastructure Team
//! Dependencies: services, database, events
//! Public API: `JobRunner`, Job, `enqueue_job`, `process_jobs`

pub mod deployment_jobs;
pub mod notification_jobs;
pub mod runner;
pub mod types;
