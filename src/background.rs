use crate::database::Database;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub fn start_notification_daemon(db_path: String, app: AppHandle) {
    thread::spawn(move || {
        let db = Database::new(&db_path);
        loop {
            if let Ok(overdue_tasks) = db.fetch_unnotified_overdue_tasks() {
                for task in overdue_tasks {
                    // Force GUI window to pop up front and center
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }

                    // Blast payload directly to frontend Javascript via Tauri IPC
                    if let Err(e) = app.emit("task-overdue", &task) {
                        eprintln!("Failed to emit event: {}", e);
                    } else {
                        let _ = db.mark_task_notified(task.id);
                    }
                }
            }
            thread::sleep(Duration::from_secs(60));
        }
    });
}
