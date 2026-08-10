//! The Vietnamese input engine state machine.
//!
//! Ported from `Engine.cpp`. The engine keeps a buffer of the in-progress word
//! (`TypingWord`) and, for each key event, decides whether to rewrite part of
//! it and produce a `HookState` describing the edit for the front-end.
//!
//! Safety: all indexing is bounds-checked; buffers are fixed-size; the engine
//! never allocates (no_std). In debug builds index arithmetic that would go out
//! of range panics loudly so tests catch bugs; in release it saturates.

extern crate alloc;

use alloc::vec::Vec;
use crate::datatype::*;
use crate::keycode::*;
use crate::vietnamese::*;

// ---------------------------------------------------------------------------
// Static tables (mirror Engine.cpp statics)
// ---------------------------------------------------------------------------

/// Keys that are "character keys" (can be part of a word / macro).
pub const CHAR_KEY_CODE: &[u16] = &[
    KEY_BACKQUOTE, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0, KEY_MINUS,
    KEY_EQUALS, KEY_LEFT_BRACKET, KEY_RIGHT_BRACKET, KEY_BACK_SLASH, KEY_SEMICOLON, KEY_QUOTE,
    KEY_COMMA, KEY_DOT, KEY_SLASH,
];

/// Keys that break a word.
pub const BREAK_CODE: &[u16] = &[
    KEY_ESC, KEY_TAB, KEY_ENTER, KEY_RETURN, KEY_LEFT, KEY_RIGHT, KEY_DOWN, KEY_UP, KEY_COMMA,
    KEY_DOT, KEY_SLASH, KEY_SEMICOLON, KEY_QUOTE, KEY_BACK_SLASH, KEY_MINUS, KEY_EQUALS,
    KEY_BACKQUOTE, KEY_TAB,
];

/// Keys that break a macro.
pub const MACRO_BREAK_CODE: &[u16] = &[
    KEY_RETURN, KEY_COMMA, KEY_DOT, KEY_SLASH, KEY_SEMICOLON, KEY_QUOTE, KEY_BACK_SLASH, KEY_MINUS,
    KEY_EQUALS,
];

/// Per-input-type processing keys (the 11 special keys).
///
/// Layout per row: [S, F, R, X, J, A, O, E, W, D, Z]
/// (for VNI: [1,2,3,4,5,6,6,7,8,9,0])
pub const PROCESSING_CHAR: [[u16; 11]; 4] = [
    [KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z], // Telex
    [KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0], // VNI
    [KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z], // Simple Telex 1
    [KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z], // Simple Telex 2
];

// ---------------------------------------------------------------------------
// Engine state
// ---------------------------------------------------------------------------

/// Configuration knobs the engine reads (mirrors the `v*` globals).
#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub language: bool,              // true = Vietnamese
    pub input_type: InputType,
    pub free_mark: bool,
    pub code_table: u8,              // 0..4
    pub check_spelling: bool,
    pub use_modern_orthography: bool,
    pub quick_telex: bool,
    pub restore_if_wrong_spelling: bool,
    pub use_macro: bool,
    pub use_macro_in_english_mode: bool,
    pub auto_caps_macro: bool,
    pub use_smart_switch_key: bool,
    pub upper_case_first_char: bool,
    pub temp_off_spelling: bool,
    pub allow_consonant_zfwj: bool,
    pub quick_start_consonant: bool,
    pub quick_end_consonant: bool,
    pub remember_code: bool,
    pub other_language: bool,
    pub temp_off_open_key: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            language: true,
            input_type: InputType::Telex,
            free_mark: false,
            code_table: 0,
            check_spelling: true,
            use_modern_orthography: true,
            quick_telex: false,
            restore_if_wrong_spelling: false,
            use_macro: false,
            use_macro_in_english_mode: false,
            auto_caps_macro: false,
            use_smart_switch_key: false,
            upper_case_first_char: false,
            temp_off_spelling: false,
            allow_consonant_zfwj: false,
            quick_start_consonant: false,
            quick_end_consonant: false,
            remember_code: false,
            other_language: false,
            temp_off_open_key: false,
        }
    }
}

/// The engine instance. Holds all mutable state (previously static globals).
#[derive(Clone, Debug)]
pub struct Engine {
    pub cfg: EngineConfig,

    /// In-progress word buffer (TypingWord).
    typing_word: [u32; MAX_BUFF],
    index: usize,

    /// Long-word overflow helper.
    long_word_helper: Vec<u32>,

    /// Saved key states (for restore-if-wrong-spelling).
    key_states: [u32; MAX_BUFF],
    state_index: usize,

    /// Typing history for undo (restoreLastTypingState).
    typing_states: Vec<Vec<u32>>,
    typing_states_data: Vec<u32>,
    special_char: Vec<u32>,
    space_count: usize,
    upper_case_status: u8,
    has_handled_macro: bool,
    has_handle_quick_consonant: bool,
    will_temp_off_engine: bool,
    use_spell_checking_before: bool,

    /// Scratch output state.
    pub state: HookState,

    // scratch fields (formerly globals i,ii,iii,j,k,kk,l,etc.)
    vowel_start_index: usize,
    vowel_end_index: usize,
    vowel_will_set_mark: usize,
    vowel_count: usize,
    is_checked_grammar: bool,
    is_correct: bool,
    is_changed: bool,
    is_restored_w: bool,
    temp_disable_key: bool,
    is_caps: bool,
    caps_elem: usize,
    key: u32,
    mark_elem: i32,
    key_for_aeo: u16,
    // loop counters
    i: usize,
    ii: usize,
    iii: usize,
    j: usize,
    k: usize,
    kk: usize,
    l: usize,
    // spelling scratch
    spelling_ok: bool,
    spelling_flag: bool,
    spelling_vowel_ok: bool,
    spelling_end_index: usize,
}

