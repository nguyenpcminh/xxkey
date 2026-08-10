//! Macro management module for Vietnamese input engine.
//! Ported from `Macro.cpp`.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::datatype::*;
use crate::engine::Engine;
use crate::keycode::*;
use crate::vietnamese::*;

/// Data associated with a macro definition.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MacroData {
    /// Macro trigger text (key).
    pub macro_text: String,
    /// Expanded replacement text.
    pub macro_content: String,
    /// Codepoint/keycode sequence for output replacement.
    pub macro_content_code: Vec<u32>,
}

/// Convert a string into engine key/character code sequence.
pub fn convert_str_to_key_codes(s: &str, code_table_idx: usize) -> Vec<u32> {
    let mut out = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i] as u32;

        // 1. Check direct character map
        if let Some(code) = character_to_key_code(ch) {
            out.push(code);
            i += 1;
            continue;
        }

        // 2. Check tone/mark character in code_table 0
        let mut found = false;
        if let Some(table0) = code_table(0, KEY_A as u32) {
            // iterate code table 0 entries
            let keys_to_check = [
                KEY_A as u32, KEY_O as u32, KEY_U as u32, KEY_E as u32, KEY_D as u32,
                KEY_A as u32 | TONE_MASK, KEY_A as u32 | TONEW_MASK,
                KEY_O as u32 | TONE_MASK, KEY_O as u32 | TONEW_MASK,
                KEY_U as u32 | TONEW_MASK, KEY_E as u32 | TONE_MASK,
                KEY_I as u32, KEY_Y as u32,
            ];

            for &key_entry in &keys_to_check {
                if let Some(entries) = code_table(0, key_entry) {
                    for (k_idx, &c_code) in entries.iter().enumerate() {
                        if (ch as u16) == c_code {
                            if let Some(target_entries) = code_table(code_table_idx, key_entry) {
                                if k_idx < target_entries.len() {
                                    out.push(target_entries[k_idx] as u32 | CHAR_CODE_MASK);
                                    found = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if found {
                    break;
                }
            }
        }

        if found {
            i += 1;
            continue;
        }

        // 3. Mark as pure character
        out.push(ch | PURE_CHARACTER_MASK);
        i += 1;
    }

    out
}

/// Macro table manager.
#[derive(Clone, Debug, Default)]
pub struct MacroManager {
    map: BTreeMap<Vec<u32>, MacroData>,
}

impl MacroManager {
    /// Create new empty macro manager.
    pub fn new() -> Self {
        MacroManager {
            map: BTreeMap::new(),
        }
    }

    /// Number of stored macros.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether macro manager is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all stored macros.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Add or update a macro definition.
    pub fn add_macro(&mut self, text: &str, content: &str, code_table_idx: usize) {
        let key = convert_str_to_key_codes(text, code_table_idx);
        let content_code = convert_str_to_key_codes(content, code_table_idx);

        let data = MacroData {
            macro_text: String::from(text),
            macro_content: String::from(content),
            macro_content_code: content_code,
        };

        self.map.insert(key, data);
    }

    /// Delete a macro by name.
    pub fn delete_macro(&mut self, text: &str, code_table_idx: usize) -> bool {
        let key = convert_str_to_key_codes(text, code_table_idx);
        self.map.remove(&key).is_some()
    }

    /// Check if a macro name exists.
    pub fn has_macro(&self, text: &str, code_table_idx: usize) -> bool {
        let key = convert_str_to_key_codes(text, code_table_idx);
        self.map.contains_key(&key)
    }

    /// Find a macro match, with optional auto-caps support.
    pub fn find_macro(
        &self,
        engine: &mut Engine,
        key: &mut [u32],
        macro_content_code: &mut Vec<u32>,
        auto_caps: bool,
        code_table_idx: usize,
    ) -> bool {
        for code in key.iter_mut() {
            *code = engine.get_character_code(*code);
        }

        if let Some(data) = self.map.get(key) {
            macro_content_code.clear();
            macro_content_code.extend_from_slice(&data.macro_content_code);
            return true;
        }

        if auto_caps {
            let mut key_vec = key.to_vec();
            let mut macro_flag = false;

            if key_vec.len() > 1 && modify_case_unicode(&mut key_vec[1], false, code_table_idx) {
                macro_flag = true;
                for c in 2..key_vec.len() {
                    modify_case_unicode(&mut key_vec[c], false, code_table_idx);
                }
            }

            if !key_vec.is_empty() && modify_case_unicode(&mut key_vec[0], false, code_table_idx) {
                if let Some(data) = self.map.get(&key_vec) {
                    macro_content_code.clear();
                    macro_content_code.extend_from_slice(&data.macro_content_code);

                    for (c, item) in macro_content_code.iter_mut().enumerate() {
                        if c == 0 || macro_flag {
                            let k_char = key_code_to_character(*item);
                            if k_char != 0 {
                                let upper = (k_char as u8 as char).to_ascii_uppercase() as u32;
                                if let Some(code) = character_to_key_code(upper) {
                                    *item = code;
                                    continue;
                                }
                            }
                            if *item & CHAR_CODE_MASK != 0 {
                                modify_case_unicode(item, true, code_table_idx);
                            }
                        }
                    }
                    return true;
                }
            }
        }

        false
    }

    /// Get all macros as a list of tuples containing (key, macro_text, macro_content).
    pub fn get_all_macros(&self) -> Vec<(Vec<u32>, String, String)> {
        self.map
            .iter()
            .map(|(k, v)| (k.clone(), v.macro_text.clone(), v.macro_content.clone()))
            .collect()
    }

    /// Update macro content codes when the active code table changes.
    pub fn on_table_code_change(&mut self, code_table_idx: usize) {
        for data in self.map.values_mut() {
            data.macro_content_code = convert_str_to_key_codes(&data.macro_content, code_table_idx);
        }
    }

    /// Import macros from UniKey text format.
    pub fn read_from_unikey_text(&mut self, text: &str, append: bool, code_table_idx: usize) {
        if !append {
            self.clear();
        }

        for (line_idx, line) in text.lines().enumerate() {
            if line_idx == 0 && line.starts_with(';') {
                continue;
            }
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            if let Some((name, content)) = line.split_once(':') {
                let name = name.trim();
                let content = content.trim();
                if !name.is_empty() {
                    self.add_macro(name, content, code_table_idx);
                }
            }
        }
    }

    /// Export macros to UniKey text format.
    pub fn save_to_unikey_text(&self) -> String {
        let mut out = String::from(";Compatible OpenKey Macro Data file for UniKey*** version=1 ***\n");
        for data in self.map.values() {
            out.push_str(&data.macro_text);
            out.push(':');
            out.push_str(&data.macro_content);
            out.push('\n');
        }
        out
    }

    /// Load binary macro data.
    pub fn init_from_binary(&mut self, p_data: &[u8], code_table_idx: usize) {
        self.clear();
        if p_data.len() < 2 {
            return;
        }
        let macro_count = u16::from_le_bytes([p_data[0], p_data[1]]) as usize;
        let mut cursor = 2;

        for _ in 0..macro_count {
            if cursor >= p_data.len() {
                break;
            }
            let text_len = p_data[cursor] as usize;
            cursor += 1;
            if cursor + text_len > p_data.len() {
                break;
            }
            let text = match core::str::from_utf8(&p_data[cursor..cursor + text_len]) {
                Ok(s) => s,
                Err(_) => break,
            };
            cursor += text_len;

            if cursor + 2 > p_data.len() {
                break;
            }
            let content_len = u16::from_le_bytes([p_data[cursor], p_data[cursor + 1]]) as usize;
            cursor += 2;
            if cursor + content_len > p_data.len() {
                break;
            }
            let content = match core::str::from_utf8(&p_data[cursor..cursor + content_len]) {
                Ok(s) => s,
                Err(_) => break,
            };
            cursor += content_len;

            self.add_macro(text, content, code_table_idx);
        }
    }

    /// Export binary macro data.
    pub fn get_binary_save_data(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let total = self.map.len() as u16;
        out.extend_from_slice(&total.to_le_bytes());

        for data in self.map.values() {
            let text_bytes = data.macro_text.as_bytes();
            out.push(text_bytes.len() as u8);
            out.extend_from_slice(text_bytes);

            let content_bytes = data.macro_content.as_bytes();
            let content_len = content_bytes.len() as u16;
            out.extend_from_slice(&content_len.to_le_bytes());
            out.extend_from_slice(content_bytes);
        }

        out
    }
}

fn modify_case_unicode(code: &mut u32, is_upper: bool, code_table_idx: usize) -> bool {
    let char_buff = *code;
    if *code & CHAR_CODE_MASK == 0 {
        if is_upper {
            *code |= CAPS_MASK;
        } else {
            *code &= !CAPS_MASK;
        }
        return *code != char_buff;
    }

    let keys = [
        KEY_A as u32, KEY_O as u32, KEY_U as u32, KEY_E as u32, KEY_D as u32,
        KEY_A as u32 | TONE_MASK, KEY_A as u32 | TONEW_MASK,
        KEY_O as u32 | TONE_MASK, KEY_O as u32 | TONEW_MASK,
        KEY_U as u32 | TONEW_MASK, KEY_E as u32 | TONE_MASK,
        KEY_I as u32, KEY_Y as u32,
    ];

    for &k_entry in &keys {
        if let Some(entries) = code_table(code_table_idx, k_entry) {
            for (idx, &c_val) in entries.iter().enumerate() {
                if (*code as u16) == c_val {
                    let mut new_idx = idx;
                    if idx % 2 == 0 && !is_upper {
                        new_idx = idx + 1;
                    } else if idx % 2 != 0 && is_upper {
                        new_idx = idx.saturating_sub(1);
                    }
                    if new_idx < entries.len() {
                        *code = entries[new_idx] as u32 | CHAR_CODE_MASK;
                        return *code != char_buff;
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_query_macro() {
        let mut mgr = MacroManager::new();
        mgr.add_macro("btw", "by the way", 0);
        assert_eq!(mgr.len(), 1);
        assert!(mgr.has_macro("btw", 0));
        assert!(!mgr.has_macro("omg", 0));

        assert!(mgr.delete_macro("btw", 0));
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn test_unikey_text_roundtrip() {
        let mut mgr = MacroManager::new();
        mgr.add_macro("btw", "by the way", 0);
        mgr.add_macro("omg", "oh my god", 0);

        let exported = mgr.save_to_unikey_text();
        assert!(exported.contains("btw:by the way"));
        assert!(exported.contains("omg:oh my god"));

        let mut mgr2 = MacroManager::new();
        mgr2.read_from_unikey_text(&exported, false, 0);
        assert_eq!(mgr2.len(), 2);
        assert!(mgr2.has_macro("btw", 0));
        assert!(mgr2.has_macro("omg", 0));
    }

    #[test]
    fn test_binary_roundtrip() {
        let mut mgr = MacroManager::new();
        mgr.add_macro("vn", "Việt Nam", 0);
        let bytes = mgr.get_binary_save_data();

        let mut mgr2 = MacroManager::new();
        mgr2.init_from_binary(&bytes, 0);
        assert_eq!(mgr2.len(), 1);
        assert!(mgr2.has_macro("vn", 0));
    }
}
