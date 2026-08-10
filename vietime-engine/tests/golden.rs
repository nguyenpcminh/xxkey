//! Differential golden tests against the C++ oracle.
//!
//! Each golden vector records the screen buffer the original C++ engine
//! produces for a key sequence, under a given input type and orthography.
//! The Rust port must reproduce it exactly.
//!
//! Golden file: `data/golden.txt` (generated from `/tmp/xxkey-oracle`).
//!
//! To regenerate:
//!   cd /tmp/xxkey-oracle && ./gen_golden.sh golden.txt
//!   cp golden.txt <this dir>/data/golden.txt

use vietime_engine::datatype::*;
use vietime_engine::engine::Engine;
use vietime_engine::keycode::*;
use vietime_engine::vietnamese::key_code_to_character;

/// Map an ASCII char to a macOS key code (mirrors the C++ test harness).
fn char_to_key(ch: char) -> u16 {
    match ch {
        'a' => KEY_A, 'b' => KEY_B, 'c' => KEY_C, 'd' => KEY_D,
        'e' => KEY_E, 'f' => KEY_F, 'g' => KEY_G, 'h' => KEY_H,
        'i' => KEY_I, 'j' => KEY_J, 'k' => KEY_K, 'l' => KEY_L,
        'm' => KEY_M, 'n' => KEY_N, 'o' => KEY_O, 'p' => KEY_P,
        'q' => KEY_Q, 'r' => KEY_R, 's' => KEY_S, 't' => KEY_T,
        'u' => KEY_U, 'v' => KEY_V, 'w' => KEY_W, 'x' => KEY_X,
        'y' => KEY_Y, 'z' => KEY_Z,
        '1' => KEY_1, '2' => KEY_2, '3' => KEY_3, '4' => KEY_4,
        '5' => KEY_5, '6' => KEY_6, '7' => KEY_7, '8' => KEY_8,
        '9' => KEY_9, '0' => KEY_0,
        ' ' => KEY_SPACE,
        ',' => KEY_COMMA, '.' => KEY_DOT, ';' => KEY_SEMICOLON,
        ':' => KEY_SEMICOLON, '/' => KEY_SLASH, '?' => KEY_SLASH,
        '-' => KEY_MINUS, '=' => KEY_EQUALS, '`' => KEY_BACKQUOTE,
        '~' => KEY_BACKQUOTE, '[' => KEY_LEFT_BRACKET,
        '{' => KEY_LEFT_BRACKET, ']' => KEY_RIGHT_BRACKET,
        '}' => KEY_RIGHT_BRACKET, '\'' => KEY_QUOTE, '"' => KEY_QUOTE,
        '\\' => KEY_BACK_SLASH, '|' => KEY_BACK_SLASH,
        '_' => KEY_DELETE, '\t' => KEY_TAB,
        _ => KEY_EMPTY,
    }
}

/// Simulated screen buffer in typing order (mirrors test_engine.cpp).
fn type_text(eng: &mut Engine, text: &str) -> Vec<u16> {
    let mut screen: Vec<u16> = Vec::new();
    for c in text.chars() {
        let key = char_to_key(c);
        if key == KEY_EMPTY {
            continue;
        }
        let state = eng
            .handle_key(KeyEvent::Keyboard, KeyEventState::KeyDown, key, 0, false)
            .clone();
        match state.code {
            HookCode::DoNothing => {
                if state.ext_code == ExtCode::Delete {
                    screen.pop();
                    continue;
                }
                let ch = key_code_to_character(key as u32);
                if ch != 0 {
                    screen.push(ch);
                }
                if state.ext_code == ExtCode::WordBreak {
                    eng.start_new_session();
                }
                continue;
            }
            _ => {}
        }
        let n = state.backspace_count as usize;
        let n = n.min(screen.len());
        screen.truncate(screen.len() - n);
        for i in (0..state.new_char_count as usize).rev() {
            let c = state.char_data[i];
            let ch = if c & CHAR_CODE_MASK != 0 {
                (c & 0xFFFF) as u16
            } else {
                let k = key_code_to_character(c);
                if k == 0 {
                    c as u16
                } else {
                    k
                }
            };
            if ch != 0 {
                screen.push(ch);
            }
        }
        if state.code == HookCode::Restore {
            if is_mark_key(eng.cfg.input_type, key) {
                let ch = key_code_to_character(key as u32);
                if ch != 0 {
                    screen.push(ch);
                }
            }
        }
        if state.code == HookCode::RestoreAndStartNewSession {
            eng.start_new_session();
        }
    }
    screen
}

fn format_codes(screen: &[u16]) -> String {
    screen
        .iter()
        .map(|&c| format!("{:04X}", c))
        .collect::<Vec<_>>()
        .join(" ")
}

