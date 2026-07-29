use crate::database::Database;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn start_notification_daemon(db_path: String, app: AppHandle) {
    thread::spawn(move || {
        let db = Database::new(&db_path);
        loop {
            if let Ok(overdue_tasks) = db.fetch_unnotified_overdue_tasks() {
                for task in overdue_tasks {
                    // Generate seamless fullscreen transparent overlay window natively
                    if let Some(existing) = app.get_webview_window("alert") {
                        let _ = existing.close();
                    }

                    if let Ok(alert_win) = WebviewWindowBuilder::new(
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
                        // Bridge task data directly into the newly booted transparent HTML frame
                        let _ = alert_win.emit("set-task", &task);
                        let _ = alert_win.set_focus();
                    }

                    let _ = db.mark_task_notified(task.id);
                }
            }
            thread::sleep(Duration::from_secs(60));
        }
    });
}