impl Default for Engine {
    fn default() -> Self {
        Engine {
            cfg: EngineConfig::default(),
            typing_word: [0; MAX_BUFF],
            index: 0,
            long_word_helper: Vec::new(),
            key_states: [0; MAX_BUFF],
            state_index: 0,
            typing_states: Vec::new(),
            typing_states_data: Vec::new(),
            special_char: Vec::new(),
            space_count: 0,
            upper_case_status: 0,
            has_handled_macro: false,
            has_handle_quick_consonant: false,
            will_temp_off_engine: false,
            use_spell_checking_before: true,
            state: HookState::default(),
            vowel_start_index: 0,
            vowel_end_index: 0,
            vowel_will_set_mark: 0,
            vowel_count: 0,
            is_checked_grammar: false,
            is_correct: false,
            is_changed: false,
            is_restored_w: false,
            temp_disable_key: false,
            is_caps: false,
            caps_elem: 0,
            key: 0,
            mark_elem: 0,
            key_for_aeo: 0,
            i: 0,
            ii: 0,
            iii: 0,
            j: 0,
            k: 0,
            kk: 0,
            l: 0,
            spelling_ok: false,
            spelling_flag: false,
            spelling_vowel_ok: false,
            spelling_end_index: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers (port of the engine's small functions)
// ---------------------------------------------------------------------------

impl Engine {
    /// Raw character at `index` in the typing word (low 16 bits).
    #[inline]
    fn chr(&self, index: usize) -> u16 {
        self.typing_word.get(index).map(|w| (*w & 0xFFFF) as u16).unwrap_or(0)
    }

    /// Direct word accessor (for tests / port verification).
    #[inline]
    pub fn word(&self, index: usize) -> u32 {
        self.typing_word.get(index).copied().unwrap_or(0)
    }

    /// Current word length.
    #[inline]
    pub fn len(&self) -> usize {
        self.index
    }

    #[inline]
    fn is_key_z(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.cfg.input_type as usize][10] == key
    }
    #[inline]
    fn is_key_d(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.cfg.input_type as usize][9] == key
    }
    #[inline]
    fn is_key_w(&self, key: u16) -> bool {
        if self.cfg.input_type != InputType::Vni {
            PROCESSING_CHAR[self.cfg.input_type as usize][8] == key
        } else {
            // VNI: W key is 7, and '6' (KEY_6) also maps through here
            PROCESSING_CHAR[self.cfg.input_type as usize][8] == key
                || PROCESSING_CHAR[self.cfg.input_type as usize][7] == key
        }
    }
    #[inline]
    fn is_key_double(&self, key: u16) -> bool {
        if self.cfg.input_type != InputType::Vni {
            PROCESSING_CHAR[self.cfg.input_type as usize][5] == key
                || PROCESSING_CHAR[self.cfg.input_type as usize][6] == key
                || PROCESSING_CHAR[self.cfg.input_type as usize][7] == key
        } else {
            PROCESSING_CHAR[self.cfg.input_type as usize][6] == key
        }
    }
    #[inline]
    fn is_key_s(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.cfg.input_type as usize][0] == key
    }
    #[inline]
    fn is_key_f(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.cfg.input_type as usize][1] == key
    }
    #[inline]
    fn is_key_r(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.cfg.input_type as usize][2] == key
    }
    #[inline]
    fn is_key_x(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.cfg.input_type as usize][3] == key
    }
    #[inline]
    fn is_key_j(&self, key: u16) -> bool {
        PROCESSING_CHAR[self.cfg.input_type as usize][4] == key
    }

    /// Whether this key is a "word break" key.
    fn is_word_break(&self, event: KeyEvent, data: u16) -> bool {
        if event == KeyEvent::Mouse {
            return true;
        }
        BREAK_CODE.contains(&data)
    }

    fn is_macro_break_code(&self, data: u16) -> bool {
        MACRO_BREAK_CODE.contains(&data)
    }

    fn is_special_key(&self, code: u16) -> bool {
        match self.cfg.input_type {
            InputType::Telex => {
                matches!(
                    code,
                    KEY_W | KEY_E | KEY_R | KEY_O | KEY_LEFT_BRACKET | KEY_RIGHT_BRACKET | KEY_A
                        | KEY_S | KEY_D | KEY_F | KEY_J | KEY_Z | KEY_X
                )
            }
            InputType::Vni => matches!(code, KEY_1 | KEY_2 | KEY_3 | KEY_4 | KEY_5 | KEY_6 | KEY_7 | KEY_8 | KEY_9 | KEY_0),
            InputType::SimpleTelex1 | InputType::SimpleTelex2 => {
                matches!(
                    code,
                    KEY_W | KEY_E | KEY_R | KEY_O | KEY_A | KEY_S | KEY_D | KEY_F | KEY_J | KEY_Z
                        | KEY_X
                )
            }
        }
    }

    fn is_quick_telex_key(&self, code: u16) -> bool {
        self.index > 0
            && is_quick_telex_key(code)
            && self.chr(self.index - 1) == code
    }

    #[inline]
    fn set_key_data(&mut self, index: usize, key_code: u16, is_caps: bool) {
        if index < MAX_BUFF {
            self.typing_word[index] = key_code as u32 | if is_caps { CAPS_MASK } else { 0 };
        }
    }

    /// Insert a key into the typing word; shift-out oldest if full.
    fn insert_key(&mut self, key_code: u16, is_caps: bool, is_check_spelling: bool) {
        if self.index >= MAX_BUFF {
            self.long_word_helper.push(self.typing_word[0]);
            for iii in 0..MAX_BUFF - 1 {
                self.typing_word[iii] = self.typing_word[iii + 1];
            }
            self.set_key_data(self.index - 1, key_code, is_caps);
        } else {
            self.set_key_data(self.index, key_code, is_caps);
            self.index += 1;
        }

        if self.cfg.check_spelling && is_check_spelling {
            self.check_spelling(false);
        }

        // allow d after consonant
        if key_code == KEY_D && self.index >= 2 && is_consonant(self.chr(self.index - 2)) {
            self.temp_disable_key = false;
        }
    }

    fn insert_state(&mut self, key_code: u16, is_caps: bool) {
        if self.state_index >= MAX_BUFF {
            for iii in 0..MAX_BUFF - 1 {
                self.key_states[iii] = self.key_states[iii + 1];
            }
            self.key_states[self.state_index - 1] =
                key_code as u32 | if is_caps { CAPS_MASK } else { 0 };
        } else {
            self.key_states[self.state_index] = key_code as u32 | if is_caps { CAPS_MASK } else { 0 };
            self.state_index += 1;
        }
    }
}

// NOTE: the remaining engine logic (spell checking, mark insertion, vowel
// handling, the main event handler) is large. It is implemented below in
// separate `impl` blocks so each piece can be reviewed against the original.

// ---------------------------------------------------------------------------
// Spell checking (port of checkSpelling)
// ---------------------------------------------------------------------------

impl Engine {
    fn check_spelling(&mut self, force_check_vowel: bool) {
        self.spelling_ok = false;
        self.spelling_vowel_ok = true;
        self.spelling_end_index = self.index;

        if self.index > 0 && self.chr(self.index - 1) == KEY_RIGHT_BRACKET {
            self.spelling_end_index = self.index - 1;
        }

        if self.spelling_end_index > 0 {
            // Check first consonant. C++ reuses the global loop counter `j`
            // both inside the loop and after it, so we must capture the final
            // value of the counter, not use a local one.
            let mut j = 0usize;
            if is_consonant(self.chr(0)) {
                let start_mask = if self.cfg.quick_start_consonant {
                    END_CONSONANT_MASK
                } else {
                    0
                };
                let allow_mask = if self.cfg.allow_consonant_zfwj {
                    CONSONANT_ALLOW_MASK
                } else {
                    0
                };
                'outer: for entry in CONSONANT_TABLE {
                    self.spelling_flag = false;
                    if self.spelling_end_index < entry.len() {
                        self.spelling_flag = true;
                    }
                    j = 0;
                    while j < entry.len() {
                        if self.spelling_end_index > j
                            && (entry[j] & !start_mask) != self.chr(j)
                            && (entry[j] & !allow_mask) != self.chr(j)
                        {
                            self.spelling_flag = true;
                            break;
                        }
                        j += 1;
                    }
                    if self.spelling_flag {
                        continue;
                    }
                    break 'outer;
                }
            }

            if j == self.spelling_end_index {
                // for "d" case
                self.spelling_ok = true;
            }

            // check next vowel
            let mut k = j;
            let mut vowel_start_index = k;

            // fix "que't"
            if self.chr(vowel_start_index) == KEY_U
                && k > 0
                && k < self.spelling_end_index - 1
                && self.chr(vowel_start_index - 1) == KEY_Q
            {
                k += 1;
                j = k;
                vowel_start_index = k;
            } else if self.index >= 2
                && self.chr(0) == KEY_G
                && self.chr(1) == KEY_I
                && is_consonant(self.chr(2))
            {
                // fix gìn
                vowel_start_index = 1;
                k = 1;
                j = 1;
            }

            let mut vowel_end_index = 0usize;
            for _l in 0..3 {
                if k < self.spelling_end_index && !is_consonant(self.chr(k)) {
                    k += 1;
                    vowel_end_index = k;
                }
            }

            if k > j {
                // has vowel
                self.spelling_vowel_ok = false;
                // check correct combined vowel
                if k - j > 1 && force_check_vowel {
                    let vowel_set = vowel_combine_table(self.chr(j));
                    for entry in vowel_set {
                        self.spelling_flag = false;
                        // capture the final `ii` counter like the C++ global
                        let mut ii = 1usize;
                        while ii < entry.len() {
                            if j + ii - 1 < self.spelling_end_index {
                                let w = self.typing_word[j + ii - 1];
                                let expected = (w & TONEW_MASK)
                                    | (w & TONE_MASK)
                                    | (self.chr(j + ii - 1) as u32);
                                if entry[ii] != expected {
                                    self.spelling_flag = true;
                                    break;
                                }
                            }
                            ii += 1;
                        }
                        if self.spelling_flag
                            || (k < self.spelling_end_index && entry[0] == 0)
                            || (j + ii - 1 < self.spelling_end_index
                                && !is_consonant(self.chr(j + ii - 1)))
                        {
                            continue;
                        }
                        self.spelling_vowel_ok = true;
                        break;
                    }
                } else if !is_consonant(self.chr(j)) {
                    self.spelling_vowel_ok = true;
                }

                // continue check last consonant
                for end_entry in END_CONSONANT_TABLE {
                    self.spelling_flag = false;
                    let mut jj = 0usize;
                    while jj < end_entry.len() {
                        if self.spelling_end_index > k + jj {
                            let mask = if self.cfg.quick_end_consonant {
                                END_CONSONANT_MASK
                            } else {
                                0
                            };
                            if (end_entry[jj] & !mask) != self.chr(k + jj) {
                                self.spelling_flag = true;
                                break;
                            }
                        }
                        jj += 1;
                    }
                    if self.spelling_flag {
                        continue;
                    }
                    if k + jj >= self.spelling_end_index {
                        self.spelling_ok = true;
                        break;
                    }
                }

                // limit: end consonant "ch","t" can not use with "~","`","?"
                if self.spelling_ok {
                    if self.index >= 3
                        && self.chr(self.index - 1) == KEY_H
                        && self.chr(self.index - 2) == KEY_C
                        && !((self.typing_word[self.index - 3] & MARK1_MASK) != 0
                            || (self.typing_word[self.index - 3] & MARK5_MASK) != 0
                            || (self.typing_word[self.index - 3] & MARK_MASK) == 0)
                    {
                        self.spelling_ok = false;
                    } else if self.index >= 2
                        && self.chr(self.index - 1) == KEY_T
                        && !((self.typing_word[self.index - 2] & MARK1_MASK) != 0
                            || (self.typing_word[self.index - 2] & MARK5_MASK) != 0
                            || (self.typing_word[self.index - 2] & MARK_MASK) == 0)
                    {
                        self.spelling_ok = false;
                    }
                }
            }
        } else {
            self.spelling_ok = true;
        }

