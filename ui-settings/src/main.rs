slint::include_modules!();

use std::fs;
use std::path::PathBuf;

struct AppConfig {
    input_type: i32,
    modern: bool,
    spelling: bool,
    use_macro: bool,
    autostart: bool,
}

#[cfg(target_os = "windows")]
fn check_autostart_enabled() -> bool {
    use std::process::Command;
    if let Ok(output) = Command::new("reg")
        .args(&["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "XXKey"])
        .output() {
        return output.status.success();
    }
    false
}

#[cfg(target_os = "windows")]
fn set_autostart(enabled: bool) {
    use std::process::Command;
    if let Ok(exe_path) = std::env::current_exe() {
        let mut target_path = exe_path.with_file_name("xxkey-daemon.exe");
        if !target_path.exists() {
            target_path = exe_path;
        }
        let path_str = target_path.to_string_lossy().to_string();
        if enabled {
            let _ = Command::new("reg")
                .args(&["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "XXKey", "/t", "REG_SZ", "/d", &path_str, "/f"])
                .status();
        } else {
            let _ = Command::new("reg")
                .args(&["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "XXKey", "/f"])
                .status();
        }
    }
}

#[cfg(target_os = "linux")]
fn check_autostart_enabled() -> bool {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut desktop_file = std::path::PathBuf::from(home);
    desktop_file.push(".config");
    desktop_file.push("autostart");
    desktop_file.push("xxkey.desktop");
    desktop_file.exists()
}

#[cfg(target_os = "linux")]
fn set_autostart(enabled: bool) {
    use std::fs;
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut autostart_dir = std::path::PathBuf::from(home);
    autostart_dir.push(".config");
    autostart_dir.push("autostart");
    let mut desktop_file = autostart_dir.clone();
    desktop_file.push("xxkey.desktop");

    if enabled {
        if let Ok(exe_path) = std::env::current_exe() {
            let mut target_path = exe_path.with_file_name("xxkey-daemon");
            if !target_path.exists() {
                target_path = exe_path;
            }
            let path_str = target_path.to_string_lossy().to_string();
            let desktop_content = format!(
                "[Desktop Entry]\n\
                 Type=Application\n\
                 Exec={}\n\
                 Hidden=false\n\
                 NoDisplay=false\n\
                 X-GNOME-Autostart-enabled=true\n\
                 Name=XXKey\n\
                 Comment=Vietnamese Input Method\n",
                path_str
            );
            let _ = fs::create_dir_all(&autostart_dir);
            let _ = fs::write(desktop_file, desktop_content);
        }
    } else {
        let _ = fs::remove_file(desktop_file);
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn check_autostart_enabled() -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn set_autostart(_enabled: bool) {}

fn get_config_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".xxkey_config");
    path
}

fn load_config() -> AppConfig {
    let path = get_config_path();
    let mut config = AppConfig {
        input_type: 0,
        modern: true,
        spelling: true,
        use_macro: false,
        autostart: check_autostart_enabled(),
    };
    if let Ok(content) = fs::read_to_string(&path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                match parts[0] {
                    "input_type" => {
                        if let Ok(val) = parts[1].parse::<i32>() {
                            config.input_type = val;
                        }
                    }
                    "modern" => {
                        config.modern = parts[1] == "true" || parts[1] == "1";
                    }
                    "spelling" => {
                        config.spelling = parts[1] == "true" || parts[1] == "1";
                    }
                    "use_macro" => {
                        config.use_macro = parts[1] == "true" || parts[1] == "1";
                    }
                    "autostart" => {
                        config.autostart = parts[1] == "true" || parts[1] == "1";
                    }
                    _ => {}
                }
            }
        }
    }
    // Sync with the actual system status
    config.autostart = check_autostart_enabled();
    config
}

fn save_config(config: &AppConfig) {
    let path = get_config_path();
    let content = format!(
        "input_type={}\nmodern={}\nspelling={}\nuse_macro={}\nautostart={}\n",
        config.input_type, config.modern, config.spelling, config.use_macro, config.autostart
    );
    let _ = fs::write(path, content);
}

fn main() -> Result<(), slint::PlatformError> {
    let config = load_config();

    let ui = MainWindow::new()?;

    // Set initial values from loaded config
    ui.set_input_type(config.input_type);
    ui.set_modern_orthography(config.modern);
    ui.set_check_spelling(config.spelling);
    ui.set_use_macro(config.use_macro);
    ui.set_autostart(config.autostart);

    // Callbacks
    let ui_weak = ui.as_weak();
    ui.on_save_settings(move |input_type, modern, spelling, use_macro, autostart| {
        let config = AppConfig {
            input_type,
            modern,
            spelling,
            use_macro,
            autostart,
        };
        save_config(&config);
        set_autostart(autostart);
        
        println!("Config saved: input_type={}, modern={}, spelling={}, use_macro={}, autostart={}", 
                 input_type, modern, spelling, use_macro, autostart);
        
        if let Some(ui) = ui_weak.upgrade() {
            let _ = ui.hide();
        }
    });

    let ui_weak2 = ui.as_weak();
    ui.on_close_window(move || {
        if let Some(ui) = ui_weak2.upgrade() {
            let _ = ui.hide();
        }
    });

    ui.run()
}
