mod background;
mod database;
mod models;
mod tray;
mod ui;

fn main() {
    println!("Personal Productivity App - Core Initialized");

    #[cfg(target_os = "linux")]
    if let Err(e) = gtk::init() {
        eprintln!("Failed to run gtk::init: {}", e);
    }

    // Verify DB layout
    let db = database::Database::new("tasks.db");
    if let Err(e) = db.initialize_schema() {
        eprintln!("Failed to initialize database: {}", e);
    } else {
        println!("Database schema initialized successfully.");
    }

    // Start daemon threads
    background::start_notification_daemon("tasks.db".to_string());
    let _tray_icon = tray::start_tray_daemon().expect("Failed to initialize system tray");

    println!("Running in background. Check System Tray or wait for notifications.");

    if let Err(e) = ui::launch_ui() {
        eprintln!("UI failed to launch: {}", e);
    }
}
