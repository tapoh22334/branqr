use std::cell::RefCell;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::rc::Rc;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetStockObject, WHITE_BRUSH};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SetFocus, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetWindowLongPtrW, KillTimer, MessageBoxW, PostQuitMessage, RegisterClassW,
    SetTimer, SetWindowLongPtrW, SetWindowTextW, ShowWindow, TranslateMessage,
    CW_USEDEFAULT, GWLP_USERDATA, MB_ICONWARNING, MB_OK, MSG, SW_SHOW, WM_CLOSE,
    WM_COMMAND, WM_CREATE, WM_DESTROY, WM_TIMER, WNDCLASSW, WS_CAPTION, WS_CHILD,
    WS_EX_DLGMODALFRAME, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

// Window styles for controls
const BS_DEFPUSHBUTTON: u32 = 0x00000001;
const ES_CENTER: u32 = 0x0001;
const ES_READONLY: u32 = 0x0800;
const SS_CENTER: u32 = 0x0001;

const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;
const MOD_WIN: u32 = 0x0008;

const ID_OK: u16 = 1;
const ID_CANCEL: u16 = 2;
const TIMER_ID: usize = 1;

static CLASS_NAME: &[u16] = &[
    'B' as u16, 'l' as u16, 'a' as u16, 'n' as u16, 'q' as u16, 'r' as u16,
    'H' as u16, 'o' as u16, 't' as u16, 'k' as u16, 'e' as u16, 'y' as u16, 0,
];

struct DialogState {
    modifiers: u32,
    key: u32,
    confirmed: bool,
    hwnd_edit: HWND,
    last_key: u32,
}

pub fn show_hotkey_dialog(current_modifiers: u32, current_key: u32) -> Option<(u32, u32)> {
    unsafe {
        let hinstance = GetModuleHandleW(null_mut());
        if hinstance.is_null() {
            return None;
        }

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(dialog_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: GetStockObject(WHITE_BRUSH) as _,
            lpszMenuName: null_mut(),
            lpszClassName: CLASS_NAME.as_ptr(),
        };

        RegisterClassW(&wc);

        let state = Rc::new(RefCell::new(DialogState {
            modifiers: current_modifiers,
            key: current_key,
            confirmed: false,
            hwnd_edit: null_mut(),
            last_key: 0,
        }));

        let title = wide_str("Configure Hotkey");
        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            CLASS_NAME.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            320,
            180,
            null_mut(),
            null_mut(),
            hinstance,
            Rc::into_raw(state.clone()) as *mut _,
        );

        if hwnd.is_null() {
            return None;
        }

        ShowWindow(hwnd, SW_SHOW);
        SetFocus(hwnd);

        // Message loop
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let state = state.borrow();
        if state.confirmed && state.key != 0 {
            Some((state.modifiers, state.key))
        } else {
            None
        }
    }
}

