use tauri::{plugin::Plugin, plugin::Builder, plugin::TauriPlugin, State, Window, Manager, Runtime, Invoke};

use std::sync::{Arc, Mutex};
use std::any::Any;

#[cfg(target_os = "linux")]
use gtk::{Menu, MenuItem, prelude::*};

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSMenuItem, NSControl};
use cocoa::base::{id, nil, selector};
use cocoa::foundation::{NSPoint, NSString, NSRect};
use objc::{msg_send, sel, sel_impl, class};
use objc::runtime::{Sel, Object, YES, NO};
use objc::declare::ClassDecl;

#[cfg(target_os = "windows")]
use native_windows_gui as nwg;

#[derive(serde::Deserialize)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Clone, serde::Deserialize)]
struct MenuItem {
    label: Option<String>,
    disabled: Option<bool>,
    shortcut: Option<String>,
    event: Option<String>,
    subitems: Option<Vec<MenuItem>>,
    icon_path: Option<String>,
    is_separator: Option<bool>,
}

#[cfg(target_os = "linux")]
fn create_context_menu() -> Menu {
    let menu = Menu::new();
    let item = MenuItem::new_with_label("Menu Item");
    menu.append(&item);
    menu.show_all();
    menu
}

#[cfg(target_os = "windows")]
fn create_context_menu() -> nwg::Menu {
    use nwg::NativeUi;

    let mut menu = nwg::Menu::default();
    let mut menu_item = nwg::MenuItem::default();

    nwg::Menu::builder()
        .parent(None)
        .popup(true)
        .build(&mut menu)
        .expect("Failed to build context menu");

    nwg::MenuItem::builder()
        .parent(&menu)
        .text("Menu Item")
        .build(&mut menu_item)
        .expect("Failed to build menu item");

    menu
}

pub struct ContextMenu<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for ContextMenu<R> {
    fn default() -> Self {
        Self {
            invoke_handler: Box::new(|_| {}),
        }
    }
}

impl<R: Runtime> Drop for ContextMenu<R> {
    fn drop(&mut self) {
        println!("ContextMenu is being deallocated!");
    }
}

pub struct WindowHolder {
    window: Arc<Mutex<Option<Arc<dyn Any + Send + Sync>>>>,
}

