use crate::database::Database;
use crate::models::{Priority, Task};
use chrono::Utc;
use serde::{Deserialize, Serialize};

fn get_db() -> Database {
    Database::new("tasks.db")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskPayload {
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub duration_minutes: Option<i32>,
    pub due_date: Option<String>,
}

#[tauri::command]
pub fn get_tasks() -> Result<Vec<Task>, String> {
    get_db().fetch_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_task(payload: TaskPayload) -> Result<(), String> {
    let db = get_db();
    let due_date = payload
        .due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    let task = Task {
        id: 0,
        title: payload.title,
        description: payload.description,
        priority: payload.priority.parse().unwrap_or_default(),
        duration_minutes: payload.duration_minutes,
        due_date,
        is_completed: false,
        is_notified: false,
        recurring_rule: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    db.insert_task(&task).map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_task(id: i64) -> Result<(), String> {
    get_db().mark_task_completed(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(id: i64) -> Result<(), String> {
    let db = get_db();
    let conn = db.get_connection().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])
        .map(|_| ())
        .map_err(|e| e.to_string())
}
