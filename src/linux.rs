use gdk::{Display, ModifierType, keys::Key};
use tauri::{Window, Runtime, State};
use std::{mem, sync::{Arc, Mutex, mpsc::Sender}, any::Any};
use gtk::{Menu, MenuItem as GtkMenuItem, prelude::*, traits::WidgetExt, AccelGroup, AccelFlags};
use glib::clone;

use crate::{ ContextMenu, MenuItem, Position };
use crate::keymap::get_key_map;

pub struct AppContext {
    pub tx: Arc<Mutex<Sender<GtkThreadCommand>>>
}

pub enum GtkThreadCommand {
    ShowContextMenu {
        pos: Option<Position>,
        items: Option<Vec<MenuItem>>,
        window: Arc<Mutex<Box<dyn Any + Send>>>
    }
}

pub async fn on_context_menu<R:Runtime>(pos:Option<Position>, items:Option<Vec<MenuItem>>, window:Arc<Mutex<Box<dyn Any + Send>>>) {
    let window_mutex = window.lock().unwrap();
    if let Some(window) = window_mutex.downcast_ref::<Window<R>>() {
        // Create and show the context menu
        // Create a new menu.
        let menu = Menu::new();
        let gtk_window = window.gtk_window().unwrap();

        if let Some(menu_items) = items.clone() {
            for item in menu_items.iter() {
                append_menu_item(window, &gtk_window, &menu, item);
            }
        }

        let keep_alive = menu.clone();
        let window_clone = window.clone();
        menu.connect_hide(clone!(@weak keep_alive => move |_| {
            window_clone.emit("menu-did-close", ()).unwrap();
            drop(keep_alive);
        }));

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

        // Show the context menu at the specified position.
        let gdk_window = gtk_window.window().unwrap();
        let rect = &gdk::Rectangle::new(x, y, 0, 0);
        menu.popup_at_rect(&gdk_window, &rect, gdk::Gravity::NorthWest, gdk::Gravity::NorthWest, None);
    }
}

pub fn show_context_menu<R: Runtime>(_context_menu: Arc<ContextMenu<R>>, app_context: State<'_, AppContext>, window: Window<R>, pos: Option<Position>, items: Option<Vec<MenuItem>>) {

    let tx = app_context.tx.lock().unwrap(); // Lock the mutex to access the sender
    tx.send(GtkThreadCommand::ShowContextMenu {
        pos: pos,
        items: items,
        window: Arc::new(Mutex::new(Box::new(window) as Box<dyn Any + Send>)),
    }).expect("Failed to send command to GTK thread");
}

fn key_to_u32(key: gdk::keys::Key) -> u32 {
    unsafe { mem::transmute(key) }
}

fn append_menu_item<R: Runtime>(window: &Window<R>, gtk_window: &gtk::ApplicationWindow, menu: &Menu, item: &MenuItem) {
    if item.is_separator.unwrap_or(false) {
        menu.append(&gtk::SeparatorMenuItem::builder().visible(true).build());
    } else {
        let label = item.label.as_deref().unwrap_or("");
        let menu_item = GtkMenuItem::with_label(&label);

        // Handle enabled/disabled state
        if item.disabled.unwrap_or(false) {
            menu_item.set_sensitive(false);
        }

        // If an event is provided, you can connect to the "activate" signal (from item.event and item.payload)
        if let Some(event) = &item.event {
            if let Some(payload) = &item.payload {
                let window_clone = window.clone();

                // get event from String to str
                let event_clone = event.clone();
                let payload_clone = payload.clone();
                menu_item.connect_activate(move |_| {
                    window_clone.emit(event_clone.as_str(), &payload_clone).unwrap(); // Emit the event to JavaScript
                });
            }
        }

        // Handle shortcut
        if let Some(shortcut) = &item.shortcut {
            let accel_group = AccelGroup::new();
            gtk_window.add_accel_group(&accel_group);

            // Parse and assign the shortcut
            let (key, mods) = parse_shortcut(shortcut);
            let key_u32 = key_to_u32(key);
            menu_item.add_accelerator("activate", &accel_group, key_u32, mods, AccelFlags::VISIBLE);
        }

        if let Some(subitems) = &item.subitems {
            let submenu = Menu::new();
            for subitem in subitems.iter() {
                append_menu_item(window, &gtk_window, &submenu, subitem);
            }
            menu_item.set_submenu(Some(&submenu));
        }

        menu.append(&menu_item);
        menu_item.show();
    }
}

fn parse_shortcut(shortcut: &str) -> (Key, ModifierType) {
    let key_map = get_key_map();
    let parts: Vec<&str> = shortcut.split('+').collect();

    let mut key: Key = 0.into();
    let mut mods = ModifierType::empty();

    for part in parts.iter() {
        if let Some(k) = key_map.get(*part) {
            key = k.clone();
        }
    }

    (key, mods)
}