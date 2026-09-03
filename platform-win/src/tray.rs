use crate::config::ConfigManager;
use vietime_engine::datatype::InputType;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub const WM_TRAYICON: u32 = WM_USER + 100;

pub const ID_TRAY_TOGGLE: u32 = 2001;
pub const ID_TRAY_TELEX: u32 = 2002;
pub const ID_TRAY_VNI: u32 = 2003;
pub const ID_TRAY_SIMPLE1: u32 = 2004;
pub const ID_TRAY_SIMPLE2: u32 = 2005;
pub const ID_TRAY_SETTINGS: u32 = 2006;
pub const ID_TRAY_EXIT: u32 = 2007;

pub unsafe fn create_system_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = unsafe { LoadIconW(0, IDI_APPLICATION as *const u16) };

    let tip = "XXKey - Vietnamese Input Method";
    let tip_units: Vec<u16> = tip.encode_utf16().collect();
    for (i, &u) in tip_units.iter().enumerate() {
        if i < nid.szTip.len() - 1 {
            nid.szTip[i] = u;
        }
    }

    unsafe {
        Shell_NotifyIconW(NIM_ADD, &nid);
    }
}

pub unsafe fn remove_system_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;

    unsafe {
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}

pub unsafe fn show_tray_popup_menu(hwnd: HWND, enabled: bool, current_input: InputType) {
    let mut cursor_pos = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut cursor_pos);
    }

    let menu = unsafe { CreatePopupMenu() };
    if menu == 0 {
        return;
    }

    let toggle_str = if enabled {
        "Status: ON (Click to Disable)\0"
    } else {
        "Status: OFF (Click to Enable)\0"
    };
    let toggle_u16: Vec<u16> = toggle_str.encode_utf16().collect();

    let telex_str = "  Telex\0";
    let vni_str = "  VNI\0";
    let simple1_str = "  Simple Telex 1\0";
    let simple2_str = "  Simple Telex 2\0";
    let settings_str = "Settings...\0";
    let exit_str = "Exit XXKey\0";

    unsafe {
        AppendMenuW(menu, MF_STRING, ID_TRAY_TOGGLE as usize, toggle_u16.as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());

        let telex_flags = MF_STRING | if current_input == InputType::Telex { MF_CHECKED } else { 0 };
        let vni_flags = MF_STRING | if current_input == InputType::Vni { MF_CHECKED } else { 0 };
        let s1_flags = MF_STRING | if current_input == InputType::SimpleTelex1 { MF_CHECKED } else { 0 };
        let s2_flags = MF_STRING | if current_input == InputType::SimpleTelex2 { MF_CHECKED } else { 0 };

        AppendMenuW(menu, telex_flags, ID_TRAY_TELEX as usize, telex_str.encode_utf16().collect::<Vec<u16>>().as_ptr());
        AppendMenuW(menu, vni_flags, ID_TRAY_VNI as usize, vni_str.encode_utf16().collect::<Vec<u16>>().as_ptr());
        AppendMenuW(menu, s1_flags, ID_TRAY_SIMPLE1 as usize, simple1_str.encode_utf16().collect::<Vec<u16>>().as_ptr());
        AppendMenuW(menu, s2_flags, ID_TRAY_SIMPLE2 as usize, simple2_str.encode_utf16().collect::<Vec<u16>>().as_ptr());

        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(menu, MF_STRING, ID_TRAY_SETTINGS as usize, settings_str.encode_utf16().collect::<Vec<u16>>().as_ptr());
        AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT as usize, exit_str.encode_utf16().collect::<Vec<u16>>().as_ptr());

        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            menu,
            TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            cursor_pos.x,
            cursor_pos.y,
            0,
            hwnd,
            std::ptr::null(),
        );
        DestroyMenu(menu);
    }
}

pub fn handle_tray_command(cmd_id: u32, config_mgr: &mut ConfigManager) -> bool {
    match cmd_id {
        ID_TRAY_TOGGLE => {
            config_mgr.current.enabled = !config_mgr.current.enabled;
            config_mgr.save();
        }
        ID_TRAY_TELEX => {
            config_mgr.current.input_type = InputType::Telex;
            config_mgr.save();
        }
        ID_TRAY_VNI => {
            config_mgr.current.input_type = InputType::Vni;
            config_mgr.save();
        }
        ID_TRAY_SIMPLE1 => {
            config_mgr.current.input_type = InputType::SimpleTelex1;
            config_mgr.save();
        }
        ID_TRAY_SIMPLE2 => {
            config_mgr.current.input_type = InputType::SimpleTelex2;
            config_mgr.save();
        }
        ID_TRAY_SETTINGS => {
            if let Ok(exe_path) = std::env::current_exe() {
                let settings_exe = exe_path.with_file_name("xxkey-settings.exe");
                let _ = std::process::Command::new(settings_exe).spawn();
            }
        }
        ID_TRAY_EXIT => {
            return true;
        }
        _ => {}
    }
    false
}
