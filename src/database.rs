use rusqlite::{Connection, Result};

pub struct Database {
    db_path: String,
}

impl Database {
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
        }
    }

    pub fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    pub fn initialize_schema(&self) -> Result<()> {
        let conn = self.get_connection()?;
        
        // Tasks Table
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
        )?;

        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN is_notified BOOLEAN NOT NULL DEFAULT 0", []);

        // Subtasks Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS subtasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                is_completed BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Tags Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color_hex TEXT NOT NULL
            )",
            [],
        )?;

        // Task_Tags Junction Table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS task_tags (
                task_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (task_id, tag_id),
                FOREIGN KEY(task_id) REFERENCES tasks(id) ON DELETE CASCADE,
                FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(())
    }

    pub fn insert_task(&self, task: &crate::models::Task) -> rusqlite::Result<i64> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO tasks (title, description, priority, duration_minutes, due_date, is_completed, is_notified, recurring_rule, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
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
                task.updated_at
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn fetch_tasks(&self) -> rusqlite::Result<Vec<crate::models::Task>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT id, title, description, priority, duration_minutes, due_date, is_completed, is_notified, recurring_rule, created_at, updated_at FROM tasks")?;
        
        let task_iter = stmt.query_map([], |row| {
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
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn mark_task_completed(&self, task_id: i64) -> rusqlite::Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE tasks SET is_completed = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now(), task_id],
        )?;
        Ok(())
    }

    pub fn fetch_unnotified_overdue_tasks(&self) -> rusqlite::Result<Vec<crate::models::Task>> {
        let conn = self.get_connection()?;
        let now_str = chrono::Utc::now().to_rfc3339();
        
        let mut stmt = conn.prepare("SELECT id, title, description, priority, duration_minutes, due_date, is_completed, is_notified, recurring_rule, created_at, updated_at FROM tasks WHERE is_completed = 0 AND is_notified = 0 AND due_date IS NOT NULL AND due_date <= ?1")?;
        
        let task_iter = stmt.query_map(rusqlite::params![now_str], |row| {
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
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn mark_task_notified(&self, task_id: i64) -> rusqlite::Result<()> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE tasks SET is_notified = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now(), task_id],
        )?;
        Ok(())
    }
}
