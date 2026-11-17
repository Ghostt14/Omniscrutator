// The UI itself. One screen, two views: the search popup (default) and a
// settings panel toggled with a gear button / Ctrl+,.

use crate::everything::{EverythingBackend, FilterMode, MockBackend, SearchResult};
use crate::settings::Settings;
use eframe::egui;

pub struct RovylApp {
    query: String,
    filter: FilterMode,
    results: Vec<SearchResult>,
    selected: usize,
    backend: Box<dyn EverythingBackend>,
    settings: Settings,
    show_settings: bool,
    status_message: Option<String>,
}

impl RovylApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // TODO: once EverythingIpcBackend::search is actually wired up, pick it
        // here on Windows (`#[cfg(windows)] Box::new(EverythingIpcBackend::default())`)
        // and fall back to MockBackend elsewhere. Staying on Mock everywhere for now
        // so the skeleton behaves identically regardless of platform.
        let backend: Box<dyn EverythingBackend> = Box::new(MockBackend::default());

        let mut app = Self {
            query: String::new(),
            filter: FilterMode::All,
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
        let full_query = format!("{}{}", self.filter.query_prefix(), self.query);
        self.results = self.backend.search(&full_query, self.settings.max_results);
        self.selected = 0;
    }

    fn move_selection(&mut self, delta: isize) {
        if self.results.is_empty() {
            return;
        }
        let len = self.results.len() as isize;
        let new = (self.selected as isize + delta).rem_euclid(len);
        self.selected = new as usize;
    }

    fn open_selected(&mut self) {
        if let Some(r) = self.results.get(self.selected) {
            // TODO: ShellExecuteW(path) on Windows. Just reporting for now.
            self.status_message = Some(format!("Would open: {}", r.full_path));
        }
    }

    fn copy_selected_path(&mut self, ctx: &egui::Context) {
        if let Some(r) = self.results.get(self.selected) {
            ctx.output().copied_text = r.full_path.clone();
            self.status_message = Some(format!("Copied path: {}", r.full_path));
        }
    }

    fn is_web_search(&self) -> bool {
        !self.settings.web_search_prefix.is_empty()
            && self.query.starts_with(self.settings.web_search_prefix.as_str())
    }
}

impl eframe::App for RovylApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Global key handling: arrows to move selection, Enter to open,
        // Ctrl+Enter to copy path, Esc to quit. Settings is opened via the
        // gear button for now; F2 toggles it too (Ctrl+, needs a key egui
        // 0.19 doesn't expose -- add it back once we're on a newer egui).
        let (esc, down, up, toggle_settings) = {
            let input = ctx.input();
            (
                input.key_pressed(egui::Key::Escape),
                input.key_pressed(egui::Key::ArrowDown),
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::F2),
            )
        };
        if esc {
            std::process::exit(0);
        }
        if down {
            self.move_selection(1);
        }
        if up {
            self.move_selection(-1);
        }
        if toggle_settings {
            self.show_settings = !self.show_settings;
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
                    [ui.available_width() - 40.0, 34.0],
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search files, folders, apps... (? to web search)")
                        .font(egui::TextStyle::Heading),
                );
                if resp.changed() {
                    self.run_search();
                }
                resp.request_focus();

                if ui.add(egui::Button::new("\u{2699}")).clicked() {
                    self.show_settings = true;
                }
            });
            ui.add_space(4.0);

            // Filter mode tabs.
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                for mode in FilterMode::ALL {
                    let selected = self.filter == mode;
                    if ui.selectable_label(selected, mode.label()).clicked() {
                        self.filter = mode;
                        self.run_search();
                    }
                }
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
                        ui.weak(format!("{} results  |  Up/Down move  Enter open  Ctrl+Enter copy path", self.results.len()));
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
                for (i, r) in self.results.iter().enumerate() {
                    let selected = i == self.selected;
                    // TODO: real icons (extracted .exe/.lnk icons, folder glyph) once
                    // we're past the layout-skeleton phase; egui's default font has
                    // no emoji glyphs, so plain bracketed text for now.
                    let icon = if r.is_dir { "[DIR]" } else { "[FILE]" };
                    let frame = egui::Frame::none()
                        .fill(if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .inner_margin(egui::style::Margin::symmetric(8.0, 6.0));
                    frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(icon);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&r.name).strong());
                                ui.label(egui::RichText::new(&r.full_path).weak().small());
                            });
                        });
                    });
                }
                if self.results.is_empty() {
                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| ui.weak("No results"));
                }
            });
        });

        // Handle Enter / Ctrl+Enter after drawing.
        let (enter, ctrl_enter) = {
            let input = ctx.input();
            let enter = input.key_pressed(egui::Key::Enter);
            (enter, enter && input.modifiers.ctrl)
        };
        if ctrl_enter {
            self.copy_selected_path(ctx);
        } else if enter {
            if self.is_web_search() {
                let term = self.query.trim_start_matches(self.settings.web_search_prefix.as_str()).trim();
                let url = self.settings.web_search_url.replace("{query}", &urlencoding_lite(term));
                self.status_message = Some(format!("Would open browser at: {url}"));
            } else {
                self.open_selected();
            }
        }
    }
}

impl RovylApp {
    fn settings_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.add_space(10.0);

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
