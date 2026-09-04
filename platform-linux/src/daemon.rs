use crate::config::ConfigManager;
use crate::injector::{BufferInjector, EditCommand};
use crate::key_map::{
    keysym_to_logical_key, ModifierState, XK_ALT_L, XK_ALT_R, XK_CAPS_LOCK, XK_CONTROL_L,
    XK_CONTROL_R, XK_SHIFT_L, XK_SHIFT_R, XK_SPACE, XK_SUPER_L, XK_SUPER_R,
};
use vietime_engine::datatype::{
    CHAR_CODE_MASK, HookCode, KeyEvent, KeyEventState,
};
use vietime_engine::engine::Engine;
use vietime_engine::vietnamese::key_code_to_character;

/// Converts engine `char_data` entry (which contains character/key code bit flags) into Unicode codepoint.
#[inline]
pub fn format_char_code(code: u32) -> u32 {
    if code & CHAR_CODE_MASK != 0 {
        code & 0xFFFF
    } else {
        let ch = key_code_to_character(code);
        if ch == 0 {
            code & 0xFFFF
        } else {
            ch as u32
        }
    }
}

/// Main Linux daemon structure managing input state, hotkeys, and engine processing.
pub struct LinuxDaemon {
    pub engine: Engine,
    pub config_mgr: ConfigManager,
    pub modifiers: ModifierState,
    pub injector: BufferInjector,
    formatted_buf: [u32; 32],
}

impl LinuxDaemon {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            config_mgr: ConfigManager::new(),
            modifiers: ModifierState::default(),
            injector: BufferInjector::new(),
            formatted_buf: [0u32; 32],
        }
    }

    pub fn with_config(config_mgr: ConfigManager) -> Self {
        Self {
            engine: Engine::default(),
            config_mgr,
            modifiers: ModifierState::default(),
            injector: BufferInjector::new(),
            formatted_buf: [0u32; 32],
        }
    }

    /// Updates tracked modifier state given a keysym and whether it was key down or up.
    #[inline]
    pub fn update_modifiers(&mut self, keysym: u32, is_down: bool) {
        match keysym {
            XK_SHIFT_L | XK_SHIFT_R => self.modifiers.shift = is_down,
            XK_CONTROL_L | XK_CONTROL_R => self.modifiers.ctrl = is_down,
            XK_ALT_L | XK_ALT_R => self.modifiers.alt = is_down,
            XK_SUPER_L | XK_SUPER_R => self.modifiers.super_key = is_down,
            XK_CAPS_LOCK => {
                if is_down {
                    self.modifiers.caps_lock = !self.modifiers.caps_lock;
                }
            }
            _ => {}
        }
    }

    /// Processes a key event on Linux.
    /// Returns `Some(EditCommand)` if the key should be intercepted and replaced, or `None` if passthrough.
    pub fn process_key(&mut self, keysym: u32, is_down: bool) -> Option<&EditCommand> {
        self.config_mgr.reload_if_needed();
        self.update_modifiers(keysym, is_down);

        // Global hotkey check: Ctrl+Space or Alt+Z toggles ON/OFF on key down (fast inline checks)
        if is_down {
            let is_ctrl_space = self.modifiers.ctrl && keysym == XK_SPACE;
            let is_alt_z = self.modifiers.alt && (keysym == 0x007A || keysym == 0x005A); // 'z' or 'Z'

            if is_ctrl_space || is_alt_z {
                self.config_mgr.current.enabled = !self.config_mgr.current.enabled;
                self.config_mgr.save();
                return None; // Suppress / consume hotkey
            }
        }

        if !is_down {
            return None;
        }

        // If engine is disabled or Ctrl/Alt/Super is held, reset engine and pass through key
        if !self.config_mgr.current.enabled || self.modifiers.has_control_modifier() {
            self.engine.reset();
            return None;
        }

        // Apply configuration to engine
        self.engine.cfg.input_type = self.config_mgr.current.input_type;
        self.engine.cfg.use_modern_orthography = self.config_mgr.current.modern;
        self.engine.cfg.check_spelling = self.config_mgr.current.spelling;
        self.engine.cfg.use_macro = self.config_mgr.current.use_macro;

        if let Some(logical_key) = keysym_to_logical_key(keysym) {
            let caps_status = if self.modifiers.is_caps() { 1 } else { 0 };
            let other_ctrl = false;

            let hook_state = self.engine.handle_key(
                KeyEvent::Keyboard,
                KeyEventState::KeyDown,
                logical_key,
                caps_status,
                other_ctrl,
            );

            match hook_state.code {
                HookCode::WillProcess
                | HookCode::Restore
                | HookCode::ReplaceMacro
                | HookCode::RestoreAndStartNewSession => {
                    let bs_count = hook_state.backspace_count as usize;
                    let new_count = hook_state.new_char_count as usize;

                    for i in 0..new_count {
                        self.formatted_buf[i] = format_char_code(hook_state.char_data[i]);
                    }

                    let cmd = self.injector.prepare(bs_count, &self.formatted_buf, new_count);
                    return Some(cmd);
                }
                HookCode::BreakWord | HookCode::DoNothing => {
                    return None;
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vietime_engine::datatype::InputType;

    #[test]
    fn test_daemon_hotkey_toggle() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_daemon_hotkey_tmp");
        let cfg_mgr = ConfigManager::with_path(path.clone());
        let mut daemon = LinuxDaemon::with_config(cfg_mgr);

        assert!(daemon.config_mgr.current.enabled);

        // Press Ctrl
        daemon.update_modifiers(XK_CONTROL_L, true);
        // Press Space -> Toggle OFF
        let res = daemon.process_key(XK_SPACE, true);
        assert!(res.is_none());
        assert!(!daemon.config_mgr.current.enabled);

        // Press Space again with Ctrl -> Toggle ON
        let res2 = daemon.process_key(XK_SPACE, true);
        assert!(res2.is_none());
        assert!(daemon.config_mgr.current.enabled);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_daemon_telex_typing() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_daemon_typing_tmp");
        let cfg_mgr = ConfigManager::with_path(path.clone());
        let mut daemon = LinuxDaemon::with_config(cfg_mgr);

        daemon.config_mgr.current.input_type = InputType::Telex;

        // Press 'd' -> passthrough (None)
        let cmd1 = daemon.process_key(0x0064, true);
        assert!(cmd1.is_none());

        // Press 'd' -> 'đ' (WillProcess: backspace 1, output [0x0111])
        let cmd2 = daemon.process_key(0x0064, true).unwrap();
        assert_eq!(cmd2.backspace_count, 1);
        assert_eq!(cmd2.chars.len(), 1);
        assert_eq!(cmd2.chars[0], 'đ' as u32);
        assert_eq!(cmd2.to_string(), "đ");

        // Press 'a' -> passthrough (None)
        let cmd3 = daemon.process_key(0x0061, true);
        assert!(cmd3.is_none());

        // Press 'a' -> 'â'
        let cmd4 = daemon.process_key(0x0061, true).unwrap();
        assert_eq!(cmd4.backspace_count, 1);
        assert_eq!(cmd4.chars[0], 'â' as u32);
        assert_eq!(cmd4.to_string(), "â");

        // Press 's' -> 'ấ'
        let cmd5 = daemon.process_key(0x0073, true).unwrap();
        assert_eq!(cmd5.backspace_count, 1);
        assert_eq!(cmd5.chars[0], 'ấ' as u32);
        assert_eq!(cmd5.to_string(), "ấ");

        let _ = std::fs::remove_file(path);
    }
}
