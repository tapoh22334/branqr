#![windows_subsystem = "windows"]

mod app;
mod color_picker;
mod color_window;
mod monitor;
mod tray;

use app::App;

fn main() {
    let app = App::new();
    app.run();
}
