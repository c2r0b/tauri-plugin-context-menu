use gdk::{keys::Key, Display, ModifierType};
use gtk::{prelude::*, traits::WidgetExt, AccelFlags, AccelGroup, Menu, MenuItem as GtkMenuItem};
use std::{mem, thread::sleep, time};
use tauri::{Runtime, Window};

use crate::keymap::{get_key_map, get_mod_map};
use crate::{MenuItem, Position};

pub fn on_context_menu<R: Runtime>(
    pos: Option<Position>,
    items: Option<Vec<MenuItem>>,
    window: Window<R>,
) {
    // Create and show the context menu
    let gtk_window = window.gtk_window().unwrap();

    // Check if the window is realized
    if !gtk_window.is_realized() {
        gtk_window.realize();
    }

    // Create a new menu.
    let menu = Menu::new();
    if let Some(menu_items) = items {
        for item in menu_items.iter() {
            append_menu_item(&window, &gtk_window, &menu, item);
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
    if is_absolute.unwrap_or(true) {
        // Adjust x and y if the coordinates are not relative to the window
        let window_position = window.outer_position().unwrap();
        x -= window_position.x;
        y -= window_position.y;
    }

    // Required otherwise the menu doesn't show properly
    sleep(time::Duration::from_millis(100));

    // Delay the display of the context menu to ensure the window is ready
    glib::idle_add_local(move || {
        // Show the context menu at the specified position.
        let gdk_window = gtk_window.window().unwrap();
        let rect = &gdk::Rectangle::new(x, y, 0, 0);
        let mut event = gdk::Event::new(gdk::EventType::ButtonPress);
        event.set_device(
            gdk_window
                .display()
                .default_seat()
                .and_then(|d| d.pointer())
                .as_ref(),
        );
        menu.show_all();
        menu.popup_at_rect(
            &gdk_window,
            rect,
            gdk::Gravity::NorthWest,
            gdk::Gravity::NorthWest,
            Some(&event),
        );
        Continue(false)
    });
}

pub fn show_context_menu<R: Runtime>(
    window: Window<R>,
    pos: Option<Position>,
    items: Option<Vec<MenuItem>>,
) {
    on_context_menu(pos, items, window);
}

fn append_menu_item<R: Runtime>(
    window: &Window<R>,
    gtk_window: &gtk::ApplicationWindow,
    menu: &Menu,
    item: &MenuItem,
) {
    if item.is_separator.unwrap_or(false) {
        menu.append(&gtk::SeparatorMenuItem::builder().visible(true).build());
    } else {
        let menu_item = match item.checked {
            Some(state) => {
                // Create a CheckMenuItem for checkable items
                let check_menu_item = gtk::CheckMenuItem::new();
                check_menu_item.set_active(state);
                check_menu_item.upcast()
            }
            None => {
                // Create a regular MenuItem for non-checkable items
                gtk::MenuItem::new()
            }
        };

        // Create a Box to hold the image and label
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.set_homogeneous(false);

        // Handle icon
        if let Some(icon) = &item.icon {
            let image = gtk::Image::from_file(&icon.path);
            if let Some(width) = icon.width {
                if let Some(height) = icon.height {
                    image.set_pixel_size(width as i32);
                    image.set_pixel_size(height as i32);
                }
            }
            hbox.pack_start(&image, false, false, 0);
        }

        // Add label to the Box
        let label = item.label.as_deref().unwrap_or("");
        let accel_label = gtk::AccelLabel::new(label);
        accel_label.set_xalign(0.0); // Align the label to the left
        hbox.pack_start(&accel_label, true, true, 0);

        // Add the Box to the MenuItem
        menu_item.add(&hbox);

        // Handle enabled/disabled state
        if item.disabled.unwrap_or(false) {
            menu_item.set_sensitive(false);
        }

        // If an event is provided, you can connect to the "activate" signal (from item.event and item.payload)
        if let Some(event) = &item.event {
            let window_clone = window.clone();

            // payload may exist
            let payload_clone = item.payload.clone();

            // get event from String to str
            let event_clone = event.clone();
            menu_item.connect_activate(move |_| {
                window_clone
                    .emit(event_clone.as_str(), &payload_clone)
                    .unwrap(); // Emit the event to JavaScript
            });
        }

        // Handle shortcut
        if let Some(shortcut) = &item.shortcut {
            let accel_group = AccelGroup::new();
            gtk_window.add_accel_group(&accel_group);

            // Parse and assign the shortcut
            let (key, mods) = parse_shortcut(shortcut);
            accel_label.set_accel_widget(Some(&menu_item));
            menu_item.add_accelerator("activate", &accel_group, key, mods, AccelFlags::VISIBLE);
        }

        if let Some(subitems) = &item.subitems {
            let submenu = Menu::new();
            for subitem in subitems.iter() {
                append_menu_item(window, gtk_window, &submenu, subitem);
            }
            menu_item.set_submenu(Some(&submenu));
        }

        menu.append(&menu_item);
    }
}

fn key_to_u32(key: gdk::keys::Key) -> u32 {
    unsafe { mem::transmute(key) }
}

fn parse_shortcut(shortcut: &str) -> (u32, ModifierType) {
    let key_map = get_key_map();
    let mod_map = get_mod_map(); // This should map strings like "ctrl" to ModifierType
    let parts: Vec<&str> = shortcut.split('+').collect();

    // Assuming last part is always the key
    let key_str = parts.last().unwrap_or(&"");

    // Get the key from the key map
    let key = if let Some(key) = key_map.get(key_str) {
        // Clone the key value to get ownership of it
        key.clone()
    } else {
        // If the key is not in the map, assume it's a character
        Key::from_name(key_str)
    };

    let key_u32 = key_to_u32(key);

    let mut mods = ModifierType::empty();

    // Process all parts except the last one as modifiers
    for &mod_str in &parts[..parts.len() - 1] {
        if let Some(&mod_type) = mod_map.get(mod_str) {
            mods.insert(mod_type);
        }
    }

    (key_u32, mods)
}
