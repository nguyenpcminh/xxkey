//! Code table conversion and text transformation tool module.
//! Ported from `ConvertTool.cpp`.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::datatype::*;
use crate::keycode::*;
use crate::vietnamese::*;

/// Options controlling conversion behavior.
#[derive(Clone, Debug, Default)]
pub struct ConvertOptions {
    /// Source code table (0..4).
    pub from_code: u8,
    /// Target code table (0..4).
    pub to_code: u8,
    /// Convert output to ALL CAPS.
    pub to_all_caps: bool,
    /// Convert output to all lowercase.
    pub to_all_non_caps: bool,
    /// Capitalize first letter of sentence.
    pub to_caps_first_letter: bool,
    /// Capitalize each word.
    pub to_caps_each_word: bool,
    /// Strip tone and circumflex/breve marks.
    pub remove_mark: bool,
}

const BREAK_CODE: &[u8] = &[b'.', b'?', b'!'];

fn find_key_code(char_code: u32, code_table_idx: usize) -> Option<(u32, usize)> {
    let keys = [
        KEY_A as u32, KEY_O as u32, KEY_U as u32, KEY_E as u32, KEY_D as u32,
        KEY_A as u32 | TONE_MASK, KEY_A as u32 | TONEW_MASK,
        KEY_O as u32 | TONE_MASK, KEY_O as u32 | TONEW_MASK,
        KEY_U as u32 | TONEW_MASK, KEY_E as u32 | TONE_MASK,
        KEY_I as u32, KEY_Y as u32,
    ];

    for &k_entry in &keys {
        if let Some(entries) = code_table(code_table_idx, k_entry) {
            for (z, &val) in entries.iter().enumerate() {
                if (char_code as u16) == val {
                    return Some((k_entry, z));
                }
            }
        }
    }
    None
}

fn get_unicode_compound_mark_index(mark: u16) -> u16 {
    for (i, &m) in UNICODE_COMPOUND_MARK.iter().enumerate() {
        if mark == m {
            return (i as u16 + 1) << 13;
        }
    }
    0
}

