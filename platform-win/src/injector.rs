use std::cell::Cell;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

thread_local! {
    /// Flag to prevent recursive key processing when we synthesize input via SendInput.
    static IS_INJECTING: Cell<bool> = const { Cell::new(false) };
}

/// Returns true if the thread is currently injecting artificial keyboard events.
pub fn is_injecting() -> bool {
    IS_INJECTING.with(|cell| cell.get())
}

/// Sets the injecting flag.
pub fn set_injecting(injecting: bool) {
    IS_INJECTING.with(|cell| cell.set(injecting));
}

/// Injects backspaces and unicode characters atomically in a single SendInput call.
pub fn send_edits(backspace_count: usize, chars: &[u32]) {
    if backspace_count == 0 && chars.is_empty() {
        return;
    }

    set_injecting(true);

    let mut inputs: Vec<INPUT> = Vec::with_capacity(backspace_count * 2 + chars.len() * 4);

    for _ in 0..backspace_count {
        let mut down: INPUT = unsafe { std::mem::zeroed() };
        down.r#type = INPUT_KEYBOARD;
        down.Anonymous.ki.wVk = VK_BACK as u16;
        down.Anonymous.ki.dwFlags = 0;

        let mut up: INPUT = unsafe { std::mem::zeroed() };
        up.r#type = INPUT_KEYBOARD;
        up.Anonymous.ki.wVk = VK_BACK as u16;
        up.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        inputs.push(down);
        inputs.push(up);
    }

    for &code in chars {
        if code == 0 {
            continue;
        }

        if let Some(ch) = char::from_u32(code) {
            let mut utf16_buf = [0u16; 2];
            let encoded = ch.encode_utf16(&mut utf16_buf);

            for &u16_val in encoded.iter() {
                let mut down: INPUT = unsafe { std::mem::zeroed() };
                down.r#type = INPUT_KEYBOARD;
                down.Anonymous.ki.wScan = u16_val;
                down.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE;

                let mut up: INPUT = unsafe { std::mem::zeroed() };
                up.r#type = INPUT_KEYBOARD;
                up.Anonymous.ki.wScan = u16_val;
                up.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

                inputs.push(down);
                inputs.push(up);
            }
        }
    }

    if !inputs.is_empty() {
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }

    set_injecting(false);
}

/// Injects `count` Backspace keystrokes into the active application.
pub fn send_backspaces(count: usize) {
    send_edits(count, &[]);
}

/// Injects a slice of Unicode UTF-32 codepoints into the active application.
pub fn send_unicode_chars(chars: &[u32]) {
    send_edits(0, chars);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injecting_flag() {
        assert!(!is_injecting());
        set_injecting(true);
        assert!(is_injecting());
        set_injecting(false);
        assert!(!is_injecting());
    }
}
