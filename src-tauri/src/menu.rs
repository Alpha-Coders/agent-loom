//! Application menu with standard macOS shortcuts

use tauri::{
    menu::{Menu, MenuItemBuilder, PredefinedMenuItem, Submenu},
    AppHandle, Emitter, Manager, Wry,
};

/// Menu item IDs for custom actions
pub const NEW_SKILL_ID: &str = "new-skill";
pub const SAVE_ID: &str = "save";
pub const SYNC_ALL_ID: &str = "sync-all";
pub const REFRESH_ID: &str = "refresh";

/// Update the enabled state of the Save menu item
pub fn set_save_enabled(app: &AppHandle, enabled: bool) {
    if let Some(menu) = app.menu() {
        if let Some(item) = menu.get(SAVE_ID) {
            if let Some(menu_item) = item.as_menuitem() {
                let _ = menu_item.set_enabled(enabled);
            }
        }
    }
}

/// Create the application menu with standard macOS shortcuts
pub fn create_menu(app: &AppHandle) -> Result<Menu<Wry>, tauri::Error> {
    let menu = Menu::new(app)?;

    // App menu (Agent Skills Manager)
    let app_menu = Submenu::new(app, "Agent Skills Manager", true)?;
    app_menu.append(&PredefinedMenuItem::about(app, Some("About Agent Skills Manager"), None)?)?;
    app_menu.append(&PredefinedMenuItem::separator(app)?)?;
    app_menu.append(&PredefinedMenuItem::services(app, None)?)?;
    app_menu.append(&PredefinedMenuItem::separator(app)?)?;
    app_menu.append(&PredefinedMenuItem::hide(app, Some("Hide Agent Skills Manager"))?)?;
    app_menu.append(&PredefinedMenuItem::hide_others(app, None)?)?;
    app_menu.append(&PredefinedMenuItem::show_all(app, None)?)?;
    app_menu.append(&PredefinedMenuItem::separator(app)?)?;
    app_menu.append(&PredefinedMenuItem::quit(app, Some("Quit Agent Skills Manager"))?)?;
    menu.append(&app_menu)?;

    // File menu
    let file_menu = Submenu::new(app, "File", true)?;

    // New Skill (Cmd+N)
    let new_skill = tauri::menu::MenuItemBuilder::with_id(NEW_SKILL_ID, "New Skill")
        .accelerator("CmdOrCtrl+N")
        .build(app)?;
    file_menu.append(&new_skill)?;

    file_menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Save (Cmd+S) - always enabled, handler checks state
    let save = MenuItemBuilder::with_id(SAVE_ID, "Save")
        .accelerator("CmdOrCtrl+S")
        .enabled(true)
        .build(app)?;
    file_menu.append(&save)?;

    // Sync All (Cmd+Shift+S)
    let sync_all = tauri::menu::MenuItemBuilder::with_id(SYNC_ALL_ID, "Sync All")
        .accelerator("CmdOrCtrl+Shift+S")
        .build(app)?;
    file_menu.append(&sync_all)?;

    // Refresh (Cmd+R)
    let refresh = tauri::menu::MenuItemBuilder::with_id(REFRESH_ID, "Refresh Skills")
        .accelerator("CmdOrCtrl+R")
        .build(app)?;
    file_menu.append(&refresh)?;

    file_menu.append(&PredefinedMenuItem::separator(app)?)?;
    file_menu.append(&PredefinedMenuItem::close_window(app, Some("Close Window"))?)?;

    menu.append(&file_menu)?;

    // Edit menu (standard text editing shortcuts)
    let edit_menu = Submenu::new(app, "Edit", true)?;
    edit_menu.append(&PredefinedMenuItem::undo(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::redo(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::separator(app)?)?;
    edit_menu.append(&PredefinedMenuItem::cut(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::copy(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::paste(app, None)?)?;
    edit_menu.append(&PredefinedMenuItem::select_all(app, None)?)?;
    menu.append(&edit_menu)?;

    // Window menu
    let window_menu = Submenu::new(app, "Window", true)?;
    window_menu.append(&PredefinedMenuItem::minimize(app, None)?)?;
    window_menu.append(&PredefinedMenuItem::maximize(app, None)?)?;
    window_menu.append(&PredefinedMenuItem::separator(app)?)?;
    window_menu.append(&PredefinedMenuItem::fullscreen(app, None)?)?;
    menu.append(&window_menu)?;

    Ok(menu)
}

/// Handle menu events
pub fn handle_menu_event(app: &AppHandle, event: &tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        NEW_SKILL_ID => {
            // Emit event to frontend
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("menu-new-skill", ());
            }
        }
        SAVE_ID => {
            // Emit event to frontend
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("menu-save", ());
            }
        }
        SYNC_ALL_ID => {
            // Emit event to frontend
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("menu-sync-all", ());
            }
        }
        REFRESH_ID => {
            // Emit event to frontend
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("menu-refresh", ());
            }
        }
        _ => {}
    }
}
