use chrono::Utc;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &str) -> Self {
        let c = Connection::open(db_path).expect("Failed to initialize SQLite");
        c.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
        Self {
            conn: Mutex::new(c),
        }
    }

    pub fn initialize_schema(&self) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                priority TEXT NOT NULL,
                duration_minutes INTEGER,
                due_date TEXT,
                is_completed BOOLEAN NOT NULL DEFAULT 0,
                is_notified BOOLEAN NOT NULL DEFAULT 0,
                recurring_rule TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        let _ = conn.execute(
            "ALTER TABLE tasks ADD COLUMN is_notified BOOLEAN NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE tasks ADD COLUMN alert_early_minutes INTEGER DEFAULT 0",
            [],
        );

        conn.execute(
            "CREATE TABLE IF NOT EXISTS subtasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                is_completed BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color_hex TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS task_tags (
                task_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (task_id, tag_id),
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn insert_task(&self, task: &crate::models::Task) -> Result<i64, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;
        conn.execute(
            "INSERT INTO tasks (title, description, priority, duration_minutes, due_date, is_completed, is_notified, recurring_rule, created_at, updated_at, alert_early_minutes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                task.title,
                task.description,
                task.priority.as_str(),
                task.duration_minutes,
                task.due_date,
                task.is_completed,
                task.is_notified,
                task.recurring_rule,
                task.created_at,
                task.updated_at,
                task.alert_early_minutes
            ],
        ).map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn fetch_tasks(&self) -> Result<Vec<crate::models::Task>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;
        let mut stmt = conn.prepare("SELECT id, title, description, priority, duration_minutes, due_date, is_completed, is_notified, recurring_rule, created_at, updated_at, alert_early_minutes FROM tasks").map_err(|e| e.to_string())?;

        let task_iter = stmt
            .query_map([], |row| {
                let priority_str: String = row.get(3)?;
                let priority = priority_str.parse().unwrap_or_default();

                Ok(crate::models::Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    priority,
                    duration_minutes: row.get(4)?,
                    due_date: row.get(5)?,
                    is_completed: row.get(6)?,
                    is_notified: row.get(7)?,
                    recurring_rule: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    alert_early_minutes: row.get(11).unwrap_or(Some(0)),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task.map_err(|e| e.to_string())?);
        }
        Ok(tasks)
    }

    pub fn mark_task_completed(&self, task_id: i64) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;
        conn.execute(
            "UPDATE tasks SET is_completed = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![Utc::now(), task_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn fetch_unnotified_overdue_tasks(&self) -> Result<Vec<crate::models::Task>, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;
        let mut stmt = conn.prepare("SELECT id, title, description, priority, duration_minutes, due_date, is_completed, is_notified, recurring_rule, created_at, updated_at, alert_early_minutes FROM tasks WHERE is_completed = 0 AND is_notified = 0 AND due_date IS NOT NULL").map_err(|e| e.to_string())?;

        let task_iter = stmt
            .query_map([], |row| {
                let priority_str: String = row.get(3)?;
                let priority = priority_str.parse().unwrap_or_default();

                Ok(crate::models::Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    priority,
                    duration_minutes: row.get(4)?,
                    due_date: row.get(5)?,
                    is_completed: row.get(6)?,
                    is_notified: row.get(7)?,
                    recurring_rule: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    alert_early_minutes: row.get(11).unwrap_or(Some(0)),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut tasks = Vec::new();
        let now = Utc::now();
        for task in task_iter {
            let t = task.map_err(|e| e.to_string())?;
            println!("Checking task: {:?}", t.title);
            if let Some(due) = t.due_date {
                let early = t.alert_early_minutes.unwrap_or(0);
                let trigger_time = due - chrono::Duration::minutes(early as i64);
                println!("Trigger time: {:?}, Now: {:?}", trigger_time, now);
                if trigger_time <= now {
                    println!("Task {} is overdue!", t.id);
                    tasks.push(t);
                }
            }
        }
        Ok(tasks)
    }

    pub fn mark_task_notified(&self, task_id: i64) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;
        conn.execute(
            "UPDATE tasks SET is_notified = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![Utc::now(), task_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_task_by_id(&self, id: i64) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;
        conn.execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_task_by_id(
        &self,
        id: i64,
        payload: &crate::commands::TaskPayload,
        due_date: Option<chrono::DateTime<Utc>>,
    ) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "DB Poison Error".to_string())?;
        conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, priority = ?3, duration_minutes = ?4, due_date = ?5, is_notified = 0, updated_at = ?6, alert_early_minutes = ?7 WHERE id = ?8",
            rusqlite::params![
                payload.title,
                payload.description,
                payload.priority,
                payload.duration_minutes,
                due_date,
                Utc::now(),
                payload.alert_early_minutes.unwrap_or(0),
                id
            ]
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
