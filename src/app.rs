//! Desktop app state + egui UI
use crate::preview::{LineKind, PreviewCache};
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct MdEditorApp {
    source: String,
    path: Option<PathBuf>,
    dirty: bool,
    preview: PreviewCache,
    last_reuse: usize,
    last_render: usize,
    need_refresh: bool,
    last_edit: Instant,
    status: String,
    find_open: bool,
    find_query: String,
    replace_with: String,
    split: f32,
    dark: bool,
    quit_confirm: bool,
}

impl MdEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        cc.egui_ctx.set_style(style);

        let (source, path) = if let Some(p) = path {
            (std::fs::read_to_string(&p).unwrap_or_default(), Some(p))
        } else {
            (
                String::from(
                    "# rust-md-editor\n\n\
桌面版 **Markdown** 编辑器（egui）。\n\n\
- 增量预览\n\
- 代码块高亮\n\
- 打开 / 保存\n\n\
```rust\n\
fn main() {\n\
    println!(\"Hello\");\n\
}\n\
```\n",
                ),
                None,
            )
        };

        let mut app = Self {
            source,
            path,
            dirty: false,
            preview: PreviewCache::new(),
            last_reuse: 0,
            last_render: 0,
            need_refresh: true,
            last_edit: Instant::now(),
            status: "Ready · Ctrl+S save · Ctrl+O open · Ctrl+F find".into(),
            find_open: false,
            find_query: String::new(),
            replace_with: String::new(),
            split: 0.5,
            dark: true,
            quit_confirm: false,
        };
        app.refresh_preview();
        app
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_style(style);
    }

    fn file_title(&self) -> String {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("untitled.md");
        if self.dirty {
            format!("{name} * — rust-md-editor")
        } else {
            format!("{name} — rust-md-editor")
        }
    }

    fn refresh_preview(&mut self) {
        let (r, n) = self.preview.update(&self.source);
        self.last_reuse = r;
        self.last_render = n;
        self.need_refresh = false;
        if !self.quit_confirm {
            self.status = format!("blocks: {r} reused / {n} rendered");
        }
    }

    fn save(&mut self) {
        if let Some(ref path) = self.path {
            if let Err(e) = std::fs::write(path, &self.source) {
                self.status = format!("Save error: {e}");
                return;
            }
            self.dirty = false;
            self.quit_confirm = false;
            self.status = format!("Saved: {}", path.display());
        } else {
            self.save_as();
        }
    }

    fn save_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown", "txt"])
            .save_file()
        {
            if let Err(e) = std::fs::write(&path, &self.source) {
                self.status = format!("Save error: {e}");
                return;
            }
            self.path = Some(path.clone());
            self.dirty = false;
            self.quit_confirm = false;
            self.status = format!("Saved: {}", path.display());
        }
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown", "txt"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(s) => {
                    self.source = s;
                    self.path = Some(path);
                    self.dirty = false;
                    self.quit_confirm = false;
                    self.need_refresh = true;
                    self.status = "Opened".into();
                }
                Err(e) => self.status = format!("Open error: {e}"),
            }
        }
    }

    fn request_quit(&mut self, ctx: &egui::Context) {
        if !self.dirty || self.quit_confirm {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            self.quit_confirm = true;
            self.status = "Unsaved — quit again to discard, or Ctrl+S to save".into();
        }
    }

    fn find_next(&mut self) {
        let q = self.find_query.to_lowercase();
        if q.is_empty() {
            self.status = "Find: empty query".into();
            return;
        }
        let lower = self.source.to_lowercase();
        if let Some(idx) = lower.find(&q) {
            let line = self.source[..idx].bytes().filter(|&b| b == b'\n').count() + 1;
            self.status = format!("Found near line {line} (byte {idx})");
        } else {
            self.status = "No match".into();
        }
    }

    fn replace_all(&mut self) {
        if self.find_query.is_empty() {
            return;
        }
        let q = self.find_query.to_lowercase();
        let mut out = String::new();
        let mut rest = self.source.as_str();
        let rest_l = self.source.to_lowercase();
        let mut rest_l_ref = rest_l.as_str();
        let mut n = 0;
        while let Some(pos) = rest_l_ref.find(&q) {
            out.push_str(&rest[..pos]);
            out.push_str(&self.replace_with);
            rest = &rest[pos + q.len()..];
            rest_l_ref = &rest_l_ref[pos + q.len()..];
            n += 1;
        }
        out.push_str(rest);
        self.source = out;
        self.dirty = true;
        self.need_refresh = true;
        self.last_edit = Instant::now();
        self.status = format!("Replaced {n}");
    }
}

