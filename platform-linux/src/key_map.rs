use vietime_engine::keycode::*;

/// Linux X11 / XKB keysym constants for key mapping
pub const XK_BACKSPACE: u32 = 0xFF08;
pub const XK_TAB: u32 = 0xFF09;
pub const XK_RETURN: u32 = 0xFF0D;
pub const XK_ESCAPE: u32 = 0xFF1B;
pub const XK_SPACE: u32 = 0x0020;
pub const XK_LEFT: u32 = 0xFF51;
pub const XK_UP: u32 = 0xFF52;
pub const XK_RIGHT: u32 = 0xFF53;
pub const XK_DOWN: u32 = 0xFF54;

pub const XK_SHIFT_L: u32 = 0xFFE1;
pub const XK_SHIFT_R: u32 = 0xFFE2;
pub const XK_CONTROL_L: u32 = 0xFFE3;
pub const XK_CONTROL_R: u32 = 0xFFE4;
pub const XK_CAPS_LOCK: u32 = 0xFFE5;
pub const XK_ALT_L: u32 = 0xFFE9;
pub const XK_ALT_R: u32 = 0xFFEA;
pub const XK_SUPER_L: u32 = 0xFFEB;
pub const XK_SUPER_R: u32 = 0xFFEC;

pub const XK_SEMICOLON: u32 = 0x003B;
pub const XK_EQUAL: u32 = 0x003D;
pub const XK_COMMA: u32 = 0x002C;
pub const XK_MINUS: u32 = 0x002D;
pub const XK_PERIOD: u32 = 0x002E;
pub const XK_SLASH: u32 = 0x002F;
pub const XK_GRAVE: u32 = 0x0060;
pub const XK_BRACKETLEFT: u32 = 0x005B;
pub const XK_BACKSLASH: u32 = 0x005C;
pub const XK_BRACKETRIGHT: u32 = 0x005D;
pub const XK_APOSTROPHE: u32 = 0x0027;

/// Maps a Linux / X11 keysym to the engine logical key code.
pub fn keysym_to_logical_key(keysym: u32) -> Option<u16> {
    match keysym {
        // Letters A-Z (lowercase 0x61..0x7A, uppercase 0x41..0x5A)
        0x0061 | 0x0041 => Some(KEY_A),
        0x0062 | 0x0042 => Some(KEY_B),
        0x0063 | 0x0043 => Some(KEY_C),
        0x0064 | 0x0044 => Some(KEY_D),
        0x0065 | 0x0045 => Some(KEY_E),
        0x0066 | 0x0046 => Some(KEY_F),
        0x0067 | 0x0047 => Some(KEY_G),
        0x0068 | 0x0048 => Some(KEY_H),
        0x0069 | 0x0049 => Some(KEY_I),
        0x006A | 0x004A => Some(KEY_J),
        0x006B | 0x004B => Some(KEY_K),
        0x006C | 0x004C => Some(KEY_L),
        0x006D | 0x004D => Some(KEY_M),
        0x006E | 0x004E => Some(KEY_N),
        0x006F | 0x004F => Some(KEY_O),
        0x0070 | 0x0050 => Some(KEY_P),
        0x0071 | 0x0051 => Some(KEY_Q),
        0x0072 | 0x0052 => Some(KEY_R),
        0x0073 | 0x0053 => Some(KEY_S),
        0x0074 | 0x0054 => Some(KEY_T),
        0x0075 | 0x0055 => Some(KEY_U),
        0x0076 | 0x0056 => Some(KEY_V),
        0x0077 | 0x0057 => Some(KEY_W),
        0x0078 | 0x0058 => Some(KEY_X),
        0x0079 | 0x0059 => Some(KEY_Y),
        0x007A | 0x005A => Some(KEY_Z),

        // Digits 0-9 (0x30 .. 0x39)
        0x0030 => Some(KEY_0),
        0x0031 => Some(KEY_1),
        0x0032 => Some(KEY_2),
        0x0033 => Some(KEY_3),
        0x0034 => Some(KEY_4),
        0x0035 => Some(KEY_5),
        0x0036 => Some(KEY_6),
        0x0037 => Some(KEY_7),
        0x0038 => Some(KEY_8),
        0x0039 => Some(KEY_9),

        // Navigation / Special
        XK_BACKSPACE => Some(KEY_DELETE),
        XK_TAB => Some(KEY_TAB),
        XK_RETURN => Some(KEY_ENTER),
        XK_ESCAPE => Some(KEY_ESC),
        XK_SPACE => Some(KEY_SPACE),
        XK_LEFT => Some(KEY_LEFT),
        XK_UP => Some(KEY_UP),
        XK_RIGHT => Some(KEY_RIGHT),
        XK_DOWN => Some(KEY_DOWN),

        // Punctuation
        XK_SEMICOLON => Some(KEY_SEMICOLON),
        XK_EQUAL => Some(KEY_EQUALS),
        XK_COMMA => Some(KEY_COMMA),
        XK_MINUS => Some(KEY_MINUS),
        XK_PERIOD => Some(KEY_DOT),
        XK_SLASH => Some(KEY_SLASH),
        XK_GRAVE => Some(KEY_BACKQUOTE),
        XK_BRACKETLEFT => Some(KEY_LEFT_BRACKET),
        XK_BACKSLASH => Some(KEY_BACK_SLASH),
        XK_BRACKETRIGHT => Some(KEY_RIGHT_BRACKET),
        XK_APOSTROPHE => Some(KEY_QUOTE),

        _ => None,
    }
}

/// Tracks Linux modifier states (Shift, Caps Lock, Ctrl, Alt, Super).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModifierState {
    pub shift: bool,
    pub caps_lock: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl ModifierState {
    /// Returns true if caps mode is active for letters (Shift XOR Caps Lock).
    pub fn is_caps(&self) -> bool {
        self.shift ^ self.caps_lock
    }

    /// Returns true if navigation or shortcut modifier (Ctrl, Alt, Super) is held.
    pub fn has_control_modifier(&self) -> bool {
        self.ctrl || self.alt || self.super_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keysym_mappings() {
        assert_eq!(keysym_to_logical_key(0x0061), Some(KEY_A));
        assert_eq!(keysym_to_logical_key(0x0041), Some(KEY_A));
        assert_eq!(keysym_to_logical_key(0x007A), Some(KEY_Z));
        assert_eq!(keysym_to_logical_key(0x0030), Some(KEY_0));
        assert_eq!(keysym_to_logical_key(XK_BACKSPACE), Some(KEY_DELETE));
        assert_eq!(keysym_to_logical_key(XK_SPACE), Some(KEY_SPACE));
        assert_eq!(keysym_to_logical_key(XK_BRACKETLEFT), Some(KEY_LEFT_BRACKET));
        assert_eq!(keysym_to_logical_key(0xFFFF_FFFF), None);
    }

    #[test]
    fn test_modifier_caps() {
        let mut mods = ModifierState::default();
        assert!(!mods.is_caps());

        mods.shift = true;
        assert!(mods.is_caps());

        mods.caps_lock = true;
        assert!(!mods.is_caps());

        mods.shift = false;
        assert!(mods.is_caps());
    }

    #[test]
    fn test_control_modifier() {
        let mut mods = ModifierState::default();
        assert!(!mods.has_control_modifier());

        mods.ctrl = true;
        assert!(mods.has_control_modifier());

        mods.ctrl = false;
        mods.super_key = true;
        assert!(mods.has_control_modifier());
    }
}
