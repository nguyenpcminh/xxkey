use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use vietime_engine::datatype::InputType;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub enabled: bool,
    pub input_type: InputType,
    pub modern: bool,
    pub spelling: bool,
    pub use_macro: bool,
    pub autostart: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            input_type: InputType::Telex,
            modern: true,
            spelling: true,
            use_macro: false,
            autostart: true,
        }
    }
}

pub struct ConfigManager {
    path: PathBuf,
    last_modified: Option<SystemTime>,
    pub current: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Self {
        let path = get_config_path();
        let mut mgr = Self {
            path,
            last_modified: None,
            current: AppConfig::default(),
        };
        mgr.reload_if_needed();
        mgr
    }

    /// Checks if config file was modified and reloads settings dynamically if so.
    pub fn reload_if_needed(&mut self) -> bool {
        let metadata = fs::metadata(&self.path);
        let mtime = metadata.ok().and_then(|m| m.modified().ok());

        if mtime.is_some() && mtime == self.last_modified {
            return false;
        }

        self.last_modified = mtime;

        if let Ok(content) = fs::read_to_string(&self.path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "enabled" => {
                            self.current.enabled = parts[1] == "true" || parts[1] == "1";
                        }
                        "input_type" => {
                            if let Ok(val) = parts[1].parse::<i32>() {
                                self.current.input_type = match val {
                                    1 => InputType::Vni,
                                    2 => InputType::SimpleTelex1,
                                    3 => InputType::SimpleTelex2,
                                    _ => InputType::Telex,
                                };
                            }
                        }
                        "modern" => {
                            self.current.modern = parts[1] == "true" || parts[1] == "1";
                        }
                        "spelling" => {
                            self.current.spelling = parts[1] == "true" || parts[1] == "1";
                        }
                        "use_macro" => {
                            self.current.use_macro = parts[1] == "true" || parts[1] == "1";
                        }
                        "autostart" => {
                            self.current.autostart = parts[1] == "true" || parts[1] == "1";
                        }
                        _ => {}
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Saves the current config to disk and updates `last_modified`.
    pub fn save(&mut self) {
        let it_val = match self.current.input_type {
            InputType::Telex => 0,
            InputType::Vni => 1,
            InputType::SimpleTelex1 => 2,
            InputType::SimpleTelex2 => 3,
        };

        let content = format!(
            "enabled={}\ninput_type={}\nmodern={}\nspelling={}\nuse_macro={}\nautostart={}\n",
            self.current.enabled,
            it_val,
            self.current.modern,
            self.current.spelling,
            self.current.use_macro,
            self.current.autostart
        );

        let _ = fs::write(&self.path, content);
        if let Ok(metadata) = fs::metadata(&self.path) {
            self.last_modified = metadata.modified().ok();
        }
    }
}

fn get_config_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".xxkey_config");
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert!(config.enabled);
        assert_eq!(config.input_type, InputType::Telex);
        assert!(config.modern);
        assert!(config.spelling);
    }
}
