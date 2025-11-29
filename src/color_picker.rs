use std::mem::zeroed;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, CreateRoundRectRgn, CreateSolidBrush, DeleteObject, EndPaint, FillRect,
    FrameRect, InvalidateRect, RoundRect, SelectObject, SetBkMode, SetTextColor, SetWindowRgn,
    TextOutW, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::Dialogs::{
    ChooseColorW, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
    GetSystemMetrics, IsWindow, RegisterClassW, SetWindowPos, ShowWindow, TranslateMessage,
    CS_HREDRAW, CS_VREDRAW, HWND_TOPMOST, MSG, SM_CXSCREEN, SM_CYSCREEN, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, SW_SHOW, WM_CLOSE, WM_DESTROY, WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_PAINT,
    WNDCLASSW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

static CLASS_NAME: &[u16] = &[
    'B' as u16, 'l' as u16, 'a' as u16, 'n' as u16, 'q' as u16, 'r' as u16, 'P' as u16, 'i' as u16,
    'c' as u16, 'k' as u16, 'e' as u16, 'r' as u16, 0,
];

const WINDOW_WIDTH: i32 = 280;
const WINDOW_HEIGHT: i32 = 400;
const PADDING: i32 = 20;
const COLOR_SIZE: i32 = 50;
const COLOR_GAP: i32 = 12;
const CORNER_RADIUS: i32 = 16;
const CUSTOM_BUTTON_HEIGHT: i32 = 36;

pub const PRESET_COLORS: &[u32] = &[
    0x00000000, // Black
    0x00FFFFFF, // White
    0x00303030, // Dark Gray
    0x00808080, // Gray
    0x0040A0FF, // Amber
    0x0060C0FF, // Warm
    0x0080D0FF, // Soft
    0x00A0E0FF, // Cream
    0x00507DCD, // Sunset
    0x00889EDE, // Blush
    0x008EBCF0, // Peach
    0x00B5D8F5, // Apricot
    0x00CD9E7D, // Sky
    0x00E0C090, // Light Blue
    0x00F0D8A0, // Pale Blue
    0x00F5E6C8, // Ice
];

// Result codes stored in GWLP_USERDATA high bits
const RESULT_NONE: isize = 0;
const RESULT_PRESET: isize = 1;
const RESULT_CUSTOM: isize = 2;

struct PickerState {
    selected_color: u32,
    result_type: isize,
    hover_index: i32,
    hover_custom: bool,
}

static mut PICKER_STATE: PickerState = PickerState {
    selected_color: 0,
    result_type: RESULT_NONE,
    hover_index: -1,
    hover_custom: false,
};

pub fn show_color_picker(current_color: u32) -> Option<u32> {
    unsafe {
        PICKER_STATE = PickerState {
            selected_color: 0,
            result_type: RESULT_NONE,
            hover_index: -1,
            hover_custom: false,
        };

        let hinstance = GetModuleHandleW(null_mut());
        if hinstance.is_null() {
            return None;
        }

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(picker_window_proc),
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

        // Center on screen
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_width - WINDOW_WIDTH) / 2;
        let y = (screen_height - WINDOW_HEIGHT) / 2;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            CLASS_NAME.as_ptr(),
            null_mut(),
            WS_POPUP,
            x,
            y,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );

        if hwnd.is_null() {
            return None;
        }

        // Rounded corners
        let rgn = CreateRoundRectRgn(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT, CORNER_RADIUS, CORNER_RADIUS);
        SetWindowRgn(hwnd, rgn, 1);

        SetWindowPos(
            hwnd,
            HWND_TOPMOST as HWND,
            0,
            0,
            0,
            0,
            SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE,
        );
        ShowWindow(hwnd, SW_SHOW);

        // Message loop
        let mut msg: MSG = zeroed();
        loop {
            let ret = GetMessageW(&mut msg, null_mut(), 0, 0);
            if ret <= 0 {
                break;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);

            // Check if window was destroyed
            if IsWindow(hwnd) == 0 {
                break;
            }
        }

        // Check result
        match PICKER_STATE.result_type {
            RESULT_PRESET => Some(PICKER_STATE.selected_color),
            RESULT_CUSTOM => {
                // Show system color picker
                show_system_color_picker(current_color)
            }
            _ => None,
        }
    }
}

fn show_system_color_picker(current_color: u32) -> Option<u32> {
    unsafe {
        let mut custom_colors: [u32; 16] = [0x00FFFFFF; 16];
        let mut cc: CHOOSECOLORW = zeroed();
        cc.lStructSize = std::mem::size_of::<CHOOSECOLORW>() as u32;
        cc.hwndOwner = null_mut();
        cc.rgbResult = current_color;
        cc.lpCustColors = custom_colors.as_mut_ptr();
        cc.Flags = CC_FULLOPEN | CC_RGBINIT;

        if ChooseColorW(&mut cc) != 0 {
            Some(cc.rgbResult)
        } else {
            None
        }
    }
}

