#![windows_subsystem = "windows"]

mod app;
mod color_picker;
mod color_window;
mod config;
mod monitor;
mod tray;

use app::App;
use config::Config;

fn main() {
    let config = Config::load();
    let app = App::new(config);
    app.run();
}
