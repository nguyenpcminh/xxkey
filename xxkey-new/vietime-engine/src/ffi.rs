//! C-compatible FFI bindings for Swift/macOS integration.

extern crate alloc;

use alloc::boxed::Box;
use crate::datatype::*;
use crate::engine::Engine;
use crate::vietnamese::key_code_to_character;

/// Create a new instance of the Vietnamese input engine.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_new_engine() -> *mut Engine {
    let engine = Box::new(Engine::default());
    Box::into_raw(engine)
}

/// Free an engine instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_free_engine(engine: *mut Engine) {
    if !engine.is_null() {
        let _ = unsafe { Box::from_raw(engine) };
    }
}

/// Reset the engine state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_reset_engine(engine: *mut Engine) {
    if !engine.is_null() {
        if let Some(eng) = unsafe { engine.as_mut() } {
            eng.reset();
        }
    }
}

/// Set the input type (0 = Telex, 1 = Vni, 2 = SimpleTelex1, 3 = SimpleTelex2).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_set_input_type(engine: *mut Engine, input_type: u8) {
    if !engine.is_null() {
        if let Some(eng) = unsafe { engine.as_mut() } {
            let it = match input_type {
                1 => InputType::Vni,
                2 => InputType::SimpleTelex1,
                3 => InputType::SimpleTelex2,
                _ => InputType::Telex,
            };
            eng.cfg.input_type = it;
            eng.reset();
        }
    }
}

/// Set modern orthography flag (0 = old/classic, 1 = modern).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_set_modern_orthography(engine: *mut Engine, modern: u8) {
    if !engine.is_null() {
        if let Some(eng) = unsafe { engine.as_mut() } {
            eng.cfg.use_modern_orthography = modern != 0;
            eng.reset();
        }
    }
}

/// Process a key event and return a pointer to the resulting HookState.
/// The returned HookState pointer is owned by the Engine and is valid until the next event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_handle_key(
    engine: *mut Engine,
    event: u8,
    state: u8,
    data: u16,
    caps_status: u8,
    other_control_key: bool,
) -> *const HookState {
    if !engine.is_null() {
        if let Some(eng) = unsafe { engine.as_mut() } {
            let ev = match event {
                1 => KeyEvent::Mouse,
                _ => KeyEvent::Keyboard,
            };
            let st = match state {
                1 => KeyEventState::KeyUp,
                2 => KeyEventState::MouseDown,
                3 => KeyEventState::MouseUp,
                _ => KeyEventState::KeyDown,
            };
            eng.handle_key(ev, st, data, caps_status, other_control_key) as *const HookState
        } else {
            core::ptr::null()
        }
    } else {
        core::ptr::null()
    }
}

/// Get the character from macOS logical keycode.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_key_code_to_char(key_code: u32) -> u16 {
    key_code_to_character(key_code)
}

/// Get code from HookState.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_get_hook_state_code(state: *const HookState) -> u8 {
    if !state.is_null() {
        if let Some(st) = unsafe { state.as_ref() } {
            st.code as u8
        } else {
            0
        }
    } else {
        0
    }
}

/// Get backspace count from HookState.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_get_hook_state_backspace_count(state: *const HookState) -> u8 {
    if !state.is_null() {
        if let Some(st) = unsafe { state.as_ref() } {
            st.backspace_count
        } else {
            0
        }
    } else {
        0
    }
}

/// Get new character count from HookState.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_get_hook_state_new_char_count(state: *const HookState) -> u8 {
    if !state.is_null() {
        if let Some(st) = unsafe { state.as_ref() } {
            st.new_char_count
        } else {
            0
        }
    } else {
        0
    }
}

/// Get ext code from HookState.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_get_hook_state_ext_code(state: *const HookState) -> u8 {
    if !state.is_null() {
        if let Some(st) = unsafe { state.as_ref() } {
            st.ext_code as u8
        } else {
            0
        }
    } else {
        0
    }
}

/// Get character data at specific index from HookState.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vietime_get_hook_state_char_at(state: *const HookState, index: u32) -> u32 {
    if !state.is_null() {
        if let Some(st) = unsafe { state.as_ref() } {
            if (index as usize) < st.char_data.len() {
                st.char_data[index as usize]
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    }
}