/// Transform text between Vietnamese code tables or change casing/marks.
pub fn convert_util(source_string: &str, opts: &ConvertOptions) -> String {
    let data: Vec<u16> = source_string.chars().map(|c| c as u16).collect();
    let mut temp: Vec<u16> = Vec::new();
    let mut has_break = false;
    let mut should_upper_case = opts.to_caps_first_letter || opts.to_caps_each_word;

    if opts.to_all_non_caps {
        should_upper_case = false;
    }

    let mut i = 0;
    while i < data.len() {
        let mut p = 0;
        let mut t: u16;

        // 1. Try 2-char compound lookups for VNI (2), 1258 (4), or Unicode Compound (3)
        if i < data.len().saturating_sub(1) {
            match opts.from_code {
                2 | 4 => {
                    t = data[i] | (data[i + 1] << 8);
                    p = 1;
                }
                3 => {
                    let target_mark = get_unicode_compound_mark_index(data[i + 1]);
                    if target_mark > 0 {
                        t = data[i] | target_mark;
                        p = 1;
                    } else {
                        t = data[i];
                    }
                }
                _ => {
                    t = data[i];
                }
            }

            if let Some((j, k)) = find_key_code(t as u32, opts.from_code as usize) {
                i += p;
                let mut target = if let Some(entries) = code_table(opts.to_code as usize, j) {
                    entries.get(k).copied().unwrap_or(data[i])
                } else {
                    data[i]
                };

                if (opts.to_all_caps || should_upper_case) && k % 2 != 0 {
                    if let Some(entries) = code_table(opts.to_code as usize, j) {
                        target = entries.get(k - 1).copied().unwrap_or(target);
                    }
                } else if (opts.to_all_non_caps || !should_upper_case) && k % 2 == 0 {
                    if let Some(entries) = code_table(opts.to_code as usize, j) {
                        target = entries.get(k + 1).copied().unwrap_or(target);
                    }
                }

                if opts.remove_mark {
                    let base_char = key_code_to_character(j & 0xFF);
                    let is_upper = k % 2 == 0;
                    target = if opts.to_all_caps || (is_upper && !opts.to_all_non_caps) {
                        (base_char as u8 as char).to_ascii_uppercase() as u16
                    } else {
                        (base_char as u8 as char).to_ascii_lowercase() as u16
                    };
                }

                match opts.to_code {
                    0 | 1 => {
                        temp.push(target);
                    }
                    2 | 4 => {
                        let hi = (target >> 8) as u8;
                        if hi > 32 {
                            temp.push((target & 0xFF) as u16);
                            temp.push(hi as u16);
                        } else {
                            temp.push((target & 0xFF) as u16);
                        }
                    }
                    3 => {
                        let mark_idx = target >> 13;
                        if mark_idx > 0 {
                            temp.push(target & 0x1FFF);
                            if (mark_idx as usize) <= UNICODE_COMPOUND_MARK.len() {
                                temp.push(UNICODE_COMPOUND_MARK[mark_idx as usize - 1]);
                            }
                        } else {
                            temp.push(target);
                        }
                    }
                    _ => {
                        temp.push(target);
                    }
                }

                should_upper_case = false;
                has_break = false;
                i += 1;
                continue;
            }
        }

        // 2. Single char lookup
        t = data[i];
        if let Some((j, k)) = find_key_code(t as u32, opts.from_code as usize) {
            let mut target = if let Some(entries) = code_table(opts.to_code as usize, j) {
                entries.get(k).copied().unwrap_or(data[i])
            } else {
                data[i]
            };

            if (opts.to_all_caps || should_upper_case) && k % 2 != 0 {
                if let Some(entries) = code_table(opts.to_code as usize, j) {
                    target = entries.get(k - 1).copied().unwrap_or(target);
                }
            } else if (opts.to_all_non_caps || !should_upper_case) && k % 2 == 0 {
                if let Some(entries) = code_table(opts.to_code as usize, j) {
                    target = entries.get(k + 1).copied().unwrap_or(target);
                }
            }

            if opts.remove_mark {
                let base_char = key_code_to_character(j & 0xFF);
                let is_upper = k % 2 == 0;
                target = if opts.to_all_caps || (is_upper && !opts.to_all_non_caps) {
                    (base_char as u8 as char).to_ascii_uppercase() as u16
                } else {
                    (base_char as u8 as char).to_ascii_lowercase() as u16
                };
            }

            temp.push(target);
            should_upper_case = false;
            has_break = false;
            i += 1;
            continue;
        }

        // 3. Plain normal character
        let ch = data[i] as u8 as char;
        let final_ch = if opts.to_all_caps || should_upper_case {
            ch.to_ascii_uppercase()
        } else if opts.to_all_non_caps {
            ch.to_ascii_lowercase()
        } else {
            ch
        };
        temp.push(final_ch as u16);

        if t == b'\n' as u16 || (has_break && t == b' ' as u16) {
            if opts.to_caps_first_letter || opts.to_caps_each_word {
                should_upper_case = true;
            }
        } else if t == b' ' as u16 && opts.to_caps_each_word {
            should_upper_case = true;
        } else if BREAK_CODE.contains(&(t as u8)) {
            has_break = true;
        } else {
            should_upper_case = false;
            has_break = false;
        }

        i += 1;
    }

    temp.into_iter().map(|c| char::from_u32(c as u32).unwrap_or('?')).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_mark() {
        let opts = ConvertOptions {
            from_code: 0,
            to_code: 0,
            remove_mark: true,
            ..Default::default()
        };
        let out = convert_util("Việt Nam", &opts);
        assert_eq!(out, "Viet Nam");
    }

    #[test]
    fn test_casing_options() {
        let opts = ConvertOptions {
            from_code: 0,
            to_code: 0,
            to_all_caps: true,
            ..Default::default()
        };
        let out = convert_util("xin chào", &opts);
        assert_eq!(out, "XIN CHÀO");

        let opts_lower = ConvertOptions {
            from_code: 0,
            to_code: 0,
            to_all_non_caps: true,
            ..Default::default()
        };
        let out_lower = convert_util("VIỆT NAM", &opts_lower);
        assert_eq!(out_lower, "việt nam");
    }
}
