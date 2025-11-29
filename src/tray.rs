use crate::startup;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    GetCursorPos, LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow, TrackPopupMenu,
    CS_HREDRAW, CS_VREDRAW, IDI_APPLICATION, MF_CHECKED, MF_GRAYED, MF_SEPARATOR, MF_STRING,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_RBUTTONUP,
    WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

const WM_TRAYICON: u32 = WM_USER + 1;

pub const MENU_SELECT_COLOR: u16 = 101;
pub const MENU_CONFIGURE_HOTKEY: u16 = 102;
pub const MENU_STARTUP: u16 = 103;
pub const MENU_EXIT: u16 = 199;

static CLASS_NAME: &[u16] = &[
    'B' as u16, 'l' as u16, 'a' as u16, 'n' as u16, 'q' as u16, 'r' as u16, 'T' as u16, 'r' as u16,
    'a' as u16, 'y' as u16, 0,
];

static mut TRAY_CALLBACK: Option<Box<dyn Fn(TrayEvent)>> = None;
static HOTKEY_DISPLAY: Mutex<String> = Mutex::new(String::new());

pub enum TrayEvent {
    DoubleClick,
    SelectColor,
    ConfigureHotkey,
    ToggleStartup,
    Exit,
}

pub struct TrayIcon {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub fn new<F>(hotkey_display: &str, callback: F) -> Option<Self>
    where
        F: Fn(TrayEvent) + 'static,
    {
        unsafe {
            TRAY_CALLBACK = Some(Box::new(callback));
            if let Ok(mut display) = HOTKEY_DISPLAY.lock() {
                *display = hotkey_display.to_string();
            }

            let hinstance = GetModuleHandleW(null_mut());
            if hinstance.is_null() {
                return None;
            }

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(tray_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: null_mut(),
                hCursor: null_mut(),
                hbrBackground: null_mut(),
                lpszMenuName: null_mut(),
                lpszClassName: CLASS_NAME.as_ptr(),
            };

            RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                0,
                CLASS_NAME.as_ptr(),
                CLASS_NAME.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                null_mut(),
                null_mut(),
                hinstance,
                null_mut(),
            );

            if hwnd.is_null() {
                return None;
            }

            let icon = LoadIconW(null_mut(), IDI_APPLICATION);

            let mut nid: NOTIFYICONDATAW = zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = icon;

            let tip = "Blanqr";
            let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
            nid.szTip[..tip_wide.len().min(128)]
                .copy_from_slice(&tip_wide[..tip_wide.len().min(128)]);

            Shell_NotifyIconW(NIM_ADD, &nid);

            Some(TrayIcon { hwnd, nid })
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.nid);
            DestroyWindow(self.hwnd);
        }
    }
}

pub fn update_hotkey_display(display: &str) {
    if let Ok(mut hotkey) = HOTKEY_DISPLAY.lock() {
        *hotkey = display.to_string();
    }
}

fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }

        // Toggle hint with configurable hotkey
        let hotkey = HOTKEY_DISPLAY
            .lock()
            .map(|s| s.clone())
            .unwrap_or_else(|_| "Ctrl+Shift+B".to_string());
        let toggle_hint = wide_str(&format!("Toggle: {}", hotkey));
        AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, toggle_hint.as_ptr());

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // Color selection
        let select_color = wide_str("Select Color...");
        AppendMenuW(
            menu,
            MF_STRING,
            MENU_SELECT_COLOR as usize,
            select_color.as_ptr(),
        );

        // Configure hotkey
        let configure_hotkey = wide_str("Configure Hotkey...");
        AppendMenuW(
            menu,
            MF_STRING,
            MENU_CONFIGURE_HOTKEY as usize,
            configure_hotkey.as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // Run at startup
        let startup_text = wide_str("Run at Startup");
        let startup_flags = if startup::is_startup_enabled() {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING
        };
        AppendMenuW(
            menu,
            startup_flags,
            MENU_STARTUP as usize,
            startup_text.as_ptr(),
        );

        AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

        // Exit
        let exit = wide_str("Exit");
        AppendMenuW(menu, MF_STRING, MENU_EXIT as usize, exit.as_ptr());

        let mut pt: POINT = zeroed();
        GetCursorPos(&mut pt);

        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN,
            pt.x,
            pt.y,
            0,
            hwnd,
            null_mut(),
        );
        DestroyMenu(menu);
    }
}

fn wide_str(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = lparam as u32;
            if event == WM_RBUTTONUP {
                show_context_menu(hwnd);
            } else if event == WM_LBUTTONDBLCLK {
                if let Some(ref cb) = TRAY_CALLBACK {
                    cb(TrayEvent::DoubleClick);
                }
            }
            0
        }
        WM_COMMAND => {
            let menu_id = (wparam & 0xFFFF) as u16;
            if let Some(ref cb) = TRAY_CALLBACK {
                match menu_id {
                    MENU_SELECT_COLOR => cb(TrayEvent::SelectColor),
                    MENU_CONFIGURE_HOTKEY => cb(TrayEvent::ConfigureHotkey),
                    MENU_STARTUP => cb(TrayEvent::ToggleStartup),
                    MENU_EXIT => cb(TrayEvent::Exit),
                    _ => {}
                }
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
