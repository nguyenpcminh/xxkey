slint::include_modules!();

use std::fs;
use std::path::PathBuf;

struct AppConfig {
    input_type: i32,
    modern: bool,
    spelling: bool,
    use_macro: bool,
}

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
                    _ => {}
                }
            }
        }
    }
    config
}

fn save_config(config: &AppConfig) {
    let path = get_config_path();
    let content = format!(
        "input_type={}\nmodern={}\nspelling={}\nuse_macro={}\n",
        config.input_type, config.modern, config.spelling, config.use_macro
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

    // Callbacks
    let ui_weak = ui.as_weak();
    ui.on_save_settings(move |input_type, modern, spelling, use_macro| {
        let config = AppConfig {
            input_type,
            modern,
            spelling,
            use_macro,
        };
        save_config(&config);
        
        println!("Config saved: input_type={}, modern={}, spelling={}, use_macro={}", 
                 input_type, modern, spelling, use_macro);
        
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