        self.temp_disable_key = !(self.spelling_ok && self.spelling_vowel_ok);
    }
}

// ---------------------------------------------------------------------------
// getCharacterCode — convert a typing word entry into a real character
// ---------------------------------------------------------------------------

impl Engine {
    /// Port of `getCharacterCode`.
    pub fn get_character_code(&mut self, data: u32) -> u32 {
        self.caps_elem = if data & CAPS_MASK != 0 { 0 } else { 1 };
        self.key = data & CHAR_MASK;

        if data & MARK_MASK != 0 {
            // has mark
            self.mark_elem = match data & MARK_MASK {
                MARK1_MASK => 0,
                MARK2_MASK => 2,
                MARK3_MASK => 4,
                MARK4_MASK => 6,
                MARK5_MASK => 8,
                _ => -2,
            };
            self.mark_elem += self.caps_elem as i32;

            match self.key as u16 {
                KEY_A | KEY_O | KEY_U | KEY_E => {
                    if data & TONE_MASK == 0 && data & TONEW_MASK == 0 {
                        self.mark_elem += 4;
                    }
                }
                _ => {}
            }

            // key is a 32-bit lookup key here: mark keys may carry TONE/TONEW
            // (e.g. KEY_A|TONE_MASK) to select the circumflex/breve table.
            if data & TONE_MASK != 0 {
                self.key |= TONE_MASK;
            } else if data & TONEW_MASK != 0 {
                self.key |= TONEW_MASK;
            }

            if let Some(table) = code_table(self.cfg.code_table as usize, self.key) {
                let idx = self.mark_elem as usize;
                if let Some(&c) = table.get(idx) {
                    return c as u32 | CHAR_CODE_MASK;
                }
            }
            return data; // not found
        } else {
            // doesn't have mark
            if let Some(table) = code_table(self.cfg.code_table as usize, self.key) {
                if data & TONE_MASK != 0 {
                    let idx = self.caps_elem;
                    if let Some(&c) = table.get(idx) {
                        return c as u32 | CHAR_CODE_MASK;
                    }
                } else if data & TONEW_MASK != 0 {
                    let idx = self.caps_elem + 2;
                    if let Some(&c) = table.get(idx) {
                        return c as u32 | CHAR_CODE_MASK;
                    }
                }
            }
            return data; // not found
        }
    }
}

// ---------------------------------------------------------------------------
// Vowel finding and mark placement
// ---------------------------------------------------------------------------

impl Engine {
    /// Port of `findAndCalculateVowel`.
    fn find_and_calculate_vowel(&mut self, for_grammar: bool) {
        self.vowel_count = 0;
        self.vowel_start_index = 0;
        self.vowel_end_index = 0;

        let mut iii = self.index;
        while iii > 0 {
            iii -= 1;
            let c = self.chr(iii);
            if is_consonant(c) {
                if self.vowel_count > 0 {
                    break;
                }
            } else {
                // is vowel
                if self.vowel_count == 0 {
                    self.vowel_end_index = iii;
                }
                if !for_grammar {
                    if (iii >= 1 && c == KEY_I && self.chr(iii - 1) == KEY_G)
                        || (iii >= 1 && c == KEY_U && self.chr(iii - 1) == KEY_Q)
                    {
                        break;
                    }
                }
                self.vowel_start_index = iii;
                self.vowel_count += 1;
            }
        }
        // don't count "u" at "q u" as a vowel
        if self.vowel_start_index >= 1
            && self.chr(self.vowel_start_index) == KEY_U
            && self.chr(self.vowel_start_index - 1) == KEY_Q
        {
            self.vowel_start_index += 1;
            self.vowel_count -= 1;
        }
    }

    /// Port of `canHasEndConsonant`.
    fn can_has_end_consonant(&self) -> bool {
        let vo = vowel_combine_table(self.chr(self.vowel_start_index));
        for entry in vo {
            let mut kk = self.vowel_start_index;
            let mut iii = 1usize;
            while iii < entry.len() {
                if kk > self.vowel_end_index
                    || ((self.chr(kk) as u32 | (self.typing_word[kk] & TONE_MASK)
                        | (self.typing_word[kk] & TONEW_MASK)) != entry[iii])
                {
                    break;
                }
                kk += 1;
                iii += 1;
            }
            if iii >= entry.len() {
                return entry[0] == 1;
            }
        }
        false
    }

    /// Port of `handleModernMark` — mark position for modern orthography.
    fn handle_modern_mark(&mut self) {
        // default
        self.vowel_will_set_mark = self.vowel_end_index;
        self.state.backspace_count = (self.index - self.vowel_end_index) as u8;

        // rule 2
        if self.vowel_count == 3
            && ((self.chr(self.vowel_start_index) == KEY_O
                && self.chr(self.vowel_start_index + 1) == KEY_A
                && self.chr(self.vowel_start_index + 2) == KEY_I)
                || (self.chr(self.vowel_start_index) == KEY_U
                    && self.chr(self.vowel_start_index + 1) == KEY_Y
                    && self.chr(self.vowel_start_index + 2) == KEY_U)
                || (self.chr(self.vowel_start_index) == KEY_O
                    && self.chr(self.vowel_start_index + 1) == KEY_E
                    && self.chr(self.vowel_start_index + 2) == KEY_O)
                || (self.chr(self.vowel_start_index) == KEY_U
                    && self.chr(self.vowel_start_index + 1) == KEY_Y
                    && self.chr(self.vowel_start_index + 2) == KEY_A))
        {
            self.vowel_will_set_mark = self.vowel_start_index + 1;
            self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
        } else if self.vowel_count >= 2
            && ((self.chr(self.vowel_start_index) == KEY_O && self.chr(self.vowel_start_index + 1) == KEY_I)
                || (self.chr(self.vowel_start_index) == KEY_A && self.chr(self.vowel_start_index + 1) == KEY_I)
                || (self.chr(self.vowel_start_index) == KEY_U && self.chr(self.vowel_start_index + 1) == KEY_I))
        {
            self.vowel_will_set_mark = self.vowel_start_index;
            self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
        } else if self.vowel_count >= 2
            && self.vowel_end_index >= 1
            && self.chr(self.vowel_end_index - 1) == KEY_A
            && self.chr(self.vowel_end_index) == KEY_Y
        {
            self.vowel_will_set_mark = self.vowel_end_index - 1;
            self.state.backspace_count = (self.index - self.vowel_end_index) as u8 + 1;
        } else if self.vowel_count >= 2
            && self.chr(self.vowel_start_index) == KEY_U
            && self.chr(self.vowel_start_index + 1) == KEY_O
        {
            self.vowel_will_set_mark = self.vowel_start_index + 1;
            self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
        } else if self.vowel_count >= 2
            && (self.chr(self.vowel_start_index + 1) == KEY_O
                || self.chr(self.vowel_start_index + 1) == KEY_U)
        {
            self.vowel_will_set_mark = self.vowel_end_index - 1;
            self.state.backspace_count = (self.index - self.vowel_end_index) as u8 + 1;
        } else if self.vowel_count >= 1
            && (self.chr(self.vowel_start_index) == KEY_O
                || self.chr(self.vowel_start_index) == KEY_U)
        {
            self.vowel_will_set_mark = self.vowel_end_index;
            self.state.backspace_count = (self.index - self.vowel_end_index) as u8;
        }

        // rule 3.1
        if self.vowel_count >= 2
            && ((self.chr(self.vowel_start_index) == KEY_I
                && (self.typing_word[self.vowel_start_index + 1] & (KEY_E as u32 | TONE_MASK)) != 0)
                || (self.chr(self.vowel_start_index) == KEY_Y
                    && (self.typing_word[self.vowel_start_index + 1] & (KEY_E as u32 | TONE_MASK)) != 0)
                || (self.chr(self.vowel_start_index) == KEY_U
                    && self.typing_word[self.vowel_start_index + 1] == (KEY_O as u32 | TONE_MASK))
                || (self.typing_word[self.vowel_start_index] == (KEY_U as u32 | TONEW_MASK)
                    && self.typing_word[self.vowel_start_index + 1] == (KEY_O as u32 | TONEW_MASK)))
        {
            if self.vowel_start_index + 2 < self.index {
                let c2 = self.chr(self.vowel_start_index + 2);
                let has_double = (self.vowel_start_index + 3 < self.index
                    && self.chr(self.vowel_start_index + 2) == KEY_C
                    && self.chr(self.vowel_start_index + 3) == KEY_H)
                    || (self.vowel_start_index + 3 < self.index
                        && self.chr(self.vowel_start_index + 2) == KEY_N
                        && self.chr(self.vowel_start_index + 3) == KEY_H)
                    || (self.vowel_start_index + 3 < self.index
                        && self.chr(self.vowel_start_index + 2) == KEY_N
                        && self.chr(self.vowel_start_index + 3) == KEY_G);
                if matches!(c2, KEY_P | KEY_T | KEY_M | KEY_N | KEY_O | KEY_U | KEY_I | KEY_C)
                    || has_double
                {
                    self.vowel_will_set_mark = self.vowel_start_index + 1;
                    self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
                } else {
                    self.vowel_will_set_mark = self.vowel_start_index;
                    self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
                }
            } else {
                self.vowel_will_set_mark = self.vowel_start_index;
                self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
            }
        }
        // rule 3.2
        else if self.vowel_count >= 2
            && ((self.chr(self.vowel_start_index) == KEY_I
                && self.chr(self.vowel_start_index + 1) == KEY_A)
                || (self.chr(self.vowel_start_index) == KEY_Y
                    && self.chr(self.vowel_start_index + 1) == KEY_A)
                || (self.chr(self.vowel_start_index) == KEY_U
                    && self.chr(self.vowel_start_index + 1) == KEY_A)
                || (self.chr(self.vowel_start_index) == KEY_U
                    && self.typing_word[self.vowel_start_index + 1] == (KEY_U as u32 | TONEW_MASK)))
        {
            self.vowel_will_set_mark = self.vowel_start_index;
            self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
        }

        // rule 4
        if self.vowel_count == 2 {
            if (self.chr(self.vowel_start_index) == KEY_I
                && self.chr(self.vowel_start_index + 1) == KEY_A)
                || (self.chr(self.vowel_start_index) == KEY_I
                    && self.chr(self.vowel_start_index + 1) == KEY_U)
                || (self.chr(self.vowel_start_index) == KEY_I
                    && self.chr(self.vowel_start_index + 1) == KEY_O)
            {
                if self.vowel_start_index == 0
                    || self.chr(self.vowel_start_index - 1) != KEY_G
                {
                    // don't have G
                    self.vowel_will_set_mark = self.vowel_start_index;
                    self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
                } else {
                    self.vowel_will_set_mark = self.vowel_start_index + 1;
                    self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
                }
            } else if self.chr(self.vowel_start_index) == KEY_U
                && self.chr(self.vowel_start_index + 1) == KEY_A
            {
                if self.vowel_start_index == 0
                    || self.chr(self.vowel_start_index - 1) != KEY_Q
                {
                    // don't have Q
                    if self.vowel_end_index + 1 >= self.index || !self.can_has_end_consonant() {
                        self.vowel_will_set_mark = self.vowel_start_index;
                        self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
                    }
                } else {
                    self.vowel_will_set_mark = self.vowel_start_index + 1;
                    self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
                }
            } else if self.chr(self.vowel_start_index) == KEY_O
                && self.chr(self.vowel_start_index + 1) == KEY_O
            {
                // thoong
                self.vowel_will_set_mark = self.vowel_end_index;
                self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
            }
        }
    }

