use serde::Deserialize;
use tauri::{plugin::Builder, plugin::TauriPlugin, Runtime, Window};

mod menu_item;

use menu_item::MenuItem;

mod keymap;

#[cfg(target_os = "windows")]
mod win_image_handler;

#[cfg(target_os = "windows")]
#[path = "win.rs"]
mod os;

#[cfg(target_os = "macos")]
mod macos_window_holder;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod os;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod os;

#[derive(Clone, Deserialize)]
pub struct Position {
    x: f64,
    y: f64,
    is_absolute: Option<bool>,
}

#[tauri::command]
fn show_context_menu<R: Runtime>(
    window: Window<R>,
    pos: Option<Position>,
    items: Option<Vec<MenuItem>>,
) {
    os::show_context_menu(window, pos, items);
}
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("context_menu")
        .invoke_handler(tauri::generate_handler![show_context_menu])
        .build()
}
