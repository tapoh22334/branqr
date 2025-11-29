use std::ptr::null_mut;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "Blanqr";

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn open_run_key(access: u32) -> Option<HKEY> {
    unsafe {
        let mut hkey: HKEY = null_mut();
        let key_path = wide_string(RUN_KEY);
        let result = RegOpenKeyExW(HKEY_CURRENT_USER, key_path.as_ptr(), 0, access, &mut hkey);
        if result == 0 {
            Some(hkey)
        } else {
            None
        }
    }
}

pub fn is_startup_enabled() -> bool {
    unsafe {
        if let Some(hkey) = open_run_key(KEY_READ) {
            let name = wide_string(APP_NAME);
            let mut data_type: u32 = 0;
            let mut data_size: u32 = 0;
            let result = RegQueryValueExW(
                hkey,
                name.as_ptr(),
                null_mut(),
                &mut data_type,
                null_mut(),
                &mut data_size,
            );
            RegCloseKey(hkey);
            result == 0
        } else {
            false
        }
    }
}

pub fn set_startup_enabled(enabled: bool) -> bool {
    unsafe {
        if let Some(hkey) = open_run_key(KEY_WRITE) {
            let name = wide_string(APP_NAME);
            let result = if enabled {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = exe_path.to_string_lossy();
                    let path_wide = wide_string(&path_str);
                    let data_len = (path_wide.len() * 2) as u32;
                    RegSetValueExW(
                        hkey,
                        name.as_ptr(),
                        0,
                        REG_SZ,
                        path_wide.as_ptr() as *const u8,
                        data_len,
                    )
                } else {
                    1 // Error
                }
            } else {
                RegDeleteValueW(hkey, name.as_ptr())
            };
            RegCloseKey(hkey);
            result == 0
        } else {
            false
        }
    }
}

pub fn ensure_startup_enabled() {
    if !is_startup_enabled() {
        set_startup_enabled(true);
    }
}