    /// Port of `handleOldMark` — mark position for old orthography.
    fn handle_old_mark(&mut self) {
        // default
        if self.vowel_count == 0 && self.chr(self.vowel_end_index) == KEY_I {
            self.vowel_will_set_mark = self.vowel_end_index;
        } else {
            self.vowel_will_set_mark = self.vowel_start_index;
        }
        self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;

        // rule 2
        if self.vowel_count == 3
            || (self.vowel_end_index + 1 < self.index
                && is_consonant(self.chr(self.vowel_end_index + 1))
                && self.can_has_end_consonant())
        {
            self.vowel_will_set_mark = self.vowel_start_index + 1;
            self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
        }

        // rule 3
        for ii in self.vowel_start_index..=self.vowel_end_index {
            if (self.chr(ii) == KEY_E && (self.typing_word[ii] & TONE_MASK) != 0)
                || (self.chr(ii) == KEY_O && (self.typing_word[ii] & TONEW_MASK) != 0)
            {
                self.vowel_will_set_mark = ii;
                self.state.backspace_count = (self.index - self.vowel_will_set_mark) as u8;
                break;
            }
        }

        self.state.new_char_count = self.state.backspace_count;
    }

    /// Port of `insertMark` — place (or toggle) a mark on the vowel.
    fn insert_mark(&mut self, mark_mask: u32, can_modify_flag: bool) {
        self.vowel_count = 0;

        if can_modify_flag {
            self.state.code = HookCode::WillProcess;
        }
        self.state.backspace_count = 0;
        self.state.new_char_count = 0;

        self.find_and_calculate_vowel(false);
        self.vowel_will_set_mark = 0;

        if self.vowel_count == 1 {
            self.vowel_will_set_mark = self.vowel_end_index;
            self.state.backspace_count = (self.index - self.vowel_end_index) as u8;
        } else {
            // vowel = 2 or 3
            if !self.cfg.use_modern_orthography {
                self.handle_old_mark();
            } else {
                self.handle_modern_mark();
            }
            if (self.typing_word[self.vowel_end_index] & TONE_MASK) != 0
                || (self.typing_word[self.vowel_end_index] & TONEW_MASK) != 0
            {
                self.vowel_will_set_mark = self.vowel_end_index;
            }
        }

        // send data
        let mut kk = self.index - 1 - self.vowel_start_index;
        // if duplicate same mark -> restore
        if (self.typing_word[self.vowel_will_set_mark] & mark_mask) != 0 {
            self.typing_word[self.vowel_will_set_mark] &= !MARK_MASK;
            if can_modify_flag {
                self.state.code = HookCode::Restore;
            }
            for ii in self.vowel_start_index..self.index {
                self.typing_word[ii] &= !MARK_MASK;
                if kk < MAX_BUFF {
                    self.state.char_data[kk] = self.get_character_code(self.typing_word[ii]);
                }
                kk = kk.wrapping_sub(1);
            }
            self.temp_disable_key = true;
        } else {
            // remove other mark
            self.typing_word[self.vowel_will_set_mark] &= !MARK_MASK;

            // add mark
            self.typing_word[self.vowel_will_set_mark] |= mark_mask;
            for ii in self.vowel_start_index..self.index {
                if ii != self.vowel_will_set_mark {
                    self.typing_word[ii] &= !MARK_MASK;
                }
                if kk < MAX_BUFF {
                    self.state.char_data[kk] = self.get_character_code(self.typing_word[ii]);
                }
                kk = kk.wrapping_sub(1);
            }

            self.state.backspace_count = (self.index - self.vowel_start_index) as u8;
        }
        self.state.new_char_count = self.state.backspace_count;
    }
}


// ---------------------------------------------------------------------------
// insertD / insertAOE / insertW — tone insertion helpers
// ---------------------------------------------------------------------------

