use mm_cli::format_task_detail;
use mm_cli::format_tasks_table;
use serde_json::json;

#[test]
fn test_format_tasks_table() {
    let tasks = vec![json!({
        "name": "task:1",
        "properties": {"status": "todo", "priority": "medium", "due_date": "2025-07-01"}
    })];
    let output = format_tasks_table(&tasks);
    assert!(output.contains("task:1"));
    assert!(output.contains("todo"));
    assert!(output.contains("medium"));
}

#[test]
fn test_format_task_detail() {
    let task = json!({
        "name": "task:1",
        "labels": ["Memory", "Task"],
        "observations": ["First"],
        "properties": {
            "description": "Test",
            "status": "todo",
            "task_type": "feature",
            "priority": "low",
            "created_at": "2025-06-26T00:00:00Z",
            "updated_at": "2025-06-26T00:00:00Z"
        }
    });
    let out = format_task_detail(&task);
    assert!(out.contains("task:1"));
    assert!(out.contains("Memory, Task"));
    assert!(out.contains("Test"));
}
