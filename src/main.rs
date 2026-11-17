// Entry point. Keeps main.rs tiny on purpose -- window chrome and sizing
// live here, everything else (state, UI, search backend) lives in its own module.

mod app;
mod everything;
mod settings;

use app::RovylApp;

fn main() {
    // Compact, resizable popup window for now -- the Raycast look (borderless,
    // centered, always-on-top, summoned by a global hotkey) comes once the
    // layout itself is settled; easier to iterate with normal window chrome.
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 440.0)),
        min_window_size: Some(egui::vec2(480.0, 320.0)),
        resizable: true,
        decorated: true,
        transparent: false,
        ..Default::default()
    };

    eframe::run_native(
        "Rovyl Search",
        options,
        Box::new(|cc| Box::new(RovylApp::new(cc))),
    );
}
