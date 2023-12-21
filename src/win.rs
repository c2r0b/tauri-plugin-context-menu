use std::collections::HashMap;
use std::convert::TryInto;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use tauri::{Runtime, Window};
use winapi::{
    shared::minwindef::LOWORD,
    shared::windef::{HMENU, HWND, HWND__, POINT},
    um::winuser::{
        AppendMenuW, ClientToScreen, CreatePopupMenu, DestroyMenu, DispatchMessageW, GetCursorPos,
        GetMessageW, PostQuitMessage, SetMenuItemBitmaps, TrackPopupMenu, TranslateMessage,
        MF_BYCOMMAND, MF_CHECKED, MF_DISABLED, MF_ENABLED, MF_POPUP, MF_SEPARATOR, MF_STRING, MSG,
        TPM_LEFTALIGN, TPM_RIGHTBUTTON, TPM_TOPALIGN, WM_COMMAND,
    },
};

use crate::keymap::get_key_map;
use crate::win_image_handler::{convert_to_hbitmap, load_bitmap_from_file};
use crate::{MenuItem, Position};

const ID_MENU_ITEM_BASE: u32 = 1000;

// We use a lazy_static Mutex to ensure thread safety.
// This will store a map from menu item IDs to events.
lazy_static::lazy_static! {
    static ref CALLBACK_MAP: Mutex<HashMap<u32, (String, Option<String>)>> = Mutex::new(HashMap::new());
}

pub fn get_label_with_shortcut(label: &str, shortcut: Option<&str>) -> String {
    let key_map = get_key_map();

    label.to_string()
        + &shortcut.map_or_else(String::new, |s| {
            format!(
                "\t{}",
                s.split('+')
                    .map(|part| {
                        let mut c = part.chars();
                        // If the part exists in the key_map, use the key_map value.
                        // Otherwise, use the original logic.
                        key_map.get(part).map_or_else(
                            || c.next().unwrap_or_default().to_uppercase().to_string() + c.as_str(),
                            |value| value.to_string(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("+")
            )
        })
}

fn append_menu_item(menu: HMENU, item: &MenuItem, counter: &mut u32) -> Result<u32, String> {
    let id = *counter;
    *counter += 1;

    if item.is_separator.unwrap_or(false) {
        unsafe {
            AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());
        }
    } else {
        let label = item.label.as_deref().unwrap_or("");
        let shortcut = item.shortcut.as_deref();
        let menu_label = get_label_with_shortcut(label, shortcut);
        let label_wide: Vec<u16> = menu_label
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect(); // Add a null terminator
        let mut flags: u32 = MF_STRING;

        // Check if the item should be disabled
        if item.disabled.unwrap_or(false) {
            flags |= MF_DISABLED;
        } else {
            flags |= MF_ENABLED;
        }

        // Check if the item is checkable and set the initial state
        if item.checked.unwrap_or(false) {
            flags |= MF_CHECKED;
        }

        if let Some(subitems) = &item.subitems {
            let submenu = unsafe { CreatePopupMenu() };
            for subitem in subitems.iter() {
                let _ = append_menu_item(submenu, subitem, counter);
            }
            unsafe {
                AppendMenuW(
                    menu,
                    MF_POPUP | flags,
                    (submenu as u32).try_into().unwrap(),
                    label_wide.as_ptr(),
                );
            }
        } else {
            unsafe {
                AppendMenuW(menu, flags, id.try_into().unwrap(), label_wide.as_ptr());
            };
        }

        // If an event is provided, store it in the callback map
        if let Some(event) = &item.event {
            CALLBACK_MAP
                .lock()
                .unwrap()
                .insert(id, (event.clone(), item.payload.clone()));
        }

        // If the icon path is provided, load the bitmap and set it for the menu item.
        if let Some(icon) = &item.icon {
            match load_bitmap_from_file(&icon.path, icon.width, icon.height) {
                Ok(bitmap) => match convert_to_hbitmap(bitmap) {
                    Ok(hbitmap) => {
                        if !hbitmap.is_null() {
                            unsafe {
                                SetMenuItemBitmaps(menu, id as u32, MF_BYCOMMAND, hbitmap, hbitmap);
                            }
                        } else {
                            return Err(format!("Failed to load bitmap from path: {}", icon.path));
                        }
                    }
                    Err(err_msg) => return Err(err_msg),
                },
                Err(err) => {
                    return Err(format!(
                        "Failed to load image from path: {}. Error: {:?}",
                        icon.path, err
                    ))
                }
            }
        }
    }

    Ok(id)
}

// This function would be called when a WM_COMMAND message is received, with the ID of the menu item that was clicked
pub fn handle_menu_item_click<R: Runtime>(id: u32, window: Window<R>) {
    if let Some((event, payload)) = CALLBACK_MAP.lock().unwrap().get(&id) {
        window.emit(event, &payload).unwrap(); // Emit the event to JavaScript
    }
}

pub fn show_context_menu<R: Runtime>(
    window: Window<R>,
    pos: Option<Position>,
    items: Option<Vec<MenuItem>>,
) {
    // Clear the callback map at the start of each context menu display
    CALLBACK_MAP.lock().unwrap().clear();

    let menu = unsafe { CreatePopupMenu() };
    let hwnd = window.hwnd().unwrap().0 as *mut HWND__;

    let mut counter = ID_MENU_ITEM_BASE;
    if let Some(menu_items) = items {
        for item in menu_items.iter() {
            let _ = append_menu_item(menu, item, &mut counter);
        }
    }

    let position = match pos {
        Some(p) => {
            let scale_factor = window.scale_factor().unwrap_or(1.0); // Use 1.0 as a default if getting the scale factor fails
            let mut point = POINT {
                x: (p.x * scale_factor) as i32,
                y: (p.y * scale_factor) as i32,
            };

            if p.is_absolute.unwrap_or(false) {
                point
            } else {
                unsafe {
                    ClientToScreen(hwnd as HWND, &mut point);
                }
                point
            }
        }
        None => {
            // Get the current cursor position using GetCursorPos
            let mut current_pos = POINT { x: 0, y: 0 };
            unsafe {
                GetCursorPos(&mut current_pos);
            }
            current_pos
        }
    };

    unsafe {
        TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RIGHTBUTTON,
            position.x,
            position.y,
            0, // reserved param
            hwnd as HWND,
            std::ptr::null_mut(),
        );

        DestroyMenu(menu);

        // Post a quit message to exit the message loop
        PostQuitMessage(0);
    }

    // Emit the menu-did-close event to JavaScript
    window.emit("menu-did-close", ()).unwrap();

    let mut msg: MSG = unsafe { std::mem::zeroed() };
    while unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) } > 0 {
        match msg.message {
            WM_COMMAND => {
                // Extract the menu item ID from wParam
                let menu_item_id = LOWORD(msg.wParam as u32);
                handle_menu_item_click(menu_item_id.into(), window.clone());
            }
            _ => unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            },
        }
    }
}
