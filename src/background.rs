use crate::database::Database;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn start_notification_daemon(db: Arc<Database>, app: AppHandle) {
    thread::spawn(move || {
        loop {
            if let Ok(overdue_tasks) = db.fetch_unnotified_overdue_tasks() {
                for task in overdue_tasks {
                    if let Some(existing) = app.get_webview_window("alert") {
                        let _ = existing.emit("set-task", &task);
                        let _ = existing.set_focus();
                    } else {
                        if let Ok(mut lock) = crate::commands::get_alert_task_state().lock() {
                            *lock = Some(task.clone());
                        }
                        match WebviewWindowBuilder::new(
                            &app,
                            "alert",
                            WebviewUrl::App("alert.html".into()),
                        )
                        .title("Overdue Alarm")
                        .transparent(true)
                        .fullscreen(true)
                        .always_on_top(true)
                        .skip_taskbar(true)
                        .decorations(false)
                        .build()
                        {
                            Ok(alert_win) => {
                                let _ = alert_win.set_focus();
                            }
                            Err(e) => {
                                eprintln!("Failed to create alert window: {:?}", e);
                            }
                        }
                    }
                    let _ = db.mark_task_notified(task.id);
                }
            }
            thread::sleep(Duration::from_secs(60));
        }
    });
}
