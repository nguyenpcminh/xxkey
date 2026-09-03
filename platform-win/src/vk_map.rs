use vietime_engine::keycode::*;

/// Windows Virtual Key codes constants for readability
pub const VK_BACK: u32 = 0x08;
pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_MENU: u32 = 0x12; // Alt
pub const VK_CAPITAL: u32 = 0x14; // Caps Lock
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_SPACE: u32 = 0x20;
pub const VK_PRIOR: u32 = 0x21; // Page Up
pub const VK_NEXT: u32 = 0x22; // Page Down
pub const VK_END: u32 = 0x23;
pub const VK_HOME: u32 = 0x24;
pub const VK_LEFT: u32 = 0x25;
pub const VK_UP: u32 = 0x26;
pub const VK_RIGHT: u32 = 0x27;
pub const VK_DOWN: u32 = 0x28;
pub const VK_DELETE: u32 = 0x2E;
pub const VK_LSHIFT: u32 = 0xA0;
pub const VK_RSHIFT: u32 = 0xA1;
pub const VK_LCONTROL: u32 = 0xA2;
pub const VK_RCONTROL: u32 = 0xA3;

pub const VK_OEM_1: u32 = 0xBA; // ';:'
pub const VK_OEM_PLUS: u32 = 0xBB; // '=+'
pub const VK_OEM_COMMA: u32 = 0xBC; // ',<'
pub const VK_OEM_MINUS: u32 = 0xBD; // '-_'
pub const VK_OEM_PERIOD: u32 = 0xBE; // '.>'
pub const VK_OEM_2: u32 = 0xBF; // '/?'
pub const VK_OEM_3: u32 = 0xC0; // '`~'
pub const VK_OEM_4: u32 = 0xDB; // '[{'
pub const VK_OEM_5: u32 = 0xDC; // '\|'
pub const VK_OEM_6: u32 = 0xDD; // ']}'
pub const VK_OEM_7: u32 = 0xDE; // '\''"

/// Maps a Windows Virtual Key code to the engine logical key code.
pub fn vk_to_logical_key(vk: u32) -> Option<u16> {
    match vk {
        // Letters A-Z (0x41 .. 0x5A)
        0x41 => Some(KEY_A),
        0x42 => Some(KEY_B),
        0x43 => Some(KEY_C),
        0x44 => Some(KEY_D),
        0x45 => Some(KEY_E),
        0x46 => Some(KEY_F),
        0x47 => Some(KEY_G),
        0x48 => Some(KEY_H),
        0x49 => Some(KEY_I),
        0x4A => Some(KEY_J),
        0x4B => Some(KEY_K),
        0x4C => Some(KEY_L),
        0x4D => Some(KEY_M),
        0x4E => Some(KEY_N),
        0x4F => Some(KEY_O),
        0x50 => Some(KEY_P),
        0x51 => Some(KEY_Q),
        0x52 => Some(KEY_R),
        0x53 => Some(KEY_S),
        0x54 => Some(KEY_T),
        0x55 => Some(KEY_U),
        0x56 => Some(KEY_V),
        0x57 => Some(KEY_W),
        0x58 => Some(KEY_X),
        0x59 => Some(KEY_Y),
        0x5A => Some(KEY_Z),

        // Digits 0-9 (0x30 .. 0x39)
        0x30 => Some(KEY_0),
        0x31 => Some(KEY_1),
        0x32 => Some(KEY_2),
        0x33 => Some(KEY_3),
        0x34 => Some(KEY_4),
        0x35 => Some(KEY_5),
        0x36 => Some(KEY_6),
        0x37 => Some(KEY_7),
        0x38 => Some(KEY_8),
        0x39 => Some(KEY_9),

        // Controls / Navigation
        VK_BACK => Some(KEY_DELETE),
        VK_TAB => Some(KEY_TAB),
        VK_RETURN => Some(KEY_ENTER),
        VK_ESCAPE => Some(KEY_ESC),
        VK_SPACE => Some(KEY_SPACE),
        VK_LEFT => Some(KEY_LEFT),
        VK_UP => Some(KEY_UP),
        VK_RIGHT => Some(KEY_RIGHT),
        VK_DOWN => Some(KEY_DOWN),

        // Punctuation / OEM
        VK_OEM_1 => Some(KEY_SEMICOLON),
        VK_OEM_PLUS => Some(KEY_EQUALS),
        VK_OEM_COMMA => Some(KEY_COMMA),
        VK_OEM_MINUS => Some(KEY_MINUS),
        VK_OEM_PERIOD => Some(KEY_DOT),
        VK_OEM_2 => Some(KEY_SLASH),
        VK_OEM_3 => Some(KEY_BACKQUOTE),
        VK_OEM_4 => Some(KEY_LEFT_BRACKET),
        VK_OEM_5 => Some(KEY_BACK_SLASH),
        VK_OEM_6 => Some(KEY_RIGHT_BRACKET),
        VK_OEM_7 => Some(KEY_QUOTE),

        _ => None,
    }
}

/// Tracks modifier states (Shift, Caps Lock, Ctrl, Alt, Win).
#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    pub shift: bool,
    pub caps_lock: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

impl ModifierState {
    /// Returns true if caps mode is active for letters (Shift XOR Caps Lock).
    pub fn is_caps(&self) -> bool {
        self.shift ^ self.caps_lock
    }

    /// Returns true if navigation or shortcut modifier (Ctrl, Alt, Win) is held.
    pub fn has_control_modifier(&self) -> bool {
        self.ctrl || self.alt || self.win
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vk_mappings() {
        assert_eq!(vk_to_logical_key(0x41), Some(KEY_A));
        assert_eq!(vk_to_logical_key(0x5A), Some(KEY_Z));
        assert_eq!(vk_to_logical_key(0x30), Some(KEY_0));
        assert_eq!(vk_to_logical_key(0x31), Some(KEY_1));
        assert_eq!(vk_to_logical_key(VK_BACK), Some(KEY_DELETE));
        assert_eq!(vk_to_logical_key(VK_SPACE), Some(KEY_SPACE));
        assert_eq!(vk_to_logical_key(VK_OEM_4), Some(KEY_LEFT_BRACKET));
        assert_eq!(vk_to_logical_key(0xFF), None);
    }

    #[test]
    fn test_modifier_caps() {
        let mut mods = ModifierState::default();
        assert!(!mods.is_caps());

        mods.shift = true;
        assert!(mods.is_caps());

        mods.caps_lock = true;
        assert!(!mods.is_caps()); // Shift + CapsLock = lowercase

        mods.shift = false;
        assert!(mods.is_caps()); // CapsLock alone = uppercase
    }
}
