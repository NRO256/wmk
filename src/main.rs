#![windows_subsystem = "windows"]

mod domain;
mod platform;
mod ui;

fn main() {
    let code = ui::run_app();
    if code != 0 {
        std::process::exit(code);
    }
}
