use std::collections::HashMap;

pub struct KeyBindings {
    map: HashMap<String, String>,
}

impl KeyBindings {
    /// Parses a JSON object mapping chord strings to command names.
    pub fn from_json(json: &str) -> Self {
        let map: HashMap<String, String> =
            serde_json::from_str(json).unwrap_or_default();
        KeyBindings { map }
    }

    /// Loads the platform-appropriate keybindings file at compile time.
    pub fn load_for_platform() -> Self {
        #[cfg(target_os = "macos")]
        let json = include_str!("../keybindings/macos.json");
        #[cfg(not(target_os = "macos"))]
        let json = "{}";

        Self::from_json(json)
    }

    /// Returns the command name for a chord, or None if not bound.
    pub fn resolve(&self, chord: &str) -> Option<&str> {
        self.map.get(chord).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_s_resolves_to_file_save() {
        let kb = KeyBindings::from_json(r#"{"cmd+s": "file.save"}"#);
        assert_eq!(kb.resolve("cmd+s"), Some("file.save"));
    }

    #[test]
    fn unknown_chord_returns_none() {
        let kb = KeyBindings::from_json(r#"{}"#);
        assert_eq!(kb.resolve("cmd+z"), None);
    }
}