impl eframe::App for MdEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.need_refresh && self.last_edit.elapsed() > Duration::from_millis(120) {
            self.refresh_preview();
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.file_title()));

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open…    Ctrl+O").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    if ui.button("Save     Ctrl+S").clicked() {
                        self.save();
                        ui.close_menu();
                    }
                    if ui.button("Save As…").clicked() {
                        self.save_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        self.request_quit(ctx);
                        ui.close_menu();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Find / Replace    Ctrl+F").clicked() {
                        self.find_open = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.add(egui::Slider::new(&mut self.split, 0.2..=0.8).text("Split"));
                    if ui
                        .button(if self.dark {
                            "Light theme"
                        } else {
                            "Dark theme"
                        })
                        .clicked()
                    {
                        self.dark = !self.dark;
                        self.apply_theme(ctx);
                        ui.close_menu();
                    }
                });
                ui.separator();
                ui.label(if self.dirty {
                    egui::RichText::new("● modified").color(egui::Color32::YELLOW)
                } else {
                    egui::RichText::new("○ saved").color(egui::Color32::GRAY)
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "{} chars · {} lines",
                        self.source.len(),
                        self.source.lines().count().max(1)
                    ));
                });
            });
        });

        if self.find_open {
            egui::Window::new("Find / Replace")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Find:");
                        ui.text_edit_singleline(&mut self.find_query);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Replace:");
                        ui.text_edit_singleline(&mut self.replace_with);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Find").clicked() {
                            self.find_next();
                        }
                        if ui.button("Replace all").clicked() {
                            self.replace_all();
                        }
                        if ui.button("Close").clicked() {
                            self.find_open = false;
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let full = ui.available_width();
            let left_w = full * self.split;

            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(left_w - 4.0, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Source").strong());
                        egui::ScrollArea::both()
                            .id_salt("editor_scroll")
                            .show(ui, |ui| {
                                let response = ui.add(
                                    egui::TextEdit::multiline(&mut self.source)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(40),
                                );
                                if response.changed() {
                                    self.dirty = true;
                                    self.quit_confirm = false;
                                    self.need_refresh = true;
                                    self.last_edit = Instant::now();
                                }
                            });
                    });
                });

                ui.separator();

                ui.allocate_ui(egui::vec2(full - left_w - 4.0, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Preview ({} blocks · {} reused)",
                                self.preview.blocks.len(),
                                self.last_reuse
                            ))
                            .strong(),
                        );
                        egui::ScrollArea::vertical()
                            .id_salt("preview_scroll")
                            .show(ui, |ui| {
                                for line in self.preview.all_lines() {
                                    let rich = match line.kind {
                                        LineKind::Heading(1) => egui::RichText::new(&line.text)
                                            .size(26.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(200, 160, 255)),
                                        LineKind::Heading(2) => egui::RichText::new(&line.text)
                                            .size(22.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(140, 180, 255)),
                                        LineKind::Heading(_) => egui::RichText::new(&line.text)
                                            .size(18.0)
                                            .strong()
                                            .color(egui::Color32::from_rgb(120, 220, 220)),
                                        LineKind::Code => egui::RichText::new(&line.text)
                                            .monospace()
                                            .color(egui::Color32::from_rgb(180, 230, 180)),
                                        LineKind::Quote => {
                                            egui::RichText::new(format!("│ {}", line.text))
                                                .italics()
                                                .color(egui::Color32::from_rgb(140, 200, 140))
                                        }
                                        LineKind::Table => egui::RichText::new(&line.text)
                                            .monospace()
                                            .color(egui::Color32::LIGHT_BLUE),
                                        LineKind::Meta => egui::RichText::new(&line.text)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                        LineKind::Normal => egui::RichText::new(&line.text),
                                    };
                                    ui.label(rich);
                                }
                            });
                    });
                });
            });
        });

        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open_file();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
            self.find_open = true;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Q)) {
            self.request_quit(ctx);
        }

        if self.need_refresh {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}
