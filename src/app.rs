use crate::color_picker::show_color_picker;
use crate::color_window::{set_hide_callback, ColorWindow};
use crate::config::{Config, HotkeyConfig};
use crate::hotkey_dialog::show_hotkey_dialog;
use crate::monitor::enumerate_monitors;
use crate::startup;
use crate::tray::{update_hotkey_display, TrayEvent, TrayIcon};
use std::cell::RefCell;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::rc::Rc;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostQuitMessage, TranslateMessage, MSG, WM_HOTKEY,
};

const DEFAULT_COLOR: u32 = 0x00000000; // Black
const HOTKEY_TOGGLE: i32 = 1;

pub struct App {
    windows: Rc<RefCell<Vec<ColorWindow>>>,
    color: Rc<RefCell<u32>>,
    visible: Rc<RefCell<bool>>,
    hotkey: Rc<RefCell<HotkeyConfig>>,
}

impl App {
    pub fn new(config: Config) -> Self {
        App {
            windows: Rc::new(RefCell::new(Vec::new())),
            color: Rc::new(RefCell::new(DEFAULT_COLOR)),
            visible: Rc::new(RefCell::new(false)),
            hotkey: Rc::new(RefCell::new(config.hotkey)),
        }
    }

    pub fn run(self) {
        // Register global hotkey
        let hotkey = self.hotkey.borrow();
        unsafe {
            RegisterHotKey(
                null_mut(),
                HOTKEY_TOGGLE,
                hotkey.modifiers,
                hotkey.key,
            );
        }

        let hotkey_display = hotkey.display();
        drop(hotkey);

        let windows_clone = Rc::clone(&self.windows);
        let color_clone = Rc::clone(&self.color);
        let visible_clone = Rc::clone(&self.visible);

        let windows_for_hide = Rc::clone(&self.windows);
        let visible_for_hide = Rc::clone(&self.visible);

        set_hide_callback(move || {
            hide_all(&windows_for_hide, &visible_for_hide);
        });

        let color_for_menu = Rc::clone(&self.color);
        let windows_for_menu = Rc::clone(&self.windows);
        let hotkey_for_menu = Rc::clone(&self.hotkey);

        let _tray = TrayIcon::new(&hotkey_display, move |event| match event {
            TrayEvent::DoubleClick => {
                toggle(&windows_clone, &color_clone, &visible_clone);
            }
            TrayEvent::SelectColor => {
                let current = *color_for_menu.borrow();
                if let Some(new_color) = show_color_picker(current) {
                    *color_for_menu.borrow_mut() = new_color;
                    update_color(&windows_for_menu, new_color);
                }
            }
            TrayEvent::ConfigureHotkey => {
                let current = hotkey_for_menu.borrow();
                let (mods, key) = (current.modifiers, current.key);
                drop(current);

                if let Some((new_mods, new_key)) = show_hotkey_dialog(mods, key) {
                    // Unregister old hotkey
                    unsafe {
                        UnregisterHotKey(null_mut(), HOTKEY_TOGGLE);
                    }

                    // Update and register new hotkey
                    let mut hotkey = hotkey_for_menu.borrow_mut();
                    hotkey.modifiers = new_mods;
                    hotkey.key = new_key;

                    unsafe {
                        RegisterHotKey(null_mut(), HOTKEY_TOGGLE, new_mods, new_key);
                    }

                    // Update display and save config
                    let display = hotkey.display();
                    update_hotkey_display(&display);

                    let config = Config {
                        hotkey: hotkey.clone(),
                    };
                    let _ = config.save();
                }
            }
            TrayEvent::ToggleStartup => {
                let enabled = startup::is_startup_enabled();
                startup::set_startup_enabled(!enabled);
            }
            TrayEvent::Exit => {
                unsafe {
                    UnregisterHotKey(null_mut(), HOTKEY_TOGGLE);
                    PostQuitMessage(0);
                }
            }
        });

        // Message loop with hotkey handling
        let windows_for_hotkey = Rc::clone(&self.windows);
        let color_for_hotkey = Rc::clone(&self.color);
        let visible_for_hotkey = Rc::clone(&self.visible);

        message_loop(windows_for_hotkey, color_for_hotkey, visible_for_hotkey);
    }
}

fn toggle(
    windows: &Rc<RefCell<Vec<ColorWindow>>>,
    color: &Rc<RefCell<u32>>,
    visible: &Rc<RefCell<bool>>,
) {
    let is_visible = *visible.borrow();
    if is_visible {
        hide_all(windows, visible);
    } else {
        show_all(windows, color, visible);
    }
}

fn show_all(
    windows: &Rc<RefCell<Vec<ColorWindow>>>,
    color: &Rc<RefCell<u32>>,
    visible: &Rc<RefCell<bool>>,
) {
    let current_color = *color.borrow();
    let monitors = enumerate_monitors();
    let mut wins = windows.borrow_mut();

    // Clear existing windows and recreate for current monitor configuration
    wins.clear();
    for monitor in &monitors {
        if let Some(window) = ColorWindow::new(monitor, current_color) {
            wins.push(window);
        }
    }

    for window in wins.iter() {
        window.show();
    }

    *visible.borrow_mut() = true;
}

fn hide_all(windows: &Rc<RefCell<Vec<ColorWindow>>>, visible: &Rc<RefCell<bool>>) {
    for window in windows.borrow().iter() {
        window.hide();
    }
    *visible.borrow_mut() = false;
}

fn update_color(windows: &Rc<RefCell<Vec<ColorWindow>>>, color: u32) {
    for window in windows.borrow().iter() {
        window.set_color(color);
    }
}

fn message_loop(
    windows: Rc<RefCell<Vec<ColorWindow>>>,
    color: Rc<RefCell<u32>>,
    visible: Rc<RefCell<bool>>,
) {
    unsafe {
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY && msg.wParam == HOTKEY_TOGGLE as usize {
                toggle(&windows, &color, &visible);
            } else {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
