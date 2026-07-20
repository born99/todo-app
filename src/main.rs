mod database;
mod models;

fn main() {
    println!("Personal Productivity App - Core Initialized");

    let db = database::Database::new("tasks.db");
    if let Err(e) = db.initialize_schema() {
        eprintln!("Failed to initialize database: {}", e);
    } else {
        println!("Database schema initialized successfully.");
    }
}
