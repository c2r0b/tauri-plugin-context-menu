use cocoa::appkit::{NSControl, NSMenuItem};
use cocoa::base::{id, nil, selector};
use cocoa::foundation::{NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel, NO, YES};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Arc;
use tauri::{Runtime, Window};

use crate::keymap::{get_key_map, get_modifier_map};
use crate::macos_window_holder::CURRENT_WINDOW;
use crate::{MenuItem, Position};

extern "C" fn menu_item_action<R: Runtime>(_self: &Object, _cmd: Sel, _item: id) {
    // Get the window from the CURRENT_WINDOW static
    let window_arc: Arc<tauri::Window<R>> = match CURRENT_WINDOW.get_window() {
        Some(window_arc) => window_arc,
        None => return println!("No window found"),
    };

    // Get the event name and payload from the representedObject of the NSMenuItem
    let nsstring_obj: id = unsafe { msg_send![_item, representedObject] };
    let combined_str: String = unsafe {
        let cstr: *const std::os::raw::c_char = msg_send![nsstring_obj, UTF8String];
        std::ffi::CStr::from_ptr(cstr)
            .to_string_lossy()
            .into_owned()
    };
    let parts: Vec<&str> = combined_str.split(":::").collect();
    let event_name = parts.get(0).unwrap_or(&"").to_string();
    let payload = parts.get(1).cloned();

    // Dereferencing the Arc to get a reference to the Window<R>
    let window = &*window_arc;

    // Emit the event on the window
    window.emit(&event_name, payload).unwrap();
}

extern "C" fn menu_did_close<R: Runtime>(_self: &Object, _cmd: Sel, _menu: id) {
    if let Some(window) = CURRENT_WINDOW.get_window::<R>() {
        window.emit("menu-did-close", ()).unwrap();
    } else {
        println!("Menu did close, but no window was found.");
    }
}

fn register_menu_item_action<R: Runtime>() -> Sel {
    let selector_name = "menuAction:";

    let exists: bool;
    let class = objc::runtime::Class::get("MenuItemDelegate");
    exists = class.is_some();

    if !exists {
        let superclass = objc::runtime::Class::get("NSObject").unwrap();
        let mut decl = ClassDecl::new("MenuItemDelegate", superclass).unwrap();

        unsafe {
            decl.add_method(
                selector(selector_name),
                menu_item_action::<R> as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                selector("menuDidClose:"),
                menu_did_close::<R> as extern "C" fn(&Object, Sel, id),
            );
            decl.register();
        }
    }

    selector(selector_name)
}

fn create_custom_menu_item<R: Runtime>(option: &MenuItem) -> id {
    // If the item is a separator, return a separator item
    if option.is_separator.unwrap_or(false) {
        let separator: id = unsafe { msg_send![class!(NSMenuItem), separatorItem] };
        return separator;
    }

    let sel = register_menu_item_action::<R>();
    let menu_item: id = unsafe {
        let title = match &option.label {
            Some(label) => NSString::alloc(nil).init_str(label),
            None => NSString::alloc(nil).init_str(""),
        };

        // Parse the shortcut
        let (key, mask) = match &option.shortcut {
            Some(shortcut) => {
                let parts: Vec<&str> = shortcut.split('+').collect();

                let key_map = get_key_map();
                let modifier_map = get_modifier_map();

                // Default values
                let mut key_str = "";
                let mut mask = cocoa::appkit::NSEventModifierFlags::empty();

                for part in parts.iter() {
                    if let Some(k) = key_map.get(*part) {
                        key_str = k;
                    } else if let Some(m) = modifier_map.get(*part) {
                        mask.insert(*m);
                    } else {
                        key_str = *part; // Assuming the last item or the only item without a '+' is the main key.
                    }
                }

                (NSString::alloc(nil).init_str(key_str), mask)
            }
            None => (
                NSString::alloc(nil).init_str(""),
                cocoa::appkit::NSEventModifierFlags::empty(),
            ),
        };

        let item = cocoa::appkit::NSMenuItem::alloc(nil)
            .initWithTitle_action_keyEquivalent_(title, sel, key);
        item.setKeyEquivalentModifierMask_(mask);

        // Set the enabled state (disabled flag is optional)
        item.setEnabled_(match option.disabled {
            Some(true) => NO,
            _ => YES,
        });

        // Set the represented object as the event name and payload
        let string_payload = match &option.payload {
            Some(payload) => format!(
                "{}:::{}",
                &option.event.as_ref().unwrap_or(&"".to_string()),
                payload
            ),
            None => option.event.as_ref().unwrap_or(&"".to_string()).clone(),
        };
        let ns_string_payload = NSString::alloc(nil).init_str(&string_payload);
        let _: () = msg_send![item, setRepresentedObject:ns_string_payload];

        // Set the icon if it exists
        if let Some(icon) = &option.icon {
            let ns_string_path: id = NSString::alloc(nil).init_str(&icon.path);
            let image: *mut Object = msg_send![class!(NSImage), alloc];
            let image: *mut Object = msg_send![image, initWithContentsOfFile:ns_string_path];
            if image.is_null() {
                println!("Failed to load image from path: {}", icon.path);
            } else {
                let width = icon.width.unwrap_or(16);
                let height = icon.height.unwrap_or(16);
                let size = NSSize::new(width as f64, height as f64);
                let _: () = msg_send![image, setSize:size];

                let _: () = msg_send![item, setImage:image];
            }
        }

        // Set the delegate
        let delegate_class_name = "MenuItemDelegate";
        let delegate_class: &'static objc::runtime::Class =
            objc::runtime::Class::get(delegate_class_name).expect("Class should exist");
        let delegate_instance: id = msg_send![delegate_class, new];
        item.setTarget_(delegate_instance);

        // Set the submenu if it exists
        if let Some(subitems) = &option.subitems {
            let submenu: id = msg_send![class!(NSMenu), new];
            let _: () = msg_send![submenu, setAutoenablesItems:NO];
            for subitem in subitems.iter() {
                let sub_menu_item: id = create_custom_menu_item::<R>(subitem);
                let _: () = msg_send![submenu, addItem:sub_menu_item];
            }
            let _: () = msg_send![item, setSubmenu:submenu];
        }

        // Handle checkable menu items
        let state = match option.checked {
            Some(true) => 1,
            _ => 0,
        };
        let _: () = msg_send![item, setState:state];

        item
    };

    menu_item
}