fn get_color_rect(index: usize) -> RECT {
    let cols = 4;
    let row = (index / cols) as i32;
    let col = (index % cols) as i32;

    let x = PADDING + col * (COLOR_SIZE + COLOR_GAP);
    let y = PADDING + 50 + row * (COLOR_SIZE + COLOR_GAP);

    RECT {
        left: x,
        top: y,
        right: x + COLOR_SIZE,
        bottom: y + COLOR_SIZE,
    }
}

fn get_custom_button_rect() -> RECT {
    let rows = PRESET_COLORS.len().div_ceil(4);
    let y = PADDING + 50 + (rows as i32) * (COLOR_SIZE + COLOR_GAP) + 10;

    RECT {
        left: PADDING,
        top: y,
        right: WINDOW_WIDTH - PADDING,
        bottom: y + CUSTOM_BUTTON_HEIGHT,
    }
}

fn hit_test(x: i32, y: i32) -> i32 {
    for (i, _) in PRESET_COLORS.iter().enumerate() {
        let rect = get_color_rect(i);
        if x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom {
            return i as i32;
        }
    }
    -1
}

fn hit_test_custom(x: i32, y: i32) -> bool {
    let rect = get_custom_button_rect();
    x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
}

unsafe extern "system" fn picker_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut client_rect: RECT = zeroed();
            GetClientRect(hwnd, &mut client_rect);

            // Background
            let bg_brush = CreateSolidBrush(0x00252525);
            FillRect(hdc, &client_rect, bg_brush);
            DeleteObject(bg_brush as _);

            // Title
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, 0x00FFFFFF);
            let title = wide_str("Color");
            TextOutW(hdc, PADDING, PADDING, title.as_ptr(), 5);

            // Color swatches
            for (i, &color) in PRESET_COLORS.iter().enumerate() {
                let rect = get_color_rect(i);
                let is_hover = PICKER_STATE.hover_index == i as i32;

                if is_hover {
                    let highlight_rect = RECT {
                        left: rect.left - 3,
                        top: rect.top - 3,
                        right: rect.right + 3,
                        bottom: rect.bottom + 3,
                    };
                    let highlight_brush = CreateSolidBrush(0x00808080);
                    FillRect(hdc, &highlight_rect, highlight_brush);
                    DeleteObject(highlight_brush as _);
                }

                let color_brush = CreateSolidBrush(color);
                let old_brush = SelectObject(hdc, color_brush as _);
                let pen = CreatePen(PS_SOLID, 1, 0x00404040);
                let old_pen = SelectObject(hdc, pen as _);

                RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, 8, 8);

                SelectObject(hdc, old_brush);
                SelectObject(hdc, old_pen);
                DeleteObject(color_brush as _);
                DeleteObject(pen as _);

                if color > 0x00C0C0C0 {
                    let border_brush = CreateSolidBrush(0x00606060);
                    FrameRect(hdc, &rect, border_brush);
                    DeleteObject(border_brush as _);
                }
            }

            // Custom color button
            let custom_rect = get_custom_button_rect();
            let btn_color = if PICKER_STATE.hover_custom { 0x00404040 } else { 0x00353535 };
            let btn_brush = CreateSolidBrush(btn_color);
            let old_brush = SelectObject(hdc, btn_brush as _);
            let pen = CreatePen(PS_SOLID, 1, 0x00505050);
            let old_pen = SelectObject(hdc, pen as _);

            RoundRect(
                hdc,
                custom_rect.left,
                custom_rect.top,
                custom_rect.right,
                custom_rect.bottom,
                8,
                8,
            );

            SelectObject(hdc, old_brush);
            SelectObject(hdc, old_pen);
            DeleteObject(btn_brush as _);
            DeleteObject(pen as _);

            SetTextColor(hdc, 0x00FFFFFF);
            let btn_text = wide_str("Custom Color...");
            let text_x = custom_rect.left + (custom_rect.right - custom_rect.left) / 2 - 50;
            let text_y = custom_rect.top + (CUSTOM_BUTTON_HEIGHT - 16) / 2;
            TextOutW(hdc, text_x, text_y, btn_text.as_ptr(), btn_text.len() as i32 - 1);

            EndPaint(hwnd, &ps);
            0
        }
        WM_MOUSEMOVE => {
            let x = (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            let new_hover = hit_test(x, y);
            let new_hover_custom = hit_test_custom(x, y);

            if new_hover != PICKER_STATE.hover_index || new_hover_custom != PICKER_STATE.hover_custom {
                PICKER_STATE.hover_index = new_hover;
                PICKER_STATE.hover_custom = new_hover_custom;
                InvalidateRect(hwnd, null_mut(), 0);
            }
            0
        }
        WM_LBUTTONDOWN => {
            let x = (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            let index = hit_test(x, y);

            if index >= 0 && (index as usize) < PRESET_COLORS.len() {
                PICKER_STATE.selected_color = PRESET_COLORS[index as usize];
                PICKER_STATE.result_type = RESULT_PRESET;
                DestroyWindow(hwnd);
            } else if hit_test_custom(x, y) {
                PICKER_STATE.result_type = RESULT_CUSTOM;
                DestroyWindow(hwnd);
            }
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn wide_str(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
