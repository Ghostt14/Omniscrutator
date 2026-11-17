// Entry point. Keeps main.rs tiny on purpose -- window chrome and sizing
// live here, everything else (state, UI, search backend, math) lives in its
// own module.

mod app;
mod everything;
mod mathexpr;
mod settings;

use app::OmniscrutatorApp;

fn main() -> eframe::Result<()> {
    // Compact, resizable popup window for now -- the Raycast look (borderless,
    // centered, always-on-top, summoned by a global hotkey, with a tray icon
    // for the real right-click menu) comes once the layout itself is settled;
    // easier to iterate with normal window chrome.
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 440.0])
            .with_min_inner_size([480.0, 320.0])
            .with_resizable(true)
            .with_decorations(true)
            .with_transparent(false),
        ..Default::default()
    };

    eframe::run_native(
        "Omniscrutator",
        options,
        Box::new(|cc| Ok(Box::new(OmniscrutatorApp::new(cc)))),
    )
}
