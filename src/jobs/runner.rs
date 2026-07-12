//! Background job runner.
//!
//! Polls the `job_queue` table for pending jobs, claims them atomically,
//! and executes the associated handler. Failed jobs are retried up to
//! `MAX_RETRIES` before being marked as failed.
