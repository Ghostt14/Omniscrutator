// Persisted user settings. Serialized to a plain JSON file sitting next to
// the exe (so the whole thing stays portable -- no installer, no registry,
// move the folder anywhere and it still works).

use crate::everything::ResultPriority;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: String,            // e.g. "Alt+Space" -- parsed once we wire RegisterHotKey
    pub max_results: usize,
    pub web_search_prefix: String, // typing this prefix routes the query to a browser search
    pub web_search_url: String,    // "{query}" gets substituted in
    pub copy_path_hotkey: String,  // e.g. "Ctrl+Enter"
    pub result_priority: ResultPriority, // what gets sorted to the front of results
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Space".to_string(),
            max_results: 50,
            web_search_prefix: "?".to_string(),
            web_search_url: "https://www.google.com/search?q={query}".to_string(),
            copy_path_hotkey: "Ctrl+Enter".to_string(),
            result_priority: ResultPriority::General,
        }
    }
}

impl Settings {
    fn path() -> std::path::PathBuf {
        // Next to the exe, for portability. Falls back to cwd if that lookup fails.
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("omniscrutator_settings.json")))
            .unwrap_or_else(|| std::path::PathBuf::from("omniscrutator_settings.json"))
    }

    pub fn load() -> Self {
        match std::fs::read_to_string(Self::path()) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(Self::path(), text);
        }
    }
}
