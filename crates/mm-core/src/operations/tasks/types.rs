use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Priority level for a task
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Status of a task
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

/// Type of task
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum TaskType {
    Feature,
    Bug,
    Chore,
    Improvement,
}

/// Properties for Task entities
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct TaskProperties {
    /// Short description of the task
    pub description: String,

    /// When the task was created
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,

    /// When the task was last updated
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,

    /// When the task is due
    #[schemars(with = "Option<String>")]
    pub due_date: Option<DateTime<Utc>>,

    /// Task type
    pub task_type: TaskType,

    /// Task status
    pub status: TaskStatus,

    /// Task priority
    pub priority: Priority,
}
