use chrono::{DateTime, Utc};
use mm_memory::MemoryValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};

/// Priority level for a task
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Status of a task
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum TaskStatus {
    Todo,
    #[strum(serialize = "inprogress", serialize = "in_progress")]
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

/// Type of task
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
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

impl Default for TaskProperties {
    fn default() -> Self {
        TaskProperties {
            description: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            due_date: None,
            task_type: TaskType::Feature,
            status: TaskStatus::Todo,
            priority: Priority::Medium,
        }
    }
}

impl From<HashMap<String, MemoryValue>> for TaskProperties {
    fn from(mut map: HashMap<String, MemoryValue>) -> Self {
        let description = match map.remove("description") {
            Some(MemoryValue::String(s)) => s,
            Some(v) => v.to_string(),
            None => String::new(),
        };

        let created_at = match map.remove("created_at") {
            Some(MemoryValue::DateTime(dt)) => dt.with_timezone(&Utc),
            Some(MemoryValue::String(s)) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            _ => Utc::now(),
        };

        let updated_at = match map.remove("updated_at") {
            Some(MemoryValue::DateTime(dt)) => dt.with_timezone(&Utc),
            Some(MemoryValue::String(s)) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            _ => Utc::now(),
        };

        let due_date = match map.remove("due_date") {
            Some(MemoryValue::DateTime(dt)) => Some(dt.with_timezone(&Utc)),
            Some(MemoryValue::String(s)) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok(),
            _ => None,
        };

        let task_type = match map.remove("task_type") {
            Some(MemoryValue::String(s)) => TaskType::from_str(&s).unwrap_or(TaskType::Feature),
            _ => TaskType::Feature,
        };

        let status = match map.remove("status") {
            Some(MemoryValue::String(s)) => TaskStatus::from_str(&s).unwrap_or(TaskStatus::Todo),
            _ => TaskStatus::Todo,
        };

        let priority = match map.remove("priority") {
            Some(MemoryValue::String(s)) => {
                Priority::from_str(&s).unwrap_or(TaskProperties::default().priority)
            }
            _ => TaskProperties::default().priority,
        };

        TaskProperties {
            description,
            created_at,
            updated_at,
            due_date,
            task_type,
            status,
            priority,
        }
    }
}

impl From<TaskProperties> for HashMap<String, MemoryValue> {
    fn from(props: TaskProperties) -> Self {
        let mut map = HashMap::new();
        map.insert(
            "description".to_string(),
            MemoryValue::String(props.description),
        );
        map.insert(
            "created_at".to_string(),
            MemoryValue::DateTime(props.created_at.into()),
        );
        map.insert(
            "updated_at".to_string(),
            MemoryValue::DateTime(props.updated_at.into()),
        );
        if let Some(due) = props.due_date {
            map.insert("due_date".to_string(), MemoryValue::DateTime(due.into()));
        }
        map.insert(
            "task_type".to_string(),
            MemoryValue::String(props.task_type.as_ref().to_string()),
        );
        map.insert(
            "status".to_string(),
            MemoryValue::String(props.status.as_ref().to_string()),
        );
        map.insert(
            "priority".to_string(),
            MemoryValue::String(props.priority.as_ref().to_string()),
        );
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_from_str() {
        assert_eq!(Priority::from_str("high").unwrap(), Priority::High);
        assert_eq!(
            TaskStatus::from_str("inprogress").unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!(TaskType::from_str("bug").unwrap(), TaskType::Bug);
        assert!(Priority::from_str("unknown").is_err());
    }

    #[test]
    fn test_task_properties_from_map() {
        let mut map = HashMap::new();
        map.insert("task_type".to_string(), MemoryValue::String("bug".into()));
        map.insert("status".to_string(), MemoryValue::String("done".into()));
        map.insert(
            "priority".to_string(),
            MemoryValue::String("critical".into()),
        );

        let props = TaskProperties::from(map);
        assert_eq!(props.task_type, TaskType::Bug);
        assert_eq!(props.status, TaskStatus::Done);
        assert_eq!(props.priority, Priority::Critical);
    }
}