impl Engine {
    /// Port of `insertD` — handles the "đ" key (KEY_D).
    fn insert_d(&mut self, is_caps: bool) {
        self.state.code = HookCode::WillProcess;
        self.state.backspace_count = 0;
        let mut ii = self.index;
        while ii > 0 {
            ii -= 1;
            self.state.backspace_count += 1;
            if self.chr(ii) == KEY_D {
                if (self.typing_word[ii] & TONE_MASK) != 0 {
                    // restore and disable temporary
                    self.state.code = HookCode::Restore;
                    self.typing_word[ii] &= !TONE_MASK;
                    let idx = self.index - 1 - ii;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.typing_word[ii];
                    }
                    self.temp_disable_key = true;
                    break;
                } else {
                    self.typing_word[ii] |= TONE_MASK;
                    let idx = self.index - 1 - ii;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.get_character_code(self.typing_word[ii]);
                    }
                }
                break;
            } else {
                // represent old char
                let idx = self.index - 1 - ii;
                if idx < MAX_BUFF {
                    self.state.char_data[idx] = self.get_character_code(self.typing_word[ii]);
                }
            }
        }
        self.state.new_char_count = self.state.backspace_count;
    }

    /// Port of `insertAOE` — the "double" vowel key (a,o,e hats).
    fn insert_aoe(&mut self, data: u16, is_caps: bool) {
        self.find_and_calculate_vowel(false);

        // remove W tone
        for ii in self.vowel_start_index..=self.vowel_end_index {
            self.typing_word[ii] &= !TONEW_MASK;
        }

        self.state.code = HookCode::WillProcess;
        self.state.backspace_count = 0;

        let mut ii = self.index;
        while ii > 0 {
            ii -= 1;
            self.state.backspace_count += 1;
            if self.chr(ii) == data {
                // reverse unicode char
                if (self.typing_word[ii] & TONE_MASK) != 0 {
                    // restore and disable temporary
                    self.state.code = HookCode::Restore;
                    self.typing_word[ii] &= !TONE_MASK;
                    let idx = self.index - 1 - ii;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.typing_word[ii];
                    }
                    if data != KEY_O {
                        // case thoòng
                        self.temp_disable_key = true;
                    }
                    break;
                } else {
                    self.typing_word[ii] |= TONE_MASK;
                    if !self.is_key_d(data) {
                        self.typing_word[ii] &= !TONEW_MASK;
                    }
                    let idx = self.index - 1 - ii;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.get_character_code(self.typing_word[ii]);
                    }
                }
                break;
            } else {
                // represent old char
                let idx = self.index - 1 - ii;
                if idx < MAX_BUFF {
                    self.state.char_data[idx] = self.get_character_code(self.typing_word[ii]);
                }
            }
        }
        self.state.new_char_count = self.state.backspace_count;
    }

    /// Port of `insertW` — the "w" tone key.
    fn insert_w(&mut self, data: u16, is_caps: bool) {
        self.is_restored_w = false;

        self.find_and_calculate_vowel(false);

        // remove ^ tone
        for ii in self.vowel_start_index..=self.vowel_end_index {
            self.typing_word[ii] &= !TONE_MASK;
        }

        if self.vowel_count > 1 {
            self.state.backspace_count = (self.index - self.vowel_start_index) as u8;
            self.state.new_char_count = self.state.backspace_count;

            if ((self.typing_word[self.vowel_start_index] & TONEW_MASK) != 0
                && (self.typing_word[self.vowel_start_index + 1] & TONEW_MASK) != 0)
                || ((self.typing_word[self.vowel_start_index] & TONEW_MASK) != 0
                    && self.chr(self.vowel_start_index + 1) == KEY_I)
                || ((self.typing_word[self.vowel_start_index] & TONEW_MASK) != 0
                    && self.chr(self.vowel_start_index + 1) == KEY_A)
            {
                // restore and disable temporary
                self.state.code = HookCode::Restore;
                for ii in self.vowel_start_index..self.index {
                    self.typing_word[ii] &= !TONEW_MASK;
                    let idx = self.index - 1 - ii;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] =
                            self.get_character_code(self.typing_word[ii]) & !STANDALONE_MASK;
                    }
                }
                self.is_restored_w = true;
                self.temp_disable_key = true;
            } else {
                self.state.code = HookCode::WillProcess;

                if self.chr(self.vowel_start_index) == KEY_U
                    && self.chr(self.vowel_start_index + 1) == KEY_O
                {
                    if self.vowel_start_index >= 2
                        && self.typing_word[self.vowel_start_index - 2] == KEY_T as u32
                        && self.typing_word[self.vowel_start_index - 1] == KEY_H as u32
                    {
                        self.typing_word[self.vowel_start_index + 1] |= TONEW_MASK;
                        if self.vowel_start_index + 2 < self.index
                            && self.chr(self.vowel_start_index + 2) == KEY_N
                        {
                            self.typing_word[self.vowel_start_index] |= TONEW_MASK;
                        }
                    } else if self.vowel_start_index >= 1
                        && self.typing_word[self.vowel_start_index - 1] == KEY_Q as u32
                    {
                        self.typing_word[self.vowel_start_index + 1] |= TONEW_MASK;
                    } else {
                        self.typing_word[self.vowel_start_index] |= TONEW_MASK;
                        self.typing_word[self.vowel_start_index + 1] |= TONEW_MASK;
                    }
                } else if (self.chr(self.vowel_start_index) == KEY_U
                    && self.chr(self.vowel_start_index + 1) == KEY_A)
                    || (self.chr(self.vowel_start_index) == KEY_U
                        && self.chr(self.vowel_start_index + 1) == KEY_I)
                    || (self.chr(self.vowel_start_index) == KEY_U
                        && self.chr(self.vowel_start_index + 1) == KEY_U)
                    || (self.chr(self.vowel_start_index) == KEY_O
                        && self.chr(self.vowel_start_index + 1) == KEY_I)
                {
                    self.typing_word[self.vowel_start_index] |= TONEW_MASK;
                } else if (self.chr(self.vowel_start_index) == KEY_I
                    && self.chr(self.vowel_start_index + 1) == KEY_O)
                    || (self.chr(self.vowel_start_index) == KEY_O
                        && self.chr(self.vowel_start_index + 1) == KEY_A)
                {
                    self.typing_word[self.vowel_start_index + 1] |= TONEW_MASK;
                } else {
                    // don't do anything
                    self.temp_disable_key = true;
                    self.is_changed = false;
                    self.state.code = HookCode::DoNothing;
                }

                for ii in self.vowel_start_index..self.index {
                    let idx = self.index - 1 - ii;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.get_character_code(self.typing_word[ii]);
                    }
                }
            }

            return;
        }

        self.state.code = HookCode::WillProcess;
        self.state.backspace_count = 0;

        let mut ii = self.index;
        while ii > 0 {
            ii -= 1;
            if ii < self.vowel_start_index {
                break;
            }
            self.state.backspace_count += 1;
            match self.chr(ii) {
                KEY_A | KEY_U | KEY_O => {
                    if (self.typing_word[ii] & TONEW_MASK) != 0 {
                        // restore and disable temporary
                        if (self.typing_word[ii] & STANDALONE_MASK) != 0 {
                            self.state.code = HookCode::WillProcess;
                            if self.chr(ii) == KEY_U {
                                self.typing_word[ii] =
                                    KEY_W as u32 | if (self.typing_word[ii] & CAPS_MASK) != 0 { CAPS_MASK } else { 0 };
                            } else if self.chr(ii) == KEY_O {
                                self.state.code = HookCode::Restore;
                                self.typing_word[ii] =
                                    KEY_O as u32 | if (self.typing_word[ii] & CAPS_MASK) != 0 { CAPS_MASK } else { 0 };
                                self.is_restored_w = true;
                            }
                            let idx = self.index - 1 - ii;
                            if idx < MAX_BUFF {
                                self.state.char_data[idx] = self.typing_word[ii];
                            }
                        } else {
                            self.state.code = HookCode::Restore;
                            self.typing_word[ii] &= !TONEW_MASK;
                            let idx = self.index - 1 - ii;
                            if idx < MAX_BUFF {
                                self.state.char_data[idx] = self.typing_word[ii];
                            }
                            self.is_restored_w = true;
                        }

                        self.temp_disable_key = true;
                    } else {
                        self.typing_word[ii] |= TONEW_MASK;
                        self.typing_word[ii] &= !TONE_MASK;
                        let idx = self.index - 1 - ii;
                        if idx < MAX_BUFF {
                            self.state.char_data[idx] = self.get_character_code(self.typing_word[ii]);
                        }
                    }
                }
                _ => {
                    let idx = self.index - 1 - ii;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.get_character_code(self.typing_word[ii]);
                    }
                }
            }
        }
        self.state.new_char_count = self.state.backspace_count;
    }

    /// Port of `reverseLastStandaloneChar`.
    fn reverse_last_standalone_char(&mut self, key_code: u16, is_caps: bool) {
        self.state.code = HookCode::WillProcess;
        self.state.backspace_count = 0;
        self.state.new_char_count = 1;
        self.state.ext_code = ExtCode::NoEmptyChar;
        self.typing_word[self.index - 1] = key_code as u32
            | TONEW_MASK
            | STANDALONE_MASK
            | if is_caps { CAPS_MASK } else { 0 };
        self.state.char_data[0] = self.get_character_code(self.typing_word[self.index - 1]);
    }

    /// Port of `checkForStandaloneChar` — the `w`, `[`, `]` standalone keys.
    fn check_for_standalone_char(&mut self, data: u16, is_caps: bool, key_will_reverse: u16) {
        if self.index > 0
            && self.chr(self.index - 1) == key_will_reverse
            && (self.typing_word[self.index - 1] & TONEW_MASK) != 0
        {
            self.state.code = HookCode::WillProcess;
            self.state.backspace_count = 1;
            self.state.new_char_count = 1;
            self.typing_word[self.index - 1] = data as u32 | if is_caps { CAPS_MASK } else { 0 };
            self.state.char_data[0] = self.get_character_code(self.typing_word[self.index - 1]);
            return;
        }

        // check standalone w -> ư
        if self.index > 0
            && self.chr(self.index - 1) == KEY_U
            && key_will_reverse == KEY_O
        {
            self.insert_key(key_will_reverse, is_caps, true);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        }

        if self.index == 0 {
            // zero char
            self.insert_key(data, is_caps, false);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        } else if self.index == 1 {
            // 1 char
            for &bad in STANDALONE_W_BAD {
                if self.chr(0) == bad {
                    self.insert_key(data, is_caps, true);
                    return;
                }
            }
            self.insert_key(data, is_caps, false);
            self.reverse_last_standalone_char(key_will_reverse, is_caps);
            return;
        } else if self.index == 2 {
            for allowed in DOUBLE_W_ALLOWED {
                if self.chr(0) == allowed[0] && self.chr(1) == allowed[1] {
                    self.insert_key(data, is_caps, false);
                    self.reverse_last_standalone_char(key_will_reverse, is_caps);
                    return;
                }
            }
            self.insert_key(data, is_caps, true);
            return;
        }

        self.insert_key(data, is_caps, true);
    }
}

