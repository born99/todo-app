use crate::database::Database;
use crate::models::{Priority, Task};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};

pub static ALERT_TASK: OnceLock<Mutex<Option<Task>>> = OnceLock::new();

pub fn get_alert_task_state() -> &'static Mutex<Option<Task>> {
    ALERT_TASK.get_or_init(|| Mutex::new(None))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskPayload {
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub duration_minutes: Option<i32>,
    pub due_date: Option<String>,
    pub alert_early_minutes: Option<i32>,
}

#[tauri::command]
pub fn get_tasks(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Task>, String> {
    db.fetch_tasks()
}

#[tauri::command]
pub fn create_task(
    payload: TaskPayload,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
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
        alert_early_minutes: payload.alert_early_minutes.or(Some(0)),
    };
    db.insert_task(&task).map(|_| ())
}

#[tauri::command]
pub fn complete_task(id: i64, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.mark_task_completed(id)
}

#[tauri::command]
pub fn delete_task(id: i64, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.delete_task_by_id(id)
}

#[tauri::command]
pub fn edit_task(
    id: i64,
    payload: TaskPayload,
    db: tauri::State<'_, Arc<Database>>,
) -> Result<(), String> {
    let due_date = payload
        .due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc));

    db.update_task_by_id(id, &payload, due_date)
}

#[tauri::command]
pub fn close_alert(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("alert") {
        let _ = window.close();
    }
    Ok(())
}

#[tauri::command]
pub fn get_alert_task() -> Result<Option<Task>, String> {
    let lock = get_alert_task_state()
        .lock()
        .map_err(|_| "Poison Error".to_string())?;
    Ok(lock.clone())
}
