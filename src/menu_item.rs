use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct MenuItem {
    pub label: Option<String>,
    pub disabled: Option<bool>,
    pub shortcut: Option<String>,
    pub event: Option<String>,
    pub subitems: Option<Vec<MenuItem>>,
    pub icon_path: Option<String>,
    pub is_separator: Option<bool>,
}

impl Default for MenuItem {
    fn default() -> Self {
        Self {
            label: None,
            disabled: Some(false),
            shortcut: None,
            event: None,
            subitems: None,
            icon_path: None,
            is_separator: Some(false),
        }
    }
}