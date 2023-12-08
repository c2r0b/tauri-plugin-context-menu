use serde::Deserialize;
use std::sync::Arc;
use tauri::{
    plugin::Builder, plugin::Plugin, plugin::TauriPlugin, Invoke, Manager, Runtime, State, Window,
};

#[cfg(target_os = "linux")]
use std::{sync::{mpsc, Mutex}, time::Duration};

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

    #[cfg(target_os = "linux")]
    fn show_context_menu(
        &self,
        app_context: State<'_, os::AppContext>,
        window: Window<R>,
        pos: Option<Position>,
        items: Option<Vec<MenuItem>>,
    ) {
        let context_menu = Arc::new(self.clone());
        os::show_context_menu(context_menu, app_context, window, pos, items);
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn show_context_menu(
        &self,
        window: Window<R>,
        pos: Option<Position>,
        items: Option<Vec<MenuItem>>,
    ) {
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

#[cfg(target_os = "linux")]
#[tauri::command]
fn show_context_menu<R: Runtime>(
    app_context: State<'_, os::AppContext>,
    manager: State<'_, ContextMenu<R>>,
    window: Window<R>,
    pos: Option<Position>,
    items: Option<Vec<MenuItem>>,
) {
    manager.show_context_menu(app_context, window, pos, items);
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[tauri::command]
fn show_context_menu<R: Runtime>(
    manager: State<'_, ContextMenu<R>>,
    window: Window<R>,
    pos: Option<Position>,
    items: Option<Vec<MenuItem>>,
) {
    manager.show_context_menu(window, pos, items);
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("context_menu")
        .invoke_handler(tauri::generate_handler![show_context_menu])
        .setup(|app| {
            app.manage(ContextMenu::<R>::default());
            Ok(())
        })
        .build()
}

#[cfg(target_os = "linux")]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    use tauri::async_runtime;

    let (tx, rx) = mpsc::channel::<os::GtkThreadCommand>();
    let rx = Arc::new(Mutex::new(rx));

    let rx = rx.clone();
    glib::timeout_add_local(Duration::from_millis(200), move || {
        let rx = rx.lock().unwrap();
        if let Ok(cmd) = rx.try_recv() {
            match cmd {
                os::GtkThreadCommand::ShowContextMenu { pos, items, window } => {
                    async_runtime::block_on(async {
                        os::on_context_menu::<R>(pos, items, window).await;
                    });
                }
            }
        }
        glib::Continue(true)
    });

    Builder::new("context_menu")
        .invoke_handler(tauri::generate_handler![show_context_menu])
        .setup(|app| {
            app.manage(ContextMenu::<R>::default());
            app.manage(os::AppContext {
                tx: Arc::new(Mutex::new(tx)),
            });
            Ok(())
        })
        .build()
}
