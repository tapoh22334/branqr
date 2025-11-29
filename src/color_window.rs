use crate::monitor::MonitorInfo;
use std::mem::zeroed;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, RegisterClassW,
    SetWindowLongPtrW, SetWindowPos, ShowCursor, ShowWindow, CS_HREDRAW, CS_VREDRAW,
    GWLP_USERDATA, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW,
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_PAINT, WM_RBUTTONDOWN, WM_SETCURSOR, WNDCLASSW, WS_EX_TOPMOST,
    WS_POPUP,
};

static CLASS_NAME: &[u16] = &[
    'B' as u16, 'l' as u16, 'a' as u16, 'n' as u16, 'q' as u16, 'r' as u16, 0,
];

const VK_ESCAPE: i32 = 0x1B;

static mut HIDE_CALLBACK: Option<Box<dyn Fn()>> = None;

pub struct ColorWindow {
    hwnd: HWND,
}

impl ColorWindow {
    pub fn new(monitor: &MonitorInfo, color: u32) -> Option<Self> {
        unsafe {
            let hinstance = GetModuleHandleW(null_mut());
            if hinstance.is_null() {
                return None;
            }

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
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
                WS_EX_TOPMOST,
                CLASS_NAME.as_ptr(),
                CLASS_NAME.as_ptr(),
                WS_POPUP,
                monitor.rect.left,
                monitor.rect.top,
                monitor.rect.width(),
                monitor.rect.height(),
                null_mut(),
                null_mut(),
                hinstance,
                null_mut(),
            );

            if hwnd.is_null() {
                return None;
            }

            let window = ColorWindow { hwnd };
            window.set_color(color);

            Some(window)
        }
    }

    pub fn show(&self) {
        unsafe {
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST as HWND,
                0,
                0,
                0,
                0,
                SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE,
            );
            ShowWindow(self.hwnd, SW_SHOW);
            ShowCursor(0);
        }
    }

    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
            ShowCursor(1);
        }
    }

    pub fn set_color(&self, color: u32) {
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, color as isize);
            InvalidateRect(self.hwnd, null_mut(), 1);
        }
    }

    pub fn destroy(&self) {
        unsafe {
            DestroyWindow(self.hwnd);
        }
    }
}

impl Drop for ColorWindow {
    fn drop(&mut self) {
        self.destroy();
    }
}

pub fn set_hide_callback<F: Fn() + 'static>(callback: F) {
    unsafe {
        HIDE_CALLBACK = Some(Box::new(callback));
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let color = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as u32;
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            let brush = CreateSolidBrush(color);
            FillRect(hdc, &ps.rcPaint, brush);
            DeleteObject(brush as _);
            EndPaint(hwnd, &ps);
            0
        }
        WM_SETCURSOR => {
            SetWindowLongPtrW(hwnd, -20, 0);
            1
        }
        WM_KEYDOWN => {
            if wparam as i32 == VK_ESCAPE {
                if let Some(ref cb) = HIDE_CALLBACK {
                    cb();
                }
            }
            0
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            if let Some(ref cb) = HIDE_CALLBACK {
                cb();
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