fn wide_str(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn format_hotkey(modifiers: u32, key: u32) -> String {
    let mut parts = Vec::new();
    if modifiers & MOD_CONTROL != 0 {
        parts.push("Ctrl");
    }
    if modifiers & MOD_ALT != 0 {
        parts.push("Alt");
    }
    if modifiers & MOD_SHIFT != 0 {
        parts.push("Shift");
    }
    if modifiers & MOD_WIN != 0 {
        parts.push("Win");
    }

    let key_name = match key {
        0x41..=0x5A => (key as u8 as char).to_string(),
        0x30..=0x39 => (key as u8 as char).to_string(),
        0x70..=0x7B => format!("F{}", key - 0x6F),
        0 => String::new(),
        _ => format!("0x{:02X}", key),
    };

    if !key_name.is_empty() {
        parts.push(&key_name);
    }

    if parts.is_empty() {
        "Press a key...".to_string()
    } else {
        parts.join("+")
    }
}

fn check_key_state() -> Option<(u32, u32)> {
    unsafe {
        // Check modifiers
        let mut modifiers = 0u32;
        if GetAsyncKeyState(VK_CONTROL as i32) < 0 {
            modifiers |= MOD_CONTROL;
        }
        if GetAsyncKeyState(VK_MENU as i32) < 0 {
            modifiers |= MOD_ALT;
        }
        if GetAsyncKeyState(VK_SHIFT as i32) < 0 {
            modifiers |= MOD_SHIFT;
        }
        if GetAsyncKeyState(VK_LWIN as i32) < 0 || GetAsyncKeyState(VK_RWIN as i32) < 0 {
            modifiers |= MOD_WIN;
        }

        // Check A-Z
        for vk in 0x41i32..=0x5A {
            if GetAsyncKeyState(vk) < 0 {
                return Some((modifiers, vk as u32));
            }
        }

        // Check 0-9
        for vk in 0x30i32..=0x39 {
            if GetAsyncKeyState(vk) < 0 {
                return Some((modifiers, vk as u32));
            }
        }

        // Check F1-F12
        for vk in 0x70i32..=0x7B {
            if GetAsyncKeyState(vk) < 0 {
                return Some((modifiers, vk as u32));
            }
        }

        None
    }
}

unsafe extern "system" fn dialog_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = &*(lparam as *const windows_sys::Win32::UI::WindowsAndMessaging::CREATESTRUCTW);
            let state = cs.lpCreateParams as *mut RefCell<DialogState>;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);

            let hinstance = GetModuleHandleW(null_mut());

            // Label
            let label = wide_str("Press your desired hotkey combination:");
            let static_class = wide_str("STATIC");
            CreateWindowExW(
                0,
                static_class.as_ptr(),
                label.as_ptr(),
                WS_CHILD | WS_VISIBLE | SS_CENTER,
                10, 15, 280, 20,
                hwnd,
                null_mut(),
                hinstance,
                null_mut(),
            );

            // Hotkey display (readonly edit)
            let edit_class = wide_str("EDIT");
            let state_ref = &*state;
            let state_borrow = state_ref.borrow();
            let initial_text = wide_str(&format_hotkey(state_borrow.modifiers, state_borrow.key));
            drop(state_borrow);

            let hwnd_edit = CreateWindowExW(
                0,
                edit_class.as_ptr(),
                initial_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_CENTER | ES_READONLY,
                40, 50, 220, 30,
                hwnd,
                null_mut(),
                hinstance,
                null_mut(),
            );

            state_ref.borrow_mut().hwnd_edit = hwnd_edit;

            // OK button
            let button_class = wide_str("BUTTON");
            let ok_text = wide_str("OK");
            CreateWindowExW(
                0,
                button_class.as_ptr(),
                ok_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON,
                60, 100, 80, 30,
                hwnd,
                ID_OK as isize as _,
                hinstance,
                null_mut(),
            );

            // Cancel button
            let cancel_text = wide_str("Cancel");
            CreateWindowExW(
                0,
                button_class.as_ptr(),
                cancel_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                160, 100, 80, 30,
                hwnd,
                ID_CANCEL as isize as _,
                hinstance,
                null_mut(),
            );

            // Start timer to poll key state
            SetTimer(hwnd, TIMER_ID, 50, None);

            0
        }
        WM_TIMER => {
            if wparam == TIMER_ID {
                let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RefCell<DialogState>;
                if !state_ptr.is_null() {
                    if let Some((modifiers, key)) = check_key_state() {
                        let state_ref = &*state_ptr;
                        let mut state = state_ref.borrow_mut();

                        // Only update if key changed (avoid flickering)
                        if state.last_key != key {
                            state.modifiers = modifiers;
                            state.key = key;
                            state.last_key = key;

                            // Update display
                            let text = wide_str(&format_hotkey(modifiers, key));
                            SetWindowTextW(state.hwnd_edit, text.as_ptr());
                        }
                    } else {
                        // No key pressed, reset last_key
                        let state_ref = &*state_ptr;
                        state_ref.borrow_mut().last_key = 0;
                    }
                }
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u16;
            match id {
                ID_OK => {
                    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RefCell<DialogState>;
                    if !state_ptr.is_null() {
                        let state_ref = &*state_ptr;
                        let mut state = state_ref.borrow_mut();
                        if state.key == 0 {
                            let title = wide_str("Warning");
                            let msg_text = wide_str("Please press a valid key combination.");
                            drop(state);
                            MessageBoxW(hwnd, msg_text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONWARNING);
                        } else {
                            state.confirmed = true;
                            drop(state);
                            KillTimer(hwnd, TIMER_ID);
                            DestroyWindow(hwnd);
                        }
                    }
                }
                ID_CANCEL => {
                    KillTimer(hwnd, TIMER_ID);
                    DestroyWindow(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            KillTimer(hwnd, TIMER_ID);
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
