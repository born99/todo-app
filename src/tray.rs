use tray_icon::{TrayIconBuilder, TrayIcon, menu::{Menu, MenuItem, MenuEvent}, Icon};
use std::thread;
use std::time::Duration;

fn generate_icon() -> Icon {
    Icon::from_rgba(vec![255, 0, 0, 255], 1, 1).unwrap()
}

pub fn start_tray_daemon() -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append(&quit_i);

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Rust Productivity App")
        .with_title("Productivity")
        .with_icon(generate_icon())
        .build()?;

    let quit_id = quit_i.id().clone();

    thread::spawn(move || {
        loop {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == quit_id {
                    std::process::exit(0);
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
    });

    Ok(tray_icon)
}