// ---------------------------------------------------------------------------
// checkGrammar — fix spelling when typing continues
// ---------------------------------------------------------------------------

impl Engine {
    fn check_grammar(&mut self, delta_back_space: i32) {
        if self.index <= 1 || self.index >= MAX_BUFF {
            return;
        }

        self.find_and_calculate_vowel(true);
        if self.vowel_count == 0 {
            return;
        }

        self.is_checked_grammar = false;

        let l = self.vowel_start_index;

        // if N key for case: "thuơn", "ưoi", "ưom", "ưoc"
        if self.index >= 3 {
            for i in (0..self.index).rev() {
                let c = self.chr(i);
                if matches!(c, KEY_N | KEY_C | KEY_I | KEY_M | KEY_P | KEY_T) {
                    if i >= 2 && self.chr(i - 1) == KEY_O && self.chr(i - 2) == KEY_U {
                        if (self.typing_word[i - 1] & TONEW_MASK)
                            != (self.typing_word[i - 2] & TONEW_MASK)
                        {
                            self.typing_word[i - 2] |= TONEW_MASK;
                            self.typing_word[i - 1] |= TONEW_MASK;
                            self.is_checked_grammar = true;
                            break;
                        }
                    }
                }
            }
        }

        // check mark
        if self.index >= 2 {
            for i in l..=self.vowel_end_index {
                if (self.typing_word[i] & MARK_MASK) != 0 {
                    let mark = self.typing_word[i] & MARK_MASK;
                    self.typing_word[i] &= !MARK_MASK;
                    self.insert_mark(mark, false);
                    if i != self.vowel_will_set_mark {
                        self.is_checked_grammar = true;
                    }
                    break;
                }
            }
        }

        // re-arrange data to sendback
        if self.is_checked_grammar {
            if self.state.code == HookCode::DoNothing {
                self.state.code = HookCode::WillProcess;
            }
            self.state.backspace_count = 0;

            for i in (l..self.index).rev() {
                self.state.backspace_count += 1;
                let idx = self.index - 1 - i;
                if idx < MAX_BUFF {
                    self.state.char_data[idx] = self.get_character_code(self.typing_word[i]);
                }
            }
            self.state.new_char_count = self.state.backspace_count;
            self.state.backspace_count = (self.state.backspace_count as i32 + delta_back_space) as u8;
            self.state.ext_code = ExtCode::NoEmptyChar;
        }
    }
}

// ---------------------------------------------------------------------------
// handleMainKey — dispatch for special keys
// ---------------------------------------------------------------------------

impl Engine {
    /// Port of `handleMainKey`.
    fn handle_main_key(&mut self, data: u16, is_caps: bool) {
        // if is Z key, remove mark
        if self.is_key_z(data) {
            self.remove_mark();
            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        if data == KEY_LEFT_BRACKET {
            // standalone key [
            self.check_for_standalone_char(data, is_caps, KEY_O);
            return;
        }

        if data == KEY_RIGHT_BRACKET {
            // standalone key ]
            self.check_for_standalone_char(data, is_caps, KEY_U);
            return;
        }

        // if is D key
        if self.is_key_d(data) {
            self.is_correct = false;
            self.is_changed = false;
            self.k = self.index;
            for entry in CONSONANT_D {
                if self.index < entry.len() {
                    continue;
                }
                self.is_correct = true;
                self.check_correct_vowel(entry, data);

                // allow d after consonant
                if !self.is_correct
                    && self.index >= 2
                    && self.chr(self.index - 1) == KEY_D
                    && is_consonant(self.chr(self.index - 2))
                {
                    self.is_correct = true;
                }
                if self.is_correct {
                    self.is_changed = true;
                    self.insert_d(is_caps);
                    break;
                }
            }

            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }
            return;
        }

        // if is mark key — the original iterates ALL _vowelForMark entries
        if is_mark_key(self.cfg.input_type, data) {
            let all_entries = [
                vowel_for_mark_table(KEY_A),
                vowel_for_mark_table(KEY_O),
                vowel_for_mark_table(KEY_E),
                vowel_for_mark_table(KEY_I),
                vowel_for_mark_table(KEY_U),
                vowel_for_mark_table(KEY_Y),
            ];
            self.is_correct = false;
            self.is_changed = false;
            self.k = self.index;
            'outer: for entries in all_entries {
                for charset in entries {
                    if self.index < charset.len() {
                        continue;
                    }
                    self.is_correct = true;
                    self.check_correct_vowel(charset, data);

                    if self.is_correct {
                        self.is_changed = true;
                        if self.is_key_s(data) {
                            self.insert_mark(MARK1_MASK, true);
                        } else if self.is_key_f(data) {
                            self.insert_mark(MARK2_MASK, true);
                        } else if self.is_key_r(data) {
                            self.insert_mark(MARK3_MASK, true);
                        } else if self.is_key_x(data) {
                            self.insert_mark(MARK4_MASK, true);
                        } else if self.is_key_j(data) {
                            self.insert_mark(MARK5_MASK, true);
                        }
                        break 'outer;
                    }
                }
            }

            if !self.is_changed {
                self.insert_key(data, is_caps, true);
            }

            return;
        }

        // check Vowel
        if self.cfg.input_type == InputType::Vni {
            for i in (0..self.index).rev() {
                let c = self.chr(i);
                if c == KEY_O || c == KEY_A || c == KEY_E {
                    self.vowel_end_index = i;
                    break;
                }
            }
        }

        self.key_for_aeo = if self.cfg.input_type != InputType::Vni {
            data
        } else {
            match data {
                KEY_7 | KEY_8 => KEY_W,
                KEY_6 => self.chr(self.vowel_end_index),
                _ => data,
            }
        };

        let charset = vowel_table(self.key_for_aeo);
        self.is_correct = false;
        self.is_changed = false;
        self.k = self.index;
        for entry in charset {
            if self.index < entry.len() {
                continue;
            }
            self.is_correct = true;
            self.check_correct_vowel(entry, data);

            if self.is_correct {
                self.is_changed = true;
                if self.is_key_double(data) {
                    self.insert_aoe(self.key_for_aeo, is_caps);
                } else if self.is_key_w(data) {
                    if self.cfg.input_type == InputType::Vni {
                        for j in (0..self.index).rev() {
                            let c = self.chr(j);
                            if c == KEY_O || c == KEY_U || c == KEY_A || c == KEY_E {
                                self.vowel_end_index = j;
                                break;
                            }
                        }
                        let cond7 = data == KEY_7
                            && self.chr(self.vowel_end_index) == KEY_A
                            && (if self.vowel_end_index >= 1 {
                                self.chr(self.vowel_end_index - 1) != KEY_U
                            } else {
                                true
                            });
                        let cond8 = data == KEY_8
                            && (self.chr(self.vowel_end_index) == KEY_O
                                || self.chr(self.vowel_end_index) == KEY_U);
                        if cond7 || cond8
                        {
                            break;
                        }
                    }
                    self.insert_w(self.key_for_aeo, is_caps);
                }
                break;
            }
        }

        if !self.is_changed {
            if data == KEY_W && self.cfg.input_type != InputType::SimpleTelex1 {
                self.check_for_standalone_char(data, is_caps, KEY_U);
            } else {
                self.insert_key(data, is_caps, true);
            }
        }
    }

    /// Port of `checkCorrectVowel` (matches current word tail against charset).
    fn check_correct_vowel(&mut self, charset: &[u16], mark_key: u16) {
        // ignore "qu" case
        if self.index >= 2
            && self.chr(self.index - 1) == KEY_U
            && self.chr(self.index - 2) == KEY_Q
        {
            self.is_correct = false;
            return;
        }
        self.k = self.index - 1;
        for j in (0..charset.len()).rev() {
            let mut mask = 0u16;
            if self.cfg.quick_end_consonant {
                mask |= END_CONSONANT_MASK;
            }
            if (charset[j] & !mask) != self.chr(self.k) {
                self.is_correct = false;
                return;
            }
            if self.k == 0 {
                self.k = 0;
                break;
            }
            self.k -= 1;
        }

        // limit mark for end consonant: "C", "T"
        if self.is_correct
            && charset.len() > 1
            && (self.is_key_f(mark_key) || self.is_key_x(mark_key) || self.is_key_r(mark_key))
        {
            if charset[1] == KEY_C || charset[1] == KEY_T {
                self.is_correct = false;
            } else if charset.len() > 2 && charset[2] == KEY_T {
                self.is_correct = false;
            }
        }

        if self.is_correct && self.k < self.index && self.chr(self.k) == self.chr(self.k + 1) {
            self.is_correct = false;
        }
    }

    /// Port of `removeMark`.
    fn remove_mark(&mut self) {
        self.find_and_calculate_vowel(true);
        self.is_changed = false;
        if self.index > 0 {
            for i in self.vowel_start_index..=self.vowel_end_index {
                if (self.typing_word[i] & MARK_MASK) != 0 {
                    self.typing_word[i] &= !MARK_MASK;
                    self.is_changed = true;
                }
            }
        }
        if self.is_changed {
            self.state.code = HookCode::WillProcess;
            self.state.backspace_count = 0;

            for i in (self.vowel_start_index..self.index).rev() {
                self.state.backspace_count += 1;
                let idx = self.index - 1 - i;
                if idx < MAX_BUFF {
                    self.state.char_data[idx] = self.get_character_code(self.typing_word[i]);
                }
            }
            self.state.new_char_count = self.state.backspace_count;
        } else {
            self.state.code = HookCode::DoNothing;
        }
    }
}

