use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Job {
    #[must_use]
    pub fn priority_level(&self) -> JobPriority {
        JobPriority::from_i32(self.priority)
    }

    #[must_use]
    pub fn status_type(&self) -> Option<JobStatus> {
        self.status.parse().ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl JobPriority {
    #[must_use]
    pub fn from_i32(n: i32) -> Self {
        match n {
            0 => Self::Low,
            1 => Self::Normal,
            2 => Self::High,
            3 => Self::Critical,
            _ if n < 0 => Self::Low,
            _ => Self::Critical,
        }
    }

    #[must_use]
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
    Cancelled,
    DeadLetter,
}

impl FromStr for JobStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "retrying" => Ok(Self::Retrying),
            "cancelled" => Ok(Self::Cancelled),
            "dead_letter" => Ok(Self::DeadLetter),
            _ => Err(()),
        }
    }
}

impl JobStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
            Self::Cancelled => "cancelled",
            Self::DeadLetter => "dead_letter",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeadLetterJob {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub last_error: Option<String>,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
    pub moved_to_dlq_at: DateTime<Utc>,
}
