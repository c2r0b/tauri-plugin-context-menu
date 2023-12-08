use std::any::Any;
use std::sync::{Arc, Mutex};
use tauri::{Runtime, Window};

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
}

lazy_static::lazy_static! {
    pub static ref CURRENT_WINDOW: WindowHolder = WindowHolder::new();
}