// ---------------------------------------------------------------------------
// Session management + main event handler
// ---------------------------------------------------------------------------

impl Engine {
    /// Port of `startNewSession`.
    pub fn start_new_session(&mut self) {
        self.index = 0;
        self.state.backspace_count = 0;
        self.state.new_char_count = 0;
        self.temp_disable_key = false;
        self.state_index = 0;
        self.has_handled_macro = false;
        self.has_handle_quick_consonant = false;
        self.long_word_helper.clear();
    }

    /// Port of `vKeyInit`.
    pub fn reset(&mut self) {
        self.index = 0;
        self.state_index = 0;
        self.use_spell_checking_before = self.cfg.check_spelling;
        self.typing_states.clear();
        self.long_word_helper.clear();
        self.state = HookState::default();
    }

    /// The main entry point — port of `vKeyHandleEvent`.
    ///
    /// `caps_status`: 0 = none, 1 = shift, 2 = caps lock.
    pub fn handle_key(
        &mut self,
        event: KeyEvent,
        state: KeyEventState,
        data: u16,
        caps_status: u8,
        other_control_key: bool,
    ) -> &HookState {
        self.is_caps = caps_status == 1 || caps_status == 2;

        let is_number_with_shift = is_number_key(data) && caps_status == 1;

        if is_number_with_shift
            || other_control_key
            || self.is_word_break(event, data)
            || (self.index == 0 && is_number_key(data))
        {
            self.state.code = HookCode::DoNothing;
            self.state.backspace_count = 0;
            self.state.new_char_count = 0;
            self.state.ext_code = ExtCode::WordBreak;

            // check macro feature
            if self.cfg.use_macro
                && self.is_macro_break_code(data)
                && !self.has_handled_macro
            {
                // macro matching is handled in the front-end via macro_data;
                // here we only mark the flag. (Full macro port in Macro module.)
                if !self.state.macro_key.is_empty() {
                    self.state.code = HookCode::ReplaceMacro;
                    self.state.backspace_count = self.state.macro_key.len() as u8;
                    self.has_handled_macro = true;
                }
            } else if (self.cfg.quick_start_consonant || self.cfg.quick_end_consonant)
                && !self.temp_disable_key
                && self.is_macro_break_code(data)
            {
                self.check_quick_consonant();
            } else if self.cfg.restore_if_wrong_spelling
                && self.is_word_break(event, data)
            {
                // restore key if wrong spelling with break-key
                if !self.temp_disable_key && self.cfg.check_spelling {
                    self.check_spelling(true); // force check spelling
                }
                if self.temp_disable_key && !self.check_restore_if_wrong_spelling() {
                    self.state.code = HookCode::DoNothing;
                }
            }

            let is_char_key_code = state == KeyEventState::KeyDown
                && CHAR_KEY_CODE.contains(&data);
            if !is_char_key_code {
                // clear all line cache
                self.special_char.clear();
                self.typing_states.clear();
            } else {
                // check and save current word
                if self.space_count > 0 {
                    self.save_word_space(self.space_count);
                    self.space_count = 0;
                } else {
                    self.save_word();
                }
                self.special_char.push(data as u32 | if self.is_caps { CAPS_MASK } else { 0 });
                self.state.ext_code = ExtCode::NormalKey;
            }

            if self.state.code == HookCode::DoNothing {
                self.start_new_session();
                self.cfg.check_spelling = self.use_spell_checking_before;
                self.will_temp_off_engine = false;
            } else if self.state.code == HookCode::ReplaceMacro || self.has_handle_quick_consonant {
                self.index = 0;
            }

            // insert key for macro function
            if self.cfg.use_macro {
                if is_char_key_code {
                    self.state.macro_key.push(data as u32 | if self.is_caps { CAPS_MASK } else { 0 });
                } else {
                    self.state.macro_key.clear();
                }
            }

            if self.cfg.upper_case_first_char {
                if data == KEY_DOT {
                    self.upper_case_status = 1;
                } else if data == KEY_ENTER || data == KEY_RETURN {
                    self.upper_case_status = 2;
                } else {
                    self.upper_case_status = 0;
                }
            }
        } else if data == KEY_SPACE {
            if !self.temp_disable_key && self.cfg.check_spelling {
                self.check_spelling(true);
            }
            if self.cfg.use_macro && !self.has_handled_macro && !self.state.macro_key.is_empty() {
                self.state.code = HookCode::ReplaceMacro;
                self.state.backspace_count = self.state.macro_key.len() as u8;
                self.space_count += 1;
                self.has_handled_macro = true;
            } else if (self.cfg.quick_start_consonant || self.cfg.quick_end_consonant)
                && !self.temp_disable_key
                && self.check_quick_consonant()
            {
                self.space_count += 1;
            } else if self.cfg.restore_if_wrong_spelling
                && self.temp_disable_key
                && !self.has_handled_macro
            {
                // restore key if wrong spelling
                if !self.check_restore_if_wrong_spelling() {
                    self.state.code = HookCode::DoNothing;
                }
                self.space_count += 1;
            } else {
                // do nothing with SPACE KEY
                self.state.code = HookCode::DoNothing;
                self.space_count += 1;
            }
            if self.cfg.use_macro {
                self.state.macro_key.clear();
            }
            if self.cfg.upper_case_first_char && self.upper_case_status == 1 {
                self.upper_case_status = 2;
            }
            // save word
            if self.space_count == 1 {
                if !self.special_char.is_empty() {
                    self.save_special_char();
                } else {
                    self.save_word();
                }
            }
            self.cfg.check_spelling = self.use_spell_checking_before;
            self.will_temp_off_engine = false;
        } else if data == KEY_DELETE {
            self.state.code = HookCode::DoNothing;
            self.state.ext_code = ExtCode::Delete;
            if !self.special_char.is_empty() {
                self.special_char.pop();
                if self.special_char.is_empty() {
                    self.restore_last_typing_state();
                }
            } else if self.space_count > 0 {
                // previous char is space
                self.space_count -= 1;
                if self.space_count == 0 {
                    self.restore_last_typing_state();
                }
            } else {
                if self.state_index > 0 {
                    self.state_index -= 1;
                }
                if self.index > 0 {
                    self.index -= 1;
                    if !self.long_word_helper.is_empty() {
                        // right shift
                        for i in (1..MAX_BUFF).rev() {
                            self.typing_word[i] = self.typing_word[i - 1];
                        }
                        self.typing_word[0] = *self.long_word_helper.last().unwrap();
                        self.long_word_helper.pop();
                        self.index += 1;
                    }
                    if self.cfg.check_spelling {
                        self.check_spelling(false);
                    }
                }
                if self.cfg.use_macro && !self.state.macro_key.is_empty() {
                    self.state.macro_key.pop();
                }

                self.state.backspace_count = 0;
                self.state.new_char_count = 0;
                self.state.ext_code = ExtCode::Delete;
                if self.index == 0 {
                    self.start_new_session();
                    self.special_char.clear();
                    self.restore_last_typing_state();
                } else {
                    // continue check grammar
                    self.check_grammar(1);
                }
            }
        } else {
            // START AND CHECK KEY
            if self.will_temp_off_engine {
                self.state.code = HookCode::DoNothing;
                self.state.ext_code = ExtCode::NormalKey;
                return &self.state;
            }
            if self.space_count > 0 {
                self.state.backspace_count = 0;
                self.state.new_char_count = 0;
                self.state.ext_code = ExtCode::WordBreak;
                self.start_new_session();
                // continue save space
                self.save_word_space(self.space_count);
                self.space_count = 0;
            } else if !self.special_char.is_empty() {
                self.save_special_char();
            }

            self.insert_state(data, self.is_caps);

            if !self.is_special_key(data) || self.temp_disable_key {
                // do nothing
                if self.cfg.quick_telex && self.is_quick_telex_key(data) {
                    self.handle_quick_telex(data, self.is_caps);
                    return &self.state;
                } else {
                    self.state.code = HookCode::DoNothing;
                    self.state.backspace_count = 0;
                    self.state.new_char_count = 0;
                    self.state.ext_code = ExtCode::NormalKey;
                    self.insert_key(data, self.is_caps, true);
                }
            } else {
                // check and update key
                self.state.code = HookCode::DoNothing;
                self.state.ext_code = ExtCode::NormalKey;
                self.handle_main_key(data, self.is_caps);
            }

            if !self.cfg.free_mark && !self.is_key_d(data) {
                if self.state.code == HookCode::DoNothing {
                    self.check_grammar(-1);
                } else {
                    self.check_grammar(0);
                }
            }

            if self.state.code == HookCode::Restore {
                self.insert_key(data, self.is_caps, true);
                if self.state_index > 0 {
                    self.state_index -= 1;
                }
            }

            // insert or replace key for macro feature
            if self.cfg.use_macro {
                if self.state.code == HookCode::DoNothing {
                    self.state.macro_key.push(data as u32 | if self.is_caps { CAPS_MASK } else { 0 });
                } else if self.state.code == HookCode::WillProcess
                    || self.state.code == HookCode::Restore
                {
                    let bpc = self.state.backspace_count as usize;
                    for _ in 0..bpc {
                        if !self.state.macro_key.is_empty() {
                            self.state.macro_key.pop();
                        }
                    }
                    let start = self.index.saturating_sub(bpc);
                    for i in start..(start + self.state.new_char_count as usize).min(MAX_BUFF) {
                        self.state.macro_key.push(self.typing_word[i]);
                    }
                }
            }

            if self.cfg.upper_case_first_char {
                if self.index == 1 && self.upper_case_status == 2 {
                    self.upper_case_first_character();
                }
                self.upper_case_status = 0;
            }

            // case [ ]
            if (data == KEY_LEFT_BRACKET || data == KEY_RIGHT_BRACKET)
                && ((self.state.char_data[0] as u16 == KEY_LEFT_BRACKET
                    || self.state.char_data[0] as u16 == KEY_RIGHT_BRACKET)
                    || self.cfg.input_type == InputType::SimpleTelex1
                    || self.cfg.input_type == InputType::SimpleTelex2)
            {
                let subtract = if self.state.code == HookCode::WillProcess {
                    self.state.backspace_count as usize
                } else {
                    0
                };
                if self.index > subtract {
                    self.index -= 1;
                    self.save_word();
                }
                self.index = 0;
                self.temp_disable_key = false;
                self.state_index = 0;
                self.state.ext_code = ExtCode::NormalKey;
                self.special_char.push(data as u32 | if self.is_caps { CAPS_MASK } else { 0 });
            }
        }

        &self.state
    }

