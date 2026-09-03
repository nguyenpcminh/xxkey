pub mod config;
pub mod injector;
pub mod tray;
pub mod vk_map;

use config::ConfigManager;
use injector::{is_injecting, send_edits};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tray::{
    create_system_tray_icon, handle_tray_command, remove_system_tray_icon, show_tray_popup_menu,
    update_system_tray_icon, WM_TRAYICON,
};
use vietime_engine::datatype::{ExtCode, HookCode, KeyEvent, KeyEventState};
use vietime_engine::engine::Engine;
use vk_map::*;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL, VK_SHIFT};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

#[repr(C)]
struct CustomWndClassExW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: WNDPROC,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: HINSTANCE,
    h_icon: HICON,
    h_cursor: HCURSOR,
    hbr_background: isize,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: HICON,
}

unsafe extern "system" {
    fn RegisterClassExW(lpwcx: *const CustomWndClassExW) -> u16;
}

struct State {
    engine: Engine,
    config_mgr: ConfigManager,
    modifiers: ModifierState,
    last_config_check: Instant,
    hwnd: HWND,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);
static mut HOOK: HHOOK = 0;

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 || is_injecting() {
        return unsafe { CallNextHookEx(HOOK, code, wparam, lparam) };
    }

    let kbd = unsafe { *(lparam as *const KBDLLHOOKSTRUCT) };
    let vk = kbd.vkCode;
    let msg_type = wparam as u32;

    let is_key_down = msg_type == WM_KEYDOWN || msg_type == WM_SYSKEYDOWN;

    if let Ok(mut guard) = STATE.lock() {
        if let Some(ref mut st) = *guard {
            // Throttle disk I/O config checks to at most once per second to ensure hook responsiveness
            let now = Instant::now();
            if now.duration_since(st.last_config_check) >= Duration::from_secs(1) {
                if st.config_mgr.reload_if_needed() {
                    if st.hwnd != 0 {
                        unsafe {
                            update_system_tray_icon(st.hwnd, st.config_mgr.current.enabled);
                        }
                    }
                }
                st.last_config_check = now;
            }

            // Sync modifier key states
            match vk {
                vk_map::VK_SHIFT | vk_map::VK_LSHIFT | vk_map::VK_RSHIFT => {
                    st.modifiers.shift = is_key_down;
                }
                vk_map::VK_CONTROL | vk_map::VK_LCONTROL | vk_map::VK_RCONTROL => {
                    st.modifiers.ctrl = is_key_down;
                }
                vk_map::VK_MENU => {
                    st.modifiers.alt = is_key_down;
                }
                0x5B | 0x5C => {
                    // VK_LWIN / VK_RWIN
                    st.modifiers.win = is_key_down;
                }
                _ => {}
            }

            // Global shortcut check: Ctrl + Space or Alt + Z toggles ON / OFF
            if is_key_down {
                if (st.modifiers.ctrl && vk == vk_map::VK_SPACE)
                    || (st.modifiers.alt && vk == 0x5A)
                {
                    st.config_mgr.current.enabled = !st.config_mgr.current.enabled;
                    st.config_mgr.save();
                    if st.hwnd != 0 {
                        unsafe {
                            update_system_tray_icon(st.hwnd, st.config_mgr.current.enabled);
                        }
                    }
                    println!(
                        "XXKey engine status toggled: {}",
                        if st.config_mgr.current.enabled {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                    return 1; // Suppress hotkey event
                }
            }

            // If engine disabled or Ctrl/Alt/Win combo is held (except Shift/Caps), pass through
            if !st.config_mgr.current.enabled || st.modifiers.has_control_modifier() {
                if is_key_down {
                    st.engine.reset();
                }
                return unsafe { CallNextHookEx(HOOK, code, wparam, lparam) };
            }

            // Apply active config to engine
            st.engine.cfg.input_type = st.config_mgr.current.input_type;
            st.engine.cfg.use_modern_orthography = st.config_mgr.current.modern;
            st.engine.cfg.check_spelling = st.config_mgr.current.spelling;
            st.engine.cfg.use_macro = st.config_mgr.current.use_macro;

            if is_key_down {
                if let Some(logical_key) = vk_to_logical_key(vk) {
                    let ev_state = KeyEventState::KeyDown;

                    // Query OS GetKeyState for exact Caps Lock and Shift status
                    let caps_on = unsafe { (GetKeyState(VK_CAPITAL as i32) as u16 & 0x0001) != 0 };
                    let shift_on = unsafe { (GetKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0 };
                    let caps_status = if shift_on ^ caps_on { 1 } else { 0 };
                    let other_ctrl = false;

                    let hook_state = st.engine.handle_key(
                        KeyEvent::Keyboard,
                        ev_state,
                        logical_key,
                        caps_status,
                        other_ctrl,
                    );

                    match hook_state.code {
                        HookCode::WillProcess
                        | HookCode::Restore
                        | HookCode::ReplaceMacro
                        | HookCode::RestoreAndStartNewSession => {
                            let bs_count = hook_state.backspace_count as usize;
                            let new_count = hook_state.new_char_count as usize;

                            let mut chars: Vec<u32> = Vec::with_capacity(new_count);
                            for i in (0..new_count).rev() {
                                chars.push(hook_state.char_data[i]);
                            }

                            // Inject synthesized edits atomically in a single SendInput invocation
                            send_edits(bs_count, &chars);

                            return 1; // Intercept key
                        }
                        HookCode::BreakWord => {
                            if hook_state.ext_code == ExtCode::Delete {
                                // Backspace was handled by engine word state
                            }
                        }
                        HookCode::DoNothing => {}
                    }
                }
            }
        }
    }

    unsafe { CallNextHookEx(HOOK, code, wparam, lparam) }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = lparam as u32;
            if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                if let Ok(mut guard) = STATE.lock() {
                    if let Some(ref mut st) = *guard {
                        unsafe {
                            show_tray_popup_menu(
                                hwnd,
                                st.config_mgr.current.enabled,
                                st.config_mgr.current.input_type,
                            );
                        }
                    }
                }
            } else if event == WM_LBUTTONDBLCLK {
                if let Ok(exe_path) = std::env::current_exe() {
                    let settings_exe = exe_path.with_file_name("xxkey-settings.exe");
                    let _ = std::process::Command::new(settings_exe).spawn();
                }
            }
            0
        }
        WM_COMMAND => {
            let cmd_id = (wparam & 0xFFFF) as u32;
            if let Ok(mut guard) = STATE.lock() {
                if let Some(ref mut st) = *guard {
                    let prev_enabled = st.config_mgr.current.enabled;
                    let should_exit = handle_tray_command(cmd_id, &mut st.config_mgr);
                    if st.config_mgr.current.enabled != prev_enabled {
                        unsafe {
                            update_system_tray_icon(hwnd, st.config_mgr.current.enabled);
                        }
                    }
                    if should_exit {
                        unsafe {
                            PostQuitMessage(0);
                        }
                    }
                }
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                remove_system_tray_icon(hwnd);
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn main() {
    println!("XXKey background daemon starting on Windows...");

    let engine = Engine::default();
    let config_mgr = ConfigManager::new();

    {
        let mut guard = STATE.lock().unwrap();
        *guard = Some(State {
            engine,
            config_mgr,
            modifiers: ModifierState::default(),
            last_config_check: Instant::now(),
            hwnd: 0,
        });
    }

    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());

        // Register a message-only window for system tray icon handling
        let class_name = [
            'X' as u16, 'X' as u16, 'K' as u16, 'e' as u16, 'y' as u16, 'C' as u16, 'l' as u16,
            'a' as u16, 's' as u16, 's' as u16, 0,
        ];
        let wnd_class = CustomWndClassExW {
            cb_size: std::mem::size_of::<CustomWndClassExW>() as u32,
            style: 0,
            lpfn_wnd_proc: Some(window_proc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: instance,
            h_icon: 0,
            h_cursor: 0,
            hbr_background: 0,
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: 0,
        };

        RegisterClassExW(&wnd_class);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            0,
            instance,
            std::ptr::null(),
        );

        if hwnd != 0 {
            if let Ok(mut guard) = STATE.lock() {
                if let Some(ref mut st) = *guard {
                    st.hwnd = hwnd;
                    create_system_tray_icon(hwnd, st.config_mgr.current.enabled);
                }
            }
        }

        HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), instance, 0);

        if HOOK == 0 {
            eprintln!("Failed to install low-level keyboard hook.");
            return;
        }

        println!("Keyboard hook installed successfully. Running message loop...");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if hwnd != 0 {
            remove_system_tray_icon(hwnd);
        }
        UnhookWindowsHookEx(HOOK);
    }
}