fn input_type_from_index(i: u32) -> InputType {
    match i {
        0 => InputType::Telex,
        1 => InputType::Vni,
        2 => InputType::SimpleTelex1,
        3 => InputType::SimpleTelex2,
        _ => InputType::Telex,
    }
}

fn make_engine(input_type: InputType, modern: bool) -> Engine {
    let mut eng = Engine::default();
    eng.cfg.input_type = input_type;
    eng.cfg.use_modern_orthography = modern;
    eng
}

fn golden_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/golden.txt")
}

fn run_all(only_telex: bool) -> (usize, Vec<(u32, bool, String, String, String)>) {
    let raw = std::fs::read_to_string(golden_path())
        .expect("golden.txt missing — run /tmp/xxkey-oracle/gen_golden.sh");
    let mut passed = 0usize;
    let mut failures = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains('|') {
            continue;
        }
        let (meta, rest) = line.split_once('|').unwrap();
        let parts: Vec<&str> = meta.splitn(3, ' ').collect();
        if parts.len() < 3 {
            continue;
        }
        let it_idx: u32 = parts[0].parse().unwrap();
        let modern: u32 = parts[1].parse().unwrap();
        let seq = parts[2];
        if only_telex && it_idx != 0 {
            continue;
        }
        let expected = rest
            .trim()
            .split_whitespace()
            .map(|s| u16::from_str_radix(s, 16).unwrap())
            .collect::<Vec<_>>();
        let mut eng = make_engine(input_type_from_index(it_idx), modern != 0);
        let screen = type_text(&mut eng, &seq);
        let got = format_codes(&screen);
        let expected_s = format_codes(&expected);
        if got == expected_s {
            passed += 1;
        } else {
            failures.push((it_idx, modern != 0, seq.to_string(), expected_s, got));
        }
    }
    (passed, failures)
}

#[test]
fn golden_telex_matches_oracle() {
    let (passed, failures) = run_all(true);
    if !failures.is_empty() {
        eprintln!(
            "GOLDEN MISMATCHES ({} of {} telex vectors):",
            failures.len(),
            passed + failures.len()
        );
        for (it, mo, seq, exp, got) in failures.iter().take(40) {
            eprintln!("  it={it} modern={mo} seq=[{seq}] expected=[{exp}] got=[{got}]");
        }
        panic!("{} telex golden vectors mismatched", failures.len());
    }
    eprintln!("golden telex: {passed} vectors OK");
}

#[test]
fn golden_all_input_types_match_oracle() {
    let (passed, failures) = run_all(false);
    if !failures.is_empty() {
        eprintln!(
            "GOLDEN MISMATCHES ({} of {} vectors):",
            failures.len(),
            passed + failures.len()
        );
        for (it, mo, seq, exp, got) in failures.iter().take(60) {
            eprintln!("  it={it} modern={mo} seq=[{seq}] expected=[{exp}] got=[{got}]");
        }
        panic!("{} golden vectors mismatched", failures.len());
    }
    eprintln!("golden all types: {passed} vectors OK");
}

#[test]
fn test_texx() {
    let mut eng = make_engine(InputType::Telex, true);
    let screen = type_text(&mut eng, "texx");
    let got = screen.iter().map(|&c| char::from_u32(c as u32).unwrap()).collect::<String>();
    assert_eq!(got, "tex");
}

#[test]
fn test_trace_words() {
    let words: &[(&str, &[char])] = &[
        ("saf",       &['s','a','f']),
        ("hoc",       &['h','o','c']),
        ("ddieesm",   &['d','d','i','e','e','s','m']),  // full điểm with dd
        ("chuaas",    &['c','h','u','a','a','s']),
        ("hoas",      &['h','o','a','s']),
        ("tuwongf",   &['t','u','w','o','n','g','f']),  // tướng
        ("dieemj",    &['d','i','e','e','m','j']),      // điệm
        ("hocj",      &['h','o','c','j']),              // học
    ];

    let mut eng = make_engine(InputType::Telex, true);
    for (label, chars) in words {
        eng.reset();
        println!("--- {} ---", label);
        for &c in *chars {
            let key = char_to_key(c);
            let state = eng.handle_key(KeyEvent::Keyboard, KeyEventState::KeyDown, key, 0, false);
            let mut char_strs = Vec::new();
            for i in (0..state.new_char_count as usize).rev() {
                let val = state.char_data[i];
                let ch = if val & CHAR_CODE_MASK != 0 {
                    (val & 0xFFFF) as u16
                } else {
                    key_code_to_character(val)
                };
                char_strs.push(char::from_u32(ch as u32).unwrap_or('?'));
            }
            let chars_out: String = char_strs.iter().collect();
            println!("  '{}' -> code={:?} bpc={} chars=\"{}\" ext={:?}",
                     c, state.code, state.backspace_count, chars_out, state.ext_code);
        }
    }
    panic!("Forcing output — check stdout above");
}

