use tauri::{Window, Runtime};
use std::sync::{Arc, Mutex};
use std::ffi::OsStr;
use std::collections::HashMap;
use std::ptr::null_mut;
use std::os::windows::ffi::OsStrExt;
use std::convert::TryInto;
use winapi::um::winuser::{
    CreatePopupMenu, AppendMenuW, TrackPopupMenu, GetCursorPos, DestroyMenu, PostQuitMessage,
    LoadImageW, SetMenuItemBitmaps, GetMessageW, TranslateMessage, DispatchMessageW, ClientToScreen,
    TPM_LEFTALIGN, TPM_TOPALIGN, TPM_RIGHTBUTTON, IMAGE_BITMAP, LR_LOADFROMFILE,
    MF_SEPARATOR, MF_ENABLED, MF_DISABLED, MF_STRING, MF_POPUP, MF_BYPOSITION,
    WM_COMMAND, MSG
};
use winapi::shared::windef::{POINT, HWND__, HWND, HMENU, HBITMAP};
use winapi::shared::minwindef::LOWORD;

use crate::{ ContextMenu, MenuItem, Position };

const ID_MENU_ITEM_BASE: u32 = 1000;

// We use a lazy_static Mutex to ensure thread safety.
// This will store a map from menu item IDs to events.
lazy_static::lazy_static! {
    static ref CALLBACK_MAP: Mutex<HashMap<u32, String>> = Mutex::new(HashMap::new());
}

unsafe fn load_bitmap_from_file(path: &str) -> HBITMAP {
    let path_wide: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0).into_iter()).collect();
    println!("Loading image from path: {}", path);
    LoadImageW(
        null_mut(),
        path_wide.as_ptr(),
        IMAGE_BITMAP,
        0,
        0,
        LR_LOADFROMFILE
    ) as HBITMAP
}

fn append_menu_item(menu: HMENU, item: &MenuItem, counter: &mut u32) -> u32 {
    let id = *counter;
    *counter += 1;

    if item.is_separator.unwrap_or(false) {
        unsafe {
            AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());
        }
    } else {
        let label = item.label.as_deref().unwrap_or("");
        let label_wide: Vec<u16> = label.encode_utf16().chain(std::iter::once(0)).collect(); // Add a null terminator
        let mut flags: u32 = MF_STRING;

        // Check if the item should be disabled
        if item.disabled.unwrap_or(false) {
            flags |= MF_DISABLED;
        } else {
            flags |= MF_ENABLED;
        }
        
        if let Some(subitems) = &item.subitems {
            let submenu = unsafe { CreatePopupMenu() };
            for subitem in subitems.iter() {
                append_menu_item(submenu, subitem, counter);
            }
            unsafe {
                AppendMenuW(menu, MF_POPUP | flags, (submenu as u32).try_into().unwrap(), label_wide.as_ptr());
            }
        } else {
            unsafe {
                AppendMenuW(menu, flags, id.try_into().unwrap(), label_wide.as_ptr());                    
            };
        }

        // If an event is provided, store it in the callback map
        if let Some(event) = &item.event {
            CALLBACK_MAP.lock().unwrap().insert(id, event.clone());
        }

        // If the icon path is provided, load the bitmap and set it for the menu item.
        if let Some(icon_path) = &item.icon_path {
            let bitmap = unsafe { load_bitmap_from_file(icon_path) };
            if bitmap.is_null() {
                println!("Failed to load image from path: {}", icon_path);
            } else {
                unsafe {
                    SetMenuItemBitmaps(menu, id as u32, MF_BYPOSITION, bitmap, bitmap);
                }
            }
        }
    }

    id
}

// This function would be called when a WM_COMMAND message is received, with the ID of the menu item that was clicked
pub fn handle_menu_item_click<R: Runtime>(id: u32, window: Window<R>) {
    if let Some(event) = CALLBACK_MAP.lock().unwrap().get(&id) {
        window.emit(event, ()).unwrap(); // Emit the event to JavaScript
    }
}

pub fn show_context_menu<R: Runtime>(_context_menu: Arc<ContextMenu<R>>, window: Window<R>, pos: Option<Position>, items: Option<Vec<MenuItem>>) {
    // Clear the callback map at the start of each context menu display
    CALLBACK_MAP.lock().unwrap().clear();

    let menu = unsafe { CreatePopupMenu() };
    let hwnd = window.hwnd().unwrap().0 as *mut HWND__;

    let mut counter = ID_MENU_ITEM_BASE;
    if let Some(menu_items) = items {
        for item in menu_items.iter() {
            append_menu_item(menu, item, &mut counter);
        }
    }

    let position = match pos {
        Some(p) => {
            let scale_factor = window.scale_factor().unwrap_or(1.0); // Use 1.0 as a default if getting the scale factor fails
            let mut point = POINT { x: (p.x * scale_factor) as i32, y: (p.y * scale_factor) as i32 };

            unsafe {
                ClientToScreen(hwnd as HWND, &mut point);
            }
            point
        },
        None => {
            // Get the current cursor position using GetCursorPos
            let mut current_pos = POINT { x: 0, y: 0 };
            unsafe {
                GetCursorPos(&mut current_pos);
            }
            current_pos
        },
    };

    unsafe {
        TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RIGHTBUTTON,
            position.x,
            position.y,
            0,  // reserved param
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
        if msg.message == WM_COMMAND {
            // Extract the menu item ID from wParam
            let menu_item_id = LOWORD(msg.wParam as u32);
            handle_menu_item_click(menu_item_id.into(), window.clone());
        }

        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
