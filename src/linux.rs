use tauri::{Window, Runtime};
use std::sync::Arc;
use std::any::Any;
use gtk::{Menu, MenuItem as GtkMenuItem, prelude::*};
use gtk::gdk::Display;
use gtk::traits::WidgetExt;
use lazy_static::lazy_static;
use tauri::async_runtime::Mutex;

use crate::{ ContextMenu, MenuItem, Position };

use glib::Sender;

#[derive(Clone)]
enum GtkContextMenuCommand{
    ShowContextMenu {
        items: Option<Vec<MenuItem>>,
        pos: Option<Position>,
        window: Arc<Mutex<Box<dyn Any + Send>>>
    }
}

lazy_static! {
    // This will be your communication channel to the main GTK thread.
    static ref GTK_SENDER: Sender<GtkContextMenuCommand> = {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        // Attach the receiver to the main context (GTK thread).
        let main_context = glib::MainContext::default();
        let _ = main_context.acquire();
        receiver.attach(Some(&main_context), move |cmd| {
            handle_gtk_command::<tauri::Wry>(cmd);
            glib::Continue(true)
        });

        sender
    };
}

fn handle_gtk_command<R:Runtime>(cmd: GtkContextMenuCommand) {
    println!("Show context menu 1");
    glib::idle_add(move || {
        let cmd_clone = cmd.clone();
        async_std::task::block_on(handle_command::<R>(&cmd_clone));
        glib::Continue(false) // Run once and don't repeat
    });
}

async fn handle_command<R:Runtime>(cmd: &GtkContextMenuCommand) {
    println!("Show context menu 3");
    match cmd {
        GtkContextMenuCommand::ShowContextMenu { items, pos, window } => {
            println!("Show context menu 4");
            let window_mutex = window.lock().await;
            if let Some(window) = window_mutex.downcast_ref::<Window<R>>() {
                // Now you have your window of type Window<R> here
                let menu: Menu = Menu::new();
                let gtk_window = window.gtk_window().unwrap();

                if let Some(menu_items) = items.clone() {
                    for item in menu_items.iter() {
                        append_menu_item(&menu, item);
                    }
                }

                let (mut x, mut y) = match pos {
                    Some(ref position) => (position.x as i32, position.y as i32),
                    None => {
                        if let Some(display) = Display::default() {
                            if let Some(seat) = display.default_seat() {
                                let pointer = seat.pointer();
                                let (_screen, x, y) = match pointer {
                                    Some(p) => p.position(),
                                    None => {
                                        eprintln!("Failed to get pointer position");
                                        (display.default_screen(), 0, 0)
                                    }
                                };
                                (x, y)
                            } else {
                                eprintln!("Failed to get default seat");
                                (0, 0)
                            }
                        } else {
                            eprintln!("Failed to get default display");
                            (0, 0)
                        }
                    }
                };

                let is_absolute = if let Some(position) = pos.clone() {
                    position.is_absolute
                } else {
                    Some(false)
                };
                if is_absolute.unwrap_or(true) == true {
                    // Adjust x and y if the coordinates are not relative to the window
                    let window_position = window.outer_position().unwrap();
                    x -= window_position.x;
                    y -= window_position.y;
                }

                // Get the gdk window from the gtk window
                let gdk_window = gtk_window.window().unwrap();
                
                // The values 3 and gtk::CURRENT_TIME can be replaced if you have specific values to provide.
                println!("Is realized: {}", gtk_window.is_realized());
                println!("Is mapped: {}", gtk_window.is_mapped());
                println!("Is visible: {}", gdk_window.is_visible());
                println!("Menu is sensitive: {}", menu.get_sensitive());
                println!("Menu is visible: {}", menu.get_visible());

                // Create rectangle based on x and y
                //let rect = &gdk::Rectangle::new(x, y, 0, 0);
                let rect = &gdk::Rectangle::new(100, 100, 0, 0);  // for testing purposes

                // Emit the menu-did-close event to JavaScript on menu close
                let window_clone = window.clone();
                menu.connect_hide(move |_| {
                    window_clone.emit("menu-did-close", ()).unwrap();
                });
                
                menu.show_all();
                menu.popup_at_rect(&gdk_window, &rect, gdk::Gravity::NorthWest, gdk::Gravity::NorthWest, None);
            }
        }
    }
}


pub fn show_context_menu<R: Runtime>(_context_menu: Arc<ContextMenu<R>>, window: Window<R>, pos: Option<Position>, items: Option<Vec<MenuItem>>) {
    let _ =GTK_SENDER.send(GtkContextMenuCommand::ShowContextMenu {
        items,
        pos,
        window: Arc::new(Mutex::new(Box::new(window) as Box<dyn Any + Send>))
    });
}

fn append_menu_item(menu: &Menu, item: &MenuItem) {
    if item.is_separator.unwrap_or(false) {
        menu.append(&gtk::SeparatorMenuItem::new());
    } else {
        let label = item.label.as_deref().unwrap_or("");
        let menu_item = GtkMenuItem::with_label(&label);

        if item.disabled.unwrap_or(false) {
            menu_item.set_sensitive(false);
        }

        // If an event is provided, you can connect to the "activate" signal
        if let Some(event) = &item.event {
            menu_item.connect_activate(move |_| {
                // Handle the event here
            });
        }

        if let Some(subitems) = &item.subitems {
            let submenu = Menu::new();
            for subitem in subitems.iter() {
                append_menu_item(&submenu, subitem);
            }
            menu_item.set_submenu(Some(&submenu));
        }

        menu.append(&menu_item);
    }
}
