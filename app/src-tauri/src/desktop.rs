//! Desktop behaviour: the tray icon and menu, keeping the window out of the way while a
//! migration runs, start-at-login, and the two notifications FERRY sends.

use ferry_core::Progress;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

const TRAY_ID: &str = "ferry";

/// The tray menu's status line, kept so it can be updated after every session.
struct TrayStatus(MenuItem<Wry>);

/// Builds the tray icon. Its menu shows the migration's progress, and offers Show and Quit.
/// Quitting pauses a migration; it resumes when FERRY next starts.
pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "No migration running", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show FERRY", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit FERRY", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&status, &separator, &show, &quit])?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("FERRY")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    app.manage(TrayStatus(status));
    Ok(())
}

/// Brings the main window back, from the tray, the Dock, or a second launch.
pub fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// One line for the tray menu and tooltip.
pub fn status_line(progress: Option<&Progress>, follow_up_waiting: bool) -> String {
    if follow_up_waiting {
        return "Needs you: sign the follow-up run".into();
    }
    let Some(progress) = progress.filter(|p| p.active) else {
        return "No migration running".into();
    };
    let count = |kind: &str| progress.pieces.iter().filter(|p| p.kind == kind).count();
    let done = |kind: &str| {
        progress
            .pieces
            .iter()
            .filter(|p| p.kind == kind && p.state == "mined")
            .count()
    };
    let (prep_done, preps) = (done("preparation"), count("preparation"));
    if prep_done < preps {
        format!("Preparing: {prep_done} of {preps} done")
    } else {
        format!("{} of {} pieces crossed", done("crossing"), count("crossing"))
    }
}

pub fn set_tray_status(app: &AppHandle, text: &str) {
    if let Some(status) = app.try_state::<TrayStatus>() {
        let _ = status.0.set_text(text);
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(format!("FERRY · {text}")));
    }
}

/// FERRY notifies only when it needs the user or has finished.
pub fn notify(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
}

/// Starts FERRY at login while a migration is running, so a reboot does not stall it, and
/// removes the login item when the migration ends.
///
/// Only in release builds: in development it would register a login item for a debug binary.
pub fn set_start_at_login(app: &AppHandle, on: bool) {
    if cfg!(debug_assertions) {
        log::info!("start at login: would be {} (skipped in debug builds)", if on { "on" } else { "off" });
        return;
    }
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let result = if on { manager.enable() } else { manager.disable() };
    if let Err(e) = result {
        log::warn!("could not change start at login: {e}");
    }
}
