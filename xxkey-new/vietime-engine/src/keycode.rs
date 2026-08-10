//! Logical key codes used by the engine.
//!
//! These mirror `platforms/mac.h` from the original C++ engine. The engine is
//! platform-agnostic: platform adapters translate OS-specific codes into these
//! logical codes before calling `Engine::handle_key`.

/// Esc.
pub const KEY_ESC: u16 = 53;
/// Delete (backspace).
pub const KEY_DELETE: u16 = 51;
pub const KEY_TAB: u16 = 48;
pub const KEY_ENTER: u16 = 76;
pub const KEY_RETURN: u16 = 36;
pub const KEY_SPACE: u16 = 49;
pub const KEY_LEFT: u16 = 123;
pub const KEY_RIGHT: u16 = 124;
pub const KEY_DOWN: u16 = 125;
pub const KEY_UP: u16 = 126;

/// No key / empty code.
pub const KEY_EMPTY: u16 = 256;
pub const KEY_A: u16 = 0;
pub const KEY_B: u16 = 11;
pub const KEY_C: u16 = 8;
pub const KEY_D: u16 = 2;
pub const KEY_E: u16 = 14;
pub const KEY_F: u16 = 3;
pub const KEY_G: u16 = 5;
pub const KEY_H: u16 = 4;
pub const KEY_I: u16 = 34;
pub const KEY_J: u16 = 38;
pub const KEY_K: u16 = 40;
pub const KEY_L: u16 = 37;
pub const KEY_M: u16 = 46;
pub const KEY_N: u16 = 45;
pub const KEY_O: u16 = 31;
pub const KEY_P: u16 = 35;
pub const KEY_Q: u16 = 12;
pub const KEY_R: u16 = 15;
pub const KEY_S: u16 = 1;
pub const KEY_T: u16 = 17;
pub const KEY_U: u16 = 32;
pub const KEY_V: u16 = 9;
pub const KEY_W: u16 = 13;
pub const KEY_X: u16 = 7;
pub const KEY_Y: u16 = 16;
pub const KEY_Z: u16 = 6;

pub const KEY_1: u16 = 18;
pub const KEY_2: u16 = 19;
pub const KEY_3: u16 = 20;
pub const KEY_4: u16 = 21;
pub const KEY_5: u16 = 23;
pub const KEY_6: u16 = 22;
pub const KEY_7: u16 = 26;
pub const KEY_8: u16 = 28;
pub const KEY_9: u16 = 25;
pub const KEY_0: u16 = 29;

pub const KEY_LEFT_BRACKET: u16 = 33;
pub const KEY_RIGHT_BRACKET: u16 = 30;

pub const KEY_LEFT_SHIFT: u16 = 57;
pub const KEY_RIGHT_SHIFT: u16 = 60;
pub const KEY_DOT: u16 = 47;

pub const KEY_BACKQUOTE: u16 = 50;
pub const KEY_MINUS: u16 = 27;
pub const KEY_EQUALS: u16 = 24;
pub const KEY_BACK_SLASH: u16 = 42;
pub const KEY_SEMICOLON: u16 = 41;
pub const KEY_QUOTE: u16 = 39;
pub const KEY_COMMA: u16 = 43;
pub const KEY_SLASH: u16 = 44;
