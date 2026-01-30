//! System tray module for Talent application
//!
//! Provides system tray functionality with quick access to common actions:
//! - Show/Hide main window
//! - Sync All skills
//! - Quit application

use tauri::{
    include_image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

/// Embedded tray icon as a template image (22x22 monochrome with alpha)
/// macOS will automatically use @2x variant and handle dark/light mode
const TRAY_ICON: tauri::image::Image<'static> = include_image!("icons/trayIconTemplate.png");

/// Menu item IDs for tray context menu
pub mod ids {
    pub const SHOW: &str = "show";
    pub const SYNC_ALL: &str = "sync_all";
    pub const QUIT: &str = "quit";
}

/// Creates and configures the system tray for the application
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // Create menu items
    let show_item = MenuItem::with_id(app, ids::SHOW, "Show Talent", true, None::<&str>)?;
    let sync_item = MenuItem::with_id(app, ids::SYNC_ALL, "Sync All", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, ids::QUIT, "Quit", true, None::<&str>)?;

    // Build the menu
    let menu = Menu::with_items(app, &[&show_item, &sync_item, &quit_item])?;

    // Build and configure the tray icon with embedded template image
    let _tray = TrayIconBuilder::new()
        .icon(TRAY_ICON.clone())
        .icon_as_template(true) // macOS: treat as template for auto dark/light mode
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Talent - Agent Skills Manager")
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(handle_tray_event)
        .build(app)?;

    Ok(())
}

/// Handles clicks on tray menu items
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        ids::SHOW => {
            show_main_window(app);
        }
        ids::SYNC_ALL => {
            // Trigger sync via the app state
            if let Some(window) = app.get_webview_window("main") {
                // Emit event to frontend to trigger sync
                let _ = window.emit("tray-sync-all", ());
                show_main_window(app);
            }
        }
        ids::QUIT => {
            app.exit(0);
        }
        _ => {}
    }
}

/// Handles tray icon events (clicks, hover, etc.)
fn handle_tray_event<R: Runtime>(tray: &tauri::tray::TrayIcon<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            // Left click toggles window visibility
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    show_main_window(&app);
                }
            }
        }
        _ => {}
    }
}

/// Shows and focuses the main window
fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
