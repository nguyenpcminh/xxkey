use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};
use vietime_engine::datatype::InputType;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    last_check: Instant,
    pub current: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Self {
        let path = get_config_path();
        Self::with_path(path)
    }

    pub fn with_path(path: PathBuf) -> Self {
        let mut mgr = Self {
            path,
            last_modified: None,
            last_check: Instant::now() - Duration::from_secs(10), // force initial check
            current: AppConfig::default(),
        };
        mgr.reload_if_needed();
        mgr
    }

    /// Checks if config file was modified and reloads settings dynamically if so.
    /// Throttles filesystem metadata checks to at most once per second for typing performance.
    pub fn reload_if_needed(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_check) < Duration::from_secs(1) {
            return false;
        }
        self.last_check = now;

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
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }

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
        self.last_check = Instant::now();
    }
}

pub fn get_config_path() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        if !config_home.is_empty() {
            let mut path = PathBuf::from(config_home);
            path.push("xxkey");
            path.push("config");
            return path;
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".config");
    path.push("xxkey");
    path.push("config");
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

    #[test]
    fn test_config_save_and_reload() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_xxkey_linux_config_tmp");
        let mut mgr = ConfigManager::with_path(path.clone());

        mgr.current.enabled = false;
        mgr.current.input_type = InputType::Vni;
        mgr.current.modern = false;
        mgr.save();

        let mgr2 = ConfigManager::with_path(path.clone());
        assert!(!mgr2.current.enabled);
        assert_eq!(mgr2.current.input_type, InputType::Vni);
        assert!(!mgr2.current.modern);

        let _ = fs::remove_file(path);
    }
}