fn create_context_menu<R: Runtime>(options: &[MenuItem], window: &Window<R>) -> id {
    let _: () = CURRENT_WINDOW.set_window(window.clone());
    unsafe {
        let title = NSString::alloc(nil).init_str("Menu");
        let menu: id = msg_send![class!(NSMenu), alloc];
        let menu: id = msg_send![menu, initWithTitle: title];

        let _: () = msg_send![menu, setAutoenablesItems:NO];

        for option in options.iter().cloned() {
            let item: id = create_custom_menu_item::<R>(&option);
            let _: () = msg_send![menu, addItem:item];
        }

        let delegate_class_name = "MenuItemDelegate";
        let delegate_class: &'static objc::runtime::Class =
            objc::runtime::Class::get(delegate_class_name).expect("Class should exist");
        let delegate_instance: id = msg_send![delegate_class, new];
        let _: () = msg_send![menu, setDelegate:delegate_instance];

        menu
    }
}

pub fn show_context_menu<R: Runtime>(
    window: Window<R>,
    pos: Option<Position>,
    items: Option<Vec<MenuItem>>,
) {
    let main_queue = dispatch::Queue::main();
    main_queue.exec_async(move || {
        let items_slice = items.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        let menu = create_context_menu(items_slice, &window);
        let location = match pos {
            // Convert web page coordinates to screen coordinates
            Some(pos) if pos.x != 0.0 || pos.y != 0.0 => unsafe {
                let window_position = window.outer_position().unwrap();
                let screen: id = msg_send![class!(NSScreen), mainScreen];
                let frame: NSRect = msg_send![screen, frame];
                let screen_height = frame.size.height;
                let scale_factor = match window.scale_factor() {
                    Ok(factor) => factor,
                    Err(_) => 1.0, // Use a default value if getting the scale factor fails
                };
                if pos.is_absolute.unwrap_or(false) {
                    let x = pos.x;
                    let y = screen_height - pos.y;
                    NSPoint::new(x, y)
                } else {
                    let x = pos.x + (window_position.x as f64 / scale_factor);
                    let y = screen_height - (window_position.y as f64 / scale_factor) - pos.y;
                    NSPoint::new(x, y)
                }
            },
            // Get the current mouse location if the web page didn't specify a position
            _ => unsafe {
                let event: NSPoint = msg_send![class!(NSEvent), mouseLocation];
                NSPoint::new(event.x, event.y)
            },
        };
        unsafe {
            let _: () =
                msg_send![menu, popUpMenuPositioningItem:nil atLocation:location inView:nil];
        }
    });
}
