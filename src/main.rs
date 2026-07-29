#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod background;
mod commands;
mod database;
mod models;
mod tray;

use std::sync::Arc;

fn main() {
    println!("Personal Productivity App - Core Initialized");

    let db = Arc::new(database::Database::new("tasks.db"));
    if let Err(e) = db.initialize_schema() {
        eprintln!("Failed to initialize database: {}", e);
    } else {
        println!("Database schema initialized successfully.");
    }

    println!("Running in background. Starting Tauri window...");
    let db_setup = Arc::clone(&db);

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        .manage(db)
        .setup(move |app| {
            // Forcefully enable OS auto-start on boot
            use tauri_plugin_autostart::ManagerExt;
            let _ = app.autolaunch().enable();

            background::start_notification_daemon(db_setup, app.handle().clone());
            use tauri::Manager;
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Rust Productivity App")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|_app, event| match event.id.as_ref() {
                    "quit" => std::process::exit(0),
                    "show" => {
                        if let Some(window) = _app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::create_task,
            commands::complete_task,
            commands::delete_task,
            commands::edit_task,
            commands::close_alert,
            commands::get_alert_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
