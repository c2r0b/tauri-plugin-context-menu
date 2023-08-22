use tauri::{Window, Runtime};
use std::sync::Arc;
use cocoa::appkit::{NSMenuItem, NSControl};
use cocoa::base::{id, nil, selector};
use cocoa::foundation::{NSPoint, NSString, NSRect};
use objc::{msg_send, sel, sel_impl, class};
use objc::runtime::{Sel, Object, YES, NO};
use objc::declare::ClassDecl;

use crate::{ ContextMenu, MenuItem, Position };
use crate::macos_window_holder::{CURRENT_WINDOW};

extern "C" fn menu_item_action<R: Runtime>(_self: &Object, _cmd: Sel, _item: id) {
    // Get the window from the CURRENT_WINDOW static
    let window_arc: Arc<tauri::Window<R>> = match CURRENT_WINDOW.get_window() {
        Some(window_arc) => window_arc,
        None => return println!("No window found"),
    };

    // Get the event name from the representedObject of the NSMenuItem
    let nsstring_obj: id = unsafe { msg_send![_item, representedObject] };
    let event_name: String = unsafe {
        let cstr: *const std::os::raw::c_char = msg_send![nsstring_obj, UTF8String];
        std::ffi::CStr::from_ptr(cstr).to_string_lossy().into_owned()
    };

    // Dereferencing the Arc to get a reference to the Window<R>
    let window = &*window_arc;

    // Emit the event on the window
    window.emit(&event_name, ()).unwrap();
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
            decl.add_method(selector(selector_name), menu_item_action::<R> as extern "C" fn(&Object, Sel, id));
            decl.add_method(selector("menuDidClose:"), menu_did_close::<R> as extern "C" fn(&Object, Sel, id));
            decl.register();
        }
    }

    selector(selector_name)
}

fn create_custom_menu_item<R: Runtime>(context_menu: &ContextMenu<R>, option: &MenuItem) -> id {
    // If the item is a separator, return a separator item
    if option.is_separator.unwrap_or(false) {
        let separator: id = unsafe {
            msg_send![class!(NSMenuItem), separatorItem]
        };
        return separator
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

                // Default values
                let mut key = "";
                let mut mask = cocoa::appkit::NSEventModifierFlags::empty();

                for part in parts.iter() {
                    match *part {
                        "cmd" => mask.insert(cocoa::appkit::NSEventModifierFlags::NSCommandKeyMask),
                        "shift" => mask.insert(cocoa::appkit::NSEventModifierFlags::NSShiftKeyMask),
                        "alt" => mask.insert(cocoa::appkit::NSEventModifierFlags::NSAlternateKeyMask),
                        "ctrl" => mask.insert(cocoa::appkit::NSEventModifierFlags::NSControlKeyMask),
                        // ... other modifier keys ...
                        _ => key = *part,  // Assuming the last item or the only item without a '+' is the main key.
                    }
                }
                
                (NSString::alloc(nil).init_str(key), mask)
            }
            None => (NSString::alloc(nil).init_str(""), cocoa::appkit::NSEventModifierFlags::empty()),
        };
        
        let item = cocoa::appkit::NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(title, sel, key);
        item.setKeyEquivalentModifierMask_(mask);
        
        // Set the enabled state (disabled flag is optional)
        item.setEnabled_(match option.disabled {
            Some(true) => NO,
            _ => YES,
        });
        
        // Set the represented object to the event name
        let string = match &option.event {
            Some(event_name) => NSString::alloc(nil).init_str(event_name),
            None => NSString::alloc(nil).init_str(""),
        };
        let _: () = msg_send![item, setRepresentedObject:string];
            
        // Set the icon if it exists
        if let Some(icon_path) = &option.icon_path {
            let ns_string_path: id = NSString::alloc(nil).init_str(icon_path);
            let image: *mut Object = msg_send![class!(NSImage), alloc];
            let image: *mut Object = msg_send![image, initWithContentsOfFile:ns_string_path];
            if image.is_null() {
                println!("Failed to load image from path: {}", icon_path);
            } else {
                let _: () = msg_send![item, setImage:image];
            }
        }        

        // Set the delegate
        let delegate_class_name = "MenuItemDelegate";
        let delegate_class: &'static objc::runtime::Class = objc::runtime::Class::get(delegate_class_name).expect("Class should exist");
        let delegate_instance: id = msg_send![delegate_class, new];
        item.setTarget_(delegate_instance);

        // Set the submenu if it exists
        if let Some(subitems) = &option.subitems {
            let submenu: id = msg_send![class!(NSMenu), new];
            let _: () = msg_send![submenu, setAutoenablesItems:NO];
            for subitem in subitems.iter() {
                let sub_menu_item: id = create_custom_menu_item(&context_menu, subitem);
                let _: () = msg_send![submenu, addItem:sub_menu_item];
            }
            let _: () = msg_send![item, setSubmenu:submenu];
        }

        item
    };
    menu_item
}

fn create_context_menu<R: Runtime>(context_menu: &ContextMenu<R>, options: &[MenuItem], window: &Window<R>) -> id {
    let _: () = CURRENT_WINDOW.set_window(window.clone());
    unsafe {
        let title = NSString::alloc(nil).init_str("Menu");
        let menu: id = msg_send![class!(NSMenu), alloc];
        let menu: id = msg_send![menu, initWithTitle: title];

        let _: () = msg_send![menu, setAutoenablesItems:NO];
        
        for option in options.iter().cloned() {
            let item: id = create_custom_menu_item(&context_menu, &option);
            let _: () = msg_send![menu, addItem:item];
        }

        let delegate_class_name = "MenuItemDelegate";
        let delegate_class: &'static objc::runtime::Class = objc::runtime::Class::get(delegate_class_name).expect("Class should exist");
        let delegate_instance: id = msg_send![delegate_class, new];
        let _: () = msg_send![menu, setDelegate:delegate_instance];

        menu
    }
}

pub fn show_context_menu<R: Runtime>(context_menu: Arc<ContextMenu<R>>, window: Window<R>, pos: Option<Position>, items: Option<Vec<MenuItem>>) {
    let main_queue = dispatch::Queue::main();
    main_queue.exec_async(move || {
        let items_slice = items.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        let menu = create_context_menu(&*context_menu, items_slice, &window);
        let location = match pos {
            // Convert web page coordinates to screen coordinates
            Some(pos) if pos.x != 0.0 || pos.y != 0.0 => unsafe {
                let window_position = window.outer_position().unwrap();
                let screen: id = msg_send![class!(NSScreen), mainScreen];
                let frame: NSRect = msg_send![screen, frame];
                let screen_height = frame.size.height;
                let scale_factor = match window.scale_factor() {
                    Ok(factor) => factor,
                    Err(_) => 1.0,  // Use a default value if getting the scale factor fails
                };
                let x = pos.x + (window_position.x as f64 / scale_factor);
                let y = screen_height - (window_position.y as f64 / scale_factor) - pos.y;
                NSPoint::new(x, y)
            }
            // Get the current mouse location if the web page didn't specify a position
            _ => unsafe {
                let event: NSPoint = msg_send![class!(NSEvent), mouseLocation];
                NSPoint::new(event.x, event.y)
            },
        };
        unsafe {
            let _: () = msg_send![menu, popUpMenuPositioningItem:nil atLocation:location inView:nil];
        }
    });
}