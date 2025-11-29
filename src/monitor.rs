use std::mem::zeroed;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MonitorInfo {
    pub hmonitor: HMONITOR,
    pub rect: Rect,
    pub is_primary: bool,
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

impl From<RECT> for Rect {
    fn from(r: RECT) -> Self {
        Rect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();
    let monitors_ptr = &mut monitors as *mut Vec<MonitorInfo>;

    unsafe {
        EnumDisplayMonitors(
            0 as HDC,
            null_mut(),
            Some(enum_monitor_callback),
            monitors_ptr as LPARAM,
        );
    }

    monitors
}

unsafe extern "system" fn enum_monitor_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam as *mut Vec<MonitorInfo>);

    let mut monitor_info: MONITORINFOEXW = zeroed();
    monitor_info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut monitor_info as *mut _ as *mut _) != 0 {
        let name = String::from_utf16_lossy(
            &monitor_info.szDevice[..monitor_info
                .szDevice
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(monitor_info.szDevice.len())],
        );

        let is_primary = (monitor_info.monitorInfo.dwFlags & 1) != 0;

        monitors.push(MonitorInfo {
            hmonitor,
            rect: monitor_info.monitorInfo.rcMonitor.into(),
            is_primary,
            name,
        });
    }

    TRUE
}
