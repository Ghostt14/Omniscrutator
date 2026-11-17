// The UI itself. Two views: the search popup (default) and a settings panel.
//
// Changes from the first skeleton pass, per feedback:
// - No visible filter tabs. Search is always "All"; what gets sorted to the
//   front is a Settings-only preference (`ResultPriority`).
// - No gear button / keyboard shortcut for Settings. It's opened by typing
//   "settings" (shown as a command row, confirmed with Enter) or, as an
//   interim stand-in for "right-click the tray icon" (no tray icon exists
//   yet -- that's Win32-specific future work), right-clicking anywhere in
//   the window.
// - Quick math: if the query parses as an arithmetic expression, the
//   answer is shown as the first row, ahead of any file results.

use crate::everything::{EverythingBackend, MockBackend, SearchResult};
use crate::mathexpr;
use crate::settings::Settings;
use eframe::egui;

/// One row in the results list. Keeping this as an enum (rather than
/// shoehorning everything into `SearchResult`) is what makes it easy to mix
/// a math answer and a "Open Settings" command in with real file rows.
enum Row {
    Math(f64),
    Command(&'static str), // label shown; only "Open Settings" exists today
    File(SearchResult),
}

pub struct OmniscrutatorApp {
    query: String,
    results: Vec<Row>,
    selected: usize,
    backend: Box<dyn EverythingBackend>,
    settings: Settings,
    show_settings: bool,
    status_message: Option<String>,
}

impl OmniscrutatorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // TODO: once EverythingIpcBackend::search is wired up, pick it here
        // on Windows and fall back to MockBackend elsewhere. Staying on Mock
        // everywhere for now so the skeleton behaves identically on any OS.
        let backend: Box<dyn EverythingBackend> = Box::new(MockBackend::default());

        let mut app = Self {
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            backend,
            settings: Settings::load(),
            show_settings: false,
            status_message: None,
        };
        app.run_search();
        app
    }

    fn run_search(&mut self) {
        self.selected = 0;
        let trimmed = self.query.trim();

        // 1. Math takes priority over everything else, including the
        // "open settings" command and any file search -- per spec, "4(5)-3"
        // should show 17 first, full stop.
        if let Some(value) = mathexpr::try_eval(trimmed) {
            self.results = vec![Row::Math(value)];
            return;
        }

        let mut rows = Vec::new();

        // 2. The "settings" command. Exact match (case-insensitive) rather
        // than a substring, so a real search for a file literally named
        // "settings.ini" still works normally alongside it.
        if trimmed.eq_ignore_ascii_case("settings") {
            rows.push(Row::Command("Open Settings"));
        }

        // 3. Normal file search, always unfiltered ("All"), then re-ordered
        // so matches of the user's chosen priority category come first.
        let mut files = self.backend.search(trimmed, self.settings.max_results);
        let priority = self.settings.result_priority;
        files.sort_by_key(|r| if priority.matches(r) { 0 } else { 1 });
        rows.extend(files.into_iter().map(Row::File));

        self.results = rows;
    }

    fn move_selection(&mut self, delta: isize) {
        if self.results.is_empty() {
            return;
        }
        let len = self.results.len() as isize;
        let new = (self.selected as isize + delta).rem_euclid(len);
        self.selected = new as usize;
    }

    fn activate_selected(&mut self, ctx: &egui::Context) {
        match self.results.get(self.selected) {
            Some(Row::Math(value)) => {
                ctx.copy_text(format!("{value}"));
                self.status_message = Some(format!("Copied result: {value}"));
            }
            Some(Row::Command("Open Settings")) => {
                self.show_settings = true;
            }
            Some(Row::Command(_)) => {}
            Some(Row::File(r)) => {
                // TODO: ShellExecuteW(path) on Windows. Just reporting for now.
                self.status_message = Some(format!("Would open: {}", r.full_path));
            }
            None => {}
        }
    }

    fn copy_selected_path(&mut self, ctx: &egui::Context) {
        if let Some(Row::File(r)) = self.results.get(self.selected) {
            ctx.copy_text(r.full_path.clone());
            self.status_message = Some(format!("Copied path: {}", r.full_path));
        }
    }

    fn is_web_search(&self) -> bool {
        !self.settings.web_search_prefix.is_empty()
            && self.query.starts_with(self.settings.web_search_prefix.as_str())
            && mathexpr::try_eval(self.query.trim()).is_none()
    }
}

