use crate::database::Database;
use notify_rust::Notification;
use std::thread;
use std::time::Duration;

pub fn start_notification_daemon(db_path: String) {
    thread::spawn(move || {
        let db = Database::new(&db_path);
        loop {
            if let Ok(overdue_tasks) = db.fetch_unnotified_overdue_tasks() {
                for task in overdue_tasks {
                    let body = task.description.clone().unwrap_or_else(|| "Task Reminder".to_string());
                    if let Err(e) = Notification::new()
                        .summary(&format!("Overdue: {}", task.title))
                        .body(&body)
                        .appname("Rust Productivity App")
                        .timeout(notify_rust::Timeout::Milliseconds(5000))
                        .show()
                    {
                        eprintln!("Failed to show notification: {}", e);
                    } else {
                        let _ = db.mark_task_notified(task.id);
                    }
                }
            }
            thread::sleep(Duration::from_secs(60));
        }
    });
}
