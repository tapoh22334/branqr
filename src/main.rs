#![windows_subsystem = "windows"]

mod app;
mod color_picker;
mod color_window;
mod config;
mod hotkey_dialog;
mod monitor;
mod startup;
mod tray;

use app::App;
use config::Config;

fn main() {
    // Enable startup on first run
    startup::ensure_startup_enabled();

    let config = Config::load();
    let app = App::new(config);
    app.run();
}
