use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn from_str(s: &str) -> Option<Theme> {
        match s {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}
