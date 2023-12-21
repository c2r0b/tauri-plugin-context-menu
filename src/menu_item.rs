use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct MenuItem {
    pub label: Option<String>,
    pub disabled: Option<bool>,
    pub shortcut: Option<String>,
    pub event: Option<String>,
    pub payload: Option<String>,
    pub subitems: Option<Vec<MenuItem>>,
    pub icon: Option<MenuItemIcon>,
    pub checked: Option<bool>,
    pub is_separator: Option<bool>,
}

#[derive(Clone, Deserialize)]
pub struct MenuItemIcon {
    pub path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Default for MenuItem {
    fn default() -> Self {
        Self {
            label: None,
            disabled: Some(false),
            shortcut: None,
            event: None,
            payload: None,
            subitems: None,
            icon: None,
            checked: Some(false),
            is_separator: Some(false),
        }
    }
}
