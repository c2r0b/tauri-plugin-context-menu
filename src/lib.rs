use tauri::{plugin::Plugin, plugin::Builder, plugin::TauriPlugin, State, Window, Manager, Runtime, Invoke};
use std::sync::Arc;

mod window_holder;
mod menu_item;

use menu_item::MenuItem;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod os;

#[derive(serde::Deserialize)]
pub struct Position {
    x: f64,
    y: f64,
}

pub struct ContextMenu<R: Runtime> {
    invoke_handler: Arc<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for ContextMenu<R> {
    fn default() -> Self {
        Self {
            invoke_handler: Arc::new(|_| {}),
        }
    }
}

impl<R: Runtime> Clone for ContextMenu<R> {
    fn clone(&self) -> Self {
        Self {
            invoke_handler: Arc::clone(&self.invoke_handler),
        }
    }
}

impl<R: Runtime> ContextMenu<R> {
    // Method to create a new ContextMenu
    pub fn new<F: 'static + Fn(Invoke<R>) + Send + Sync>(handler: F) -> Self {
        Self {
            invoke_handler: Arc::new(handler),
        }
    }

    fn show_context_menu(&self, window: Window<R>, pos: Option<Position>, items: Option<Vec<MenuItem>>) {
        let context_menu = Arc::new(self.clone());
        os::show_context_menu(context_menu, window, pos, items);
    }
}

impl<R: Runtime> Plugin<R> for ContextMenu<R> {
    fn name(&self) -> &'static str {
        "context_menu"
    }

    fn extend_api(&mut self, invoke: Invoke<R>) {
        (self.invoke_handler)(invoke);
    }
}

#[tauri::command]
fn show_context_menu<R: Runtime>(manager: State<'_, ContextMenu<R>>, window: Window<R>, pos: Option<Position>, items: Option<Vec<MenuItem>>) {
    manager.show_context_menu(window, pos, items);
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("context_menu")
        .invoke_handler(tauri::generate_handler![show_context_menu])
        .setup(|app| {
            app.manage(ContextMenu::<R>::default());
            Ok(())
        })
        .build()
}