impl WindowHolder {
    pub fn new() -> Self {
        Self {
            window: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_window<R: Runtime>(&self, window: Window<R>) {
        let mut lock = self.window.lock().unwrap();
        *lock = Some(Arc::new(window));
    }

    pub fn get_window<R: Runtime>(&self) -> Option<Arc<Window<R>>> {
        let lock = self.window.lock().unwrap();
        match &*lock {
            Some(window) => Some(window.clone().downcast::<Window<R>>().unwrap()),
            None => None,
        }
    }

    pub fn clear_window(&self) {
        let mut lock = self.window.lock().unwrap();
        *lock = None;
    }
}

lazy_static::lazy_static! {
    static ref CURRENT_WINDOW: WindowHolder = WindowHolder::new();
}

impl<R: Runtime> ContextMenu<R> 
{
    fn set_window(window: Window<R>) {
        CURRENT_WINDOW.set_window(window);
    }
    
    fn get_window() -> Option<Arc<Window<R>>> {
        CURRENT_WINDOW.get_window()
    }

    #[cfg(target_os = "macos")]
    extern "C" fn menu_item_action(_self: &Object, _cmd: Sel, _item: id) {
        // Get the window from the CURRENT_WINDOW static
        let window_arc: Arc<tauri::Window<R>> = match Self::get_window() {
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
    
    #[cfg(target_os = "macos")]
    extern "C" fn menu_did_close(_self: &Object, _cmd: Sel, _menu: id) {
        if let Some(window) = Self::get_window() {
            window.emit("menu-did-close", ()).unwrap();
        } else {
            println!("Menu did close, but no window was found.");
        }
    }

    #[cfg(target_os = "macos")]
    fn register_menu_item_action() -> Sel {
        let selector_name = "menuAction:";
    
        let exists: bool;
        let class = objc::runtime::Class::get("MenuItemDelegate");
        exists = class.is_some();
    
        if !exists {
            let superclass = objc::runtime::Class::get("NSObject").unwrap();
            let mut decl = ClassDecl::new("MenuItemDelegate", superclass).unwrap();
    
            unsafe {
                decl.add_method(selector(selector_name), Self::menu_item_action as extern "C" fn(&Object, Sel, id));
                decl.add_method(selector("menuDidClose:"), Self::menu_did_close as extern "C" fn(&Object, Sel, id));
                decl.register();
            }
        }
    
        selector(selector_name)
    }

    #[cfg(target_os = "macos")]
    fn create_custom_menu_item(option: &MenuItem) -> id {
        // If the item is a separator, return a separator item
        if option.is_separator.unwrap_or(false) {
            let separator: id = unsafe {
                msg_send![class!(NSMenuItem), separatorItem]
            };
            return separator
        }

        let sel = Self::register_menu_item_action();
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
                let image: id = msg_send![class!(NSImage), imageNamed:ns_string_path];
                let _: () = msg_send![item, setImage:image];
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
                    let sub_menu_item: id = Self::create_custom_menu_item(subitem);
                    let _: () = msg_send![submenu, addItem:sub_menu_item];
                }
                let _: () = msg_send![item, setSubmenu:submenu];
            }

            item
        };
        menu_item
    }

    #[cfg(target_os = "macos")]
    fn create_context_menu(options: &[MenuItem], window: &Window<R>) -> id {
        let _: () = Self::set_window(window.clone());
        unsafe {
            let title = NSString::alloc(nil).init_str("Menu");
            let menu: id = msg_send![class!(NSMenu), alloc];
            let menu: id = msg_send![menu, initWithTitle: title];

            let _: () = msg_send![menu, setAutoenablesItems:NO];
            
            for option in options.iter().cloned() {
                let item: id = Self::create_custom_menu_item(&option);
                let _: () = msg_send![menu, addItem:item];
            }

            let delegate_class_name = "MenuItemDelegate";
            let delegate_class: &'static objc::runtime::Class = objc::runtime::Class::get(delegate_class_name).expect("Class should exist");
            let delegate_instance: id = msg_send![delegate_class, new];
            let _: () = msg_send![menu, setDelegate:delegate_instance];

            menu
        }
    }

    fn show_context_menu(&self, window: Window<R>, pos: Option<Position>, items: Option<Vec<MenuItem>>) {
        #[cfg(target_os = "linux")]
        {
            let menu = create_context_menu();
            let window_handle = window.handle(); // This is hypothetical. There might not be a way to get the native window handle
            let gdk_window = gtk::gdk::Window::foreign_new_for_display(&gtk::gdk::Display::default(), window_handle as u32).unwrap(); // This is also hypothetical. Creating a foreign GdkWindow might not be this simple
            gdk_window.get_pointer(); // Get the pointer position
            menu.popup_at_pointer(None);
        }
        #[cfg(target_os = "macos")]
        {
            let main_queue = dispatch::Queue::main();
            main_queue.exec_async(move || {
                let items_slice = items.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
                let menu = Self::create_context_menu(items_slice, &window);
                let location = match pos {
                    // Convert web page coordinates to screen coordinates
                    Some(pos) if pos.x != 0.0 || pos.y != 0.0 => unsafe {
                        let window_position = window.outer_position().unwrap();
                        let screen: id = msg_send![class!(NSScreen), mainScreen];
                        let frame: NSRect = msg_send![screen, frame];
                        let screen_height = frame.size.height;let scale_factor = match window.scale_factor() {
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
        #[cfg(target_os = "windows")]
        {
            let menu = create_context_menu();
            nwg::menu::show_context_menu(&menu, [0, 0]);
        }
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