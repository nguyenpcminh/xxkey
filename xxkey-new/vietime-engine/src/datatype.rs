//! Core data types and bit masks, ported from `DataType.h`.
//!
//! The engine stores each character of the in-progress word in a `u32`
//! "typing word" slot. The low 16 bits are the character/key code, and the
//! upper bits are flags:
//!
//! ```text
//! bit 16       : has caps
//! bit 17       : has tone ^  (hat / circumflex)
//! bit 18       : has tone w  (horn / breve)
//! bit 19 .. 23 : has mark (sắc, huyền, hỏi, ngã, nặng)
//! bit 24       : standalone key (w, [, ])
//! bit 25       : 1 = character code, 0 = keyboard code
//! ```

extern crate alloc;

use alloc::vec::Vec;

/// Max length of the in-progress word buffer.
pub const MAX_BUFF: usize = 32;

// ---------------------------------------------------------------------------
// Key event enums
// ---------------------------------------------------------------------------

/// Source of an input event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyEvent {
    Keyboard,
    Mouse,
}

/// State of the input event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyEventState {
    KeyDown,
    KeyUp,
    MouseDown,
    MouseUp,
}

/// Input method ("kiểu gõ").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputType {
    Telex = 0,
    Vni = 1,
    SimpleTelex1 = 2,
    SimpleTelex2 = 3,
}

/// Result code the engine produces for each key event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum HookCode {
    /// Do nothing, pass the key through untouched.
    DoNothing = 0,
    /// Process: rewrite part of the buffer.
    WillProcess = 1,
    /// Word break.
    BreakWord = 2,
    /// Restore characters to their previous form.
    Restore = 3,
    /// Replace by macro.
    ReplaceMacro = 4,
    /// Restore key if invalid word with break character (, . ")
    RestoreAndStartNewSession = 5,
}

/// Extra classification of the event used by front-ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ExtCode {
    /// Word break.
    WordBreak = 1,
    /// Delete key.
    Delete = 2,
    /// Normal key.
    NormalKey = 3,
    /// Should not send empty character.
    NoEmptyChar = 4,
}

/// The output of `Engine::handle_key`: how the front-end should edit text.
#[derive(Clone, Debug)]
pub struct HookState {
    pub code: HookCode,
    pub backspace_count: u8,
    pub new_char_count: u8,
    pub ext_code: ExtCode,
    /// New characters, filled last-character-first (typing order is reverse).
    pub char_data: [u32; MAX_BUFF],
    /// Used for macro (key sequence).
    pub macro_key: Vec<u32>,
    /// Used for macro (keycode data).
    pub macro_data: Vec<u32>,
}

impl Default for HookState {
    fn default() -> Self {
        HookState {
            code: HookCode::DoNothing,
            backspace_count: 0,
            new_char_count: 0,
            ext_code: ExtCode::NormalKey,
            char_data: [0; MAX_BUFF],
            macro_key: Vec::new(),
            macro_data: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Bit masks — exactly as in DataType.h
// ---------------------------------------------------------------------------

pub const CAPS_MASK: u32 = 0x1_0000;
pub const TONE_MASK: u32 = 0x2_0000;
pub const TONEW_MASK: u32 = 0x4_0000;

// MARK MASK
// 1: Dấu Sắc - á
// 2: Dấu Huyền - à
// 3: Dấu Hỏi - ả
// 4: Dấu Ngã - ã
// 5: Dấu Nặng - ạ
pub const MARK1_MASK: u32 = 0x8_0000;
pub const MARK2_MASK: u32 = 0x10_0000;
pub const MARK3_MASK: u32 = 0x20_0000;
pub const MARK4_MASK: u32 = 0x40_0000;
pub const MARK5_MASK: u32 = 0x80_0000;

/// Any mark set.
pub const MARK_MASK: u32 = 0xF8_0000;

/// Mask to get the first 16 bits (character or key code).
pub const CHAR_MASK: u32 = 0xFFFF;

/// Data was created by a standalone key (W).
pub const STANDALONE_MASK: u32 = 0x100_0000;

/// Data is a character code (not a keyboard code).
pub const CHAR_CODE_MASK: u32 = 0x200_0000;

/// For special features.
pub const PURE_CHARACTER_MASK: u32 = 0x8000_0000;

// for special feature
pub const END_CONSONANT_MASK: u16 = 0x4000;
pub const CONSONANT_ALLOW_MASK: u16 = 0x8000;

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// True if `key_code` is not one of the vowels (a,e,u,y,i,o).
#[inline]
pub fn is_consonant(key_code: u16) -> bool {
    !(key_code == crate::keycode::KEY_A
        || key_code == crate::keycode::KEY_E
        || key_code == crate::keycode::KEY_U
        || key_code == crate::keycode::KEY_Y
        || key_code == crate::keycode::KEY_I
        || key_code == crate::keycode::KEY_O)
}

/// Whether `code` is a mark key (per input type).
#[inline]
pub fn is_mark_key(input_type: InputType, code: u16) -> bool {
    match input_type {
        InputType::Telex | InputType::SimpleTelex1 | InputType::SimpleTelex2 => {
            code == crate::keycode::KEY_S
                || code == crate::keycode::KEY_F
                || code == crate::keycode::KEY_R
                || code == crate::keycode::KEY_J
                || code == crate::keycode::KEY_X
        }
        InputType::Vni => {
            code == crate::keycode::KEY_1
                || code == crate::keycode::KEY_2
                || code == crate::keycode::KEY_3
                || code == crate::keycode::KEY_5
                || code == crate::keycode::KEY_4
        }
    }
}

/// Number key (top row digits 0-9).
#[inline]
pub fn is_number_key(code: u16) -> bool {
    code >= crate::keycode::KEY_1 && code <= crate::keycode::KEY_0
}