    fn save_word(&mut self) {
        if self.state.code != HookCode::ReplaceMacro {
            if self.index > 0 {
                if !self.long_word_helper.is_empty() {
                    self.typing_states_data.clear();
                    for (i, &w) in self.long_word_helper.iter().enumerate() {
                        if i != 0 && i % MAX_BUFF == 0 {
                            self.typing_states.push(self.typing_states_data.clone());
                            self.typing_states_data.clear();
                        }
                        self.typing_states_data.push(w);
                    }
                    self.typing_states.push(self.typing_states_data.clone());
                    self.long_word_helper.clear();
                }
                self.typing_states_data.clear();
                for i in 0..self.index {
                    self.typing_states_data.push(self.typing_word[i]);
                }
                self.typing_states.push(self.typing_states_data.clone());
            }
        } else {
            // save macro words
            self.typing_states_data.clear();
            for (i, &m) in self.state.macro_data.iter().enumerate() {
                if i != 0 && i % MAX_BUFF == 0 {
                    self.typing_states.push(self.typing_states_data.clone());
                    self.typing_states_data.clear();
                }
                self.typing_states_data.push(m);
            }
            self.typing_states.push(self.typing_states_data.clone());
        }
    }

    fn save_word_space(&mut self, count: usize) {
        self.typing_states_data.clear();
        for _ in 0..count {
            self.typing_states_data.push(KEY_SPACE as u32);
        }
        self.typing_states.push(self.typing_states_data.clone());
    }

    fn save_special_char(&mut self) {
        self.typing_states_data.clear();
        for &c in &self.special_char {
            self.typing_states_data.push(c);
        }
        self.typing_states.push(self.typing_states_data.clone());
        self.special_char.clear();
    }

    fn restore_last_typing_state(&mut self) {
        if let Some(last) = self.typing_states.pop() {
            if !last.is_empty() {
                if last[0] == KEY_SPACE as u32 {
                    self.space_count = last.len();
                    self.index = 0;
                } else if CHAR_KEY_CODE.contains(&(last[0] as u16)) {
                    self.index = 0;
                    self.special_char = last;
                    self.check_spelling(false);
                } else {
                    for (i, &w) in last.iter().enumerate() {
                        if i < MAX_BUFF {
                            self.typing_word[i] = w;
                        }
                    }
                    self.index = last.len().min(MAX_BUFF);
                }
            }
        }
    }

    fn check_quick_consonant(&mut self) -> bool {
        if self.index <= 1 {
            return false;
        }
        self.l = 0;
        if self.index > 0 {
            if self.cfg.quick_start_consonant
                && let Some(qsc) = quick_start_consonant(self.chr(0))
            {
                self.state.code = HookCode::Restore;
                self.state.backspace_count = self.index as u8;
                self.state.new_char_count = (self.index + 1) as u8;
                if self.index < MAX_BUFF - 1 {
                    self.index += 1;
                }
                // right shift
                for i in (2..self.index).rev() {
                    self.typing_word[i] = self.typing_word[i - 1];
                }
                let caps2 = (self.typing_word[0] & CAPS_MASK) != 0
                    && (self.typing_word[2] & CAPS_MASK) != 0;
                self.typing_word[1] =
                    qsc[1] as u32 | if caps2 { CAPS_MASK } else { 0 };
                self.typing_word[0] = qsc[0] as u32
                    | if (self.typing_word[0] & CAPS_MASK) != 0 { CAPS_MASK } else { 0 };
                self.l = 1;
            }
            if self.cfg.quick_end_consonant
                && self.index >= 2
                && !is_consonant(self.chr(self.index - 2))
                && let Some(qec) = quick_end_consonant(self.chr(self.index - 1))
            {
                self.state.code = HookCode::Restore;
                if self.l == 1 {
                    self.state.new_char_count += 1;
                } else {
                    self.state.backspace_count = 1;
                    self.state.new_char_count = 2;
                }
                if self.index < MAX_BUFF - 1 {
                    self.index += 1;
                }
                let caps = (self.typing_word[self.index - 2] & CAPS_MASK) != 0;
                self.typing_word[self.index - 1] =
                    qec[1] as u32 | if caps { CAPS_MASK } else { 0 };
                self.typing_word[self.index - 2] =
                    qec[0] as u32 | if caps { CAPS_MASK } else { 0 };
                self.l = 1;
            }
            if self.l == 1 {
                self.has_handle_quick_consonant = true;
                for i in (0..self.index).rev() {
                    let idx = self.index - 1 - i;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.get_character_code(self.typing_word[i]);
                    }
                }
                return true;
            }
        }
        false
    }

    fn check_restore_if_wrong_spelling(&mut self) -> bool {
        for ii in 0..self.index {
            if !is_consonant(self.chr(ii))
                && ((self.typing_word[ii] & MARK_MASK) != 0
                    || (self.typing_word[ii] & TONE_MASK) != 0
                    || (self.typing_word[ii] & TONEW_MASK) != 0)
            {
                self.state.code = HookCode::RestoreAndStartNewSession;
                self.state.backspace_count = self.index as u8;
                self.state.new_char_count = self.state_index as u8;
                for i in 0..self.state_index {
                    self.typing_word[i] = self.key_states[i];
                    let idx = self.state_index - 1 - i;
                    if idx < MAX_BUFF {
                        self.state.char_data[idx] = self.typing_word[i];
                    }
                }
                self.index = self.state_index;
                return true;
            }
        }
        false
    }

    fn handle_quick_telex(&mut self, data: u16, is_caps: bool) {
        if let Some(qt) = quick_telex(data) {
            self.state.code = HookCode::WillProcess;
            self.state.backspace_count = 1;
            self.state.new_char_count = 2;
            self.state.char_data[1] = qt[0] as u32 | if is_caps { CAPS_MASK } else { 0 };
            self.state.char_data[0] = qt[1] as u32 | if is_caps { CAPS_MASK } else { 0 };
            self.insert_key(qt[1], is_caps, false);
        }
    }

    fn upper_case_first_character(&mut self) {
        if (self.typing_word[0] & CAPS_MASK) == 0 {
            self.state.code = HookCode::WillProcess;
            self.state.backspace_count = 0;
            self.state.new_char_count = 1;
            self.typing_word[0] |= CAPS_MASK;
            self.state.char_data[0] = self.get_character_code(self.typing_word[0]);
            self.upper_case_status = 0;
            if self.cfg.use_macro {
                // macro key[0] |= CAPS_MASK
            }
        }
    }
}
