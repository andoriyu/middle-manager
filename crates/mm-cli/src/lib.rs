use anyhow::{Result, anyhow};
use serde_json::{self, Value};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct TaskRow<'a> {
    #[tabled(rename = "Name")]
    name: &'a str,
    #[tabled(rename = "Status")]
    status: &'a str,
    #[tabled(rename = "Priority")]
    priority: &'a str,
    #[tabled(rename = "Due")]
    due: &'a str,
}

pub fn format_tasks_table(tasks: &[serde_json::Value]) -> String {
    let rows: Vec<TaskRow> = tasks
        .iter()
        .map(|t| TaskRow {
            name: t["name"].as_str().unwrap_or(""),
            status: t["properties"]["status"].as_str().unwrap_or(""),
            priority: t["properties"]["priority"].as_str().unwrap_or(""),
            due: t["properties"]["due_date"].as_str().unwrap_or(""),
        })
        .collect();

    Table::new(rows).to_string()
}

pub fn format_task_detail(task: &serde_json::Value) -> String {
    if !task.is_object() {
        return "Task not found".to_string();
    }
    let mut out = String::new();
    out.push_str(&format!("Name: {}\n", task["name"].as_str().unwrap_or("")));
    let labels = task["labels"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("Labels: {}\n", labels));
    out.push_str(&format!(
        "Description: {}\n",
        task["properties"]["description"].as_str().unwrap_or("")
    ));
    out.push_str(&format!(
        "Status: {}\n",
        task["properties"]["status"].as_str().unwrap_or("")
    ));
    out.push_str(&format!(
        "Type: {}\n",
        task["properties"]["task_type"].as_str().unwrap_or("")
    ));
    out.push_str(&format!(
        "Priority: {}\n",
        task["properties"]["priority"].as_str().unwrap_or("")
    ));
    if let Some(due) = task["properties"]["due_date"].as_str() {
        out.push_str(&format!("Due: {}\n", due));
    }
    if let Some(created) = task["properties"]["created_at"].as_str() {
        out.push_str(&format!("Created: {}\n", created));
    }
    if let Some(updated) = task["properties"]["updated_at"].as_str() {
        out.push_str(&format!("Updated: {}\n", updated));
    }
    if let Some(obs) = task["observations"].as_array() {
        if !obs.is_empty() {
            out.push_str("Observations:\n");
            for o in obs {
                if let Some(s) = o.as_str() {
                    out.push_str(&format!("  - {}\n", s));
                }
            }
        }
    }
    out
}

pub fn parse_json_result(result: &rust_mcp_sdk::schema::CallToolResult) -> Result<Value> {
    let text = result
        .content
        .first()
        .ok_or_else(|| anyhow!("No content"))?
        .as_text_content()
        .map_err(|e| anyhow!(e.to_string()))?
        .text
        .clone();
    Ok(serde_json::from_str(&text)?)
}