impl eframe::App for OmniscrutatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global key handling: arrows to move selection, Enter to activate,
        // Ctrl+Enter to copy path, Esc to quit.
        let (esc, down, up, right_click) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::ArrowUp),
                i.pointer.secondary_clicked(),
            )
        });
        if esc {
            std::process::exit(0);
        }
        if down {
            self.move_selection(1);
        }
        if up {
            self.move_selection(-1);
        }
        if right_click {
            // Interim stand-in for "right-click the tray icon" -- there's no
            // tray icon yet (Win32-specific, future work), so right-clicking
            // anywhere in the window opens Settings for now.
            self.show_settings = true;
        }

        if self.show_settings {
            self.settings_view(ctx);
            return;
        }

        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let resp = ui.add_sized(
                    [ui.available_width() - 8.0, 34.0],
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search files, folders, apps... try \"4(5)-3\" or \"settings\"")
                        .font(egui::TextStyle::Heading),
                );
                if resp.changed() {
                    self.run_search();
                }
                resp.request_focus();
            });
            ui.add_space(6.0);
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.weak(self.backend.status());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(msg) = &self.status_message {
                        ui.label(msg);
                    } else {
                        ui.weak("Up/Down move  Enter activate  Ctrl+Enter copy path  right-click settings");
                    }
                });
            });
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_web_search() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    let term = self.query.trim_start_matches(self.settings.web_search_prefix.as_str());
                    ui.label(format!("Press Enter to search the web for \"{}\"", term.trim()));
                });
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in 0..self.results.len() {
                    let selected = i == self.selected;
                    let frame = egui::Frame::none()
                        .fill(if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .inner_margin(egui::Margin::symmetric(8.0, 6.0));
                    frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        match &self.results[i] {
                            Row::Math(value) => {
                                ui.horizontal(|ui| {
                                    ui.label("[=]");
                                    ui.label(
                                        egui::RichText::new(format!("{value}"))
                                            .strong()
                                            .size(22.0),
                                    );
                                    ui.weak("Enter to copy");
                                });
                            }
                            Row::Command(label) => {
                                ui.horizontal(|ui| {
                                    ui.label("[>]");
                                    ui.label(egui::RichText::new(*label).strong());
                                });
                            }
                            Row::File(r) => {
                                // TODO: real icons once past the layout-skeleton phase;
                                // egui's default font has no emoji glyphs.
                                let icon = if r.is_dir { "[DIR]" } else { "[FILE]" };
                                ui.horizontal(|ui| {
                                    ui.label(icon);
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&r.name).strong());
                                        ui.label(egui::RichText::new(&r.full_path).weak().small());
                                    });
                                });
                            }
                        }
                    });
                }
                if self.results.is_empty() {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| ui.weak("No results"));
                }
            });
        });

        let (enter, ctrl_enter) = ctx.input(|i| {
            let enter = i.key_pressed(egui::Key::Enter);
            (enter, enter && i.modifiers.ctrl)
        });
        if ctrl_enter {
            self.copy_selected_path(ctx);
        } else if enter {
            if self.is_web_search() {
                let term = self.query.trim_start_matches(self.settings.web_search_prefix.as_str()).trim();
                let url = self.settings.web_search_url.replace("{query}", &urlencoding_lite(term));
                self.status_message = Some(format!("Would open browser at: {url}"));
            } else {
                self.activate_selected(ctx);
            }
        }
    }
}

impl OmniscrutatorApp {
    fn settings_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.add_space(10.0);

            ui.label("What should be sorted to the front of results?");
            egui::ComboBox::from_id_salt("result_priority")
                .selected_text(self.settings.result_priority.label())
                .show_ui(ui, |ui| {
                    for p in crate::everything::ResultPriority::ALL {
                        ui.selectable_value(&mut self.settings.result_priority, p, p.label());
                    }
                });
            ui.add_space(8.0);

            ui.label("Summon hotkey");
            ui.text_edit_singleline(&mut self.settings.hotkey);
            ui.add_space(8.0);

            ui.label("Copy-path hotkey");
            ui.text_edit_singleline(&mut self.settings.copy_path_hotkey);
            ui.add_space(8.0);

            ui.label("Max results shown");
            ui.add(egui::Slider::new(&mut self.settings.max_results, 10..=200));
            ui.add_space(8.0);

            ui.label("Web search prefix (type this to search the web instead)");
            ui.text_edit_singleline(&mut self.settings.web_search_prefix);
            ui.add_space(8.0);

            ui.label("Web search URL (\"{query}\" is replaced)");
            ui.text_edit_singleline(&mut self.settings.web_search_url);
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.settings.save();
                    self.status_message = Some("Settings saved".to_string());
                }
                if ui.button("Back").clicked() {
                    self.show_settings = false;
                }
            });
        });
    }
}

// Minimal placeholder -- swap for the `urlencoding` crate once we wire up
// actually opening a browser; good enough for displaying a preview URL now.
fn urlencoding_lite(s: &str) -> String {
    s.replace(' ', "+")
}
