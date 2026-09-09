//! Desktop app state + egui UI
use crate::preview::{LineKind, PreviewCache};
use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use std::path::PathBuf;
use std::time::{Duration, Instant};

// —— palette ——
const BG: Color32 = Color32::from_rgb(22, 24, 28);
const PANEL: Color32 = Color32::from_rgb(30, 33, 39);
const PANEL2: Color32 = Color32::from_rgb(36, 40, 48);
const BORDER: Color32 = Color32::from_rgb(55, 60, 72);
const ACCENT: Color32 = Color32::from_rgb(88, 166, 255);
const TEXT: Color32 = Color32::from_rgb(230, 234, 242);
const MUTED: Color32 = Color32::from_rgb(140, 148, 164);
const OK: Color32 = Color32::from_rgb(126, 231, 135);
const WARN: Color32 = Color32::from_rgb(255, 200, 80);

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
        style.visuals.window_fill = PANEL;
        style.visuals.panel_fill = BG;
        style.visuals.extreme_bg_color = PANEL2;
        style.visuals.widgets.noninteractive.bg_fill = PANEL;
        style.visuals.widgets.inactive.bg_fill = PANEL2;
        style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(88, 166, 255, 60);
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.window_margin = Margin::same(12);
        cc.egui_ctx.set_style(style);

        let (source, path) = if let Some(p) = path {
            (std::fs::read_to_string(&p).unwrap_or_default(), Some(p))
        } else {
            (
                String::from(
                    "# rust-md-editor\n\n\
支持 **中文** 显示与输入。\n\n\
- `Enter`：确认输入法候选（不换行）\n\
- `Ctrl+Enter`：换行\n\
- `Ctrl+S` 保存 · `Ctrl+O` 打开 · `Ctrl+F` 查找\n\n\
```rust\n\
fn main() {\n\
    println!(\"你好\");\n\
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
            status: "Ready · Enter 确认输入 · Ctrl+Enter 换行".into(),
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
        if self.dark {
            style.visuals = egui::Visuals::dark();
            style.visuals.window_fill = PANEL;
            style.visuals.panel_fill = BG;
            style.visuals.extreme_bg_color = PANEL2;
        } else {
            style.visuals = egui::Visuals::light();
        }
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
            self.status = format!("blocks: {r} reused / {n} rendered · Ctrl+Enter 换行");
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
            self.status = format!("已保存: {}", path.display());
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
            self.status = format!("已保存: {}", path.display());
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
                    self.status = "已打开".into();
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
            self.status = "有未保存更改 — 再按一次退出丢弃，或 Ctrl+S 保存".into();
        }
    }

    fn find_next(&mut self) {
        let q = self.find_query.to_lowercase();
        if q.is_empty() {
            self.status = "查找：查询为空".into();
            return;
        }
        let lower = self.source.to_lowercase();
        if let Some(idx) = lower.find(&q) {
            let line = self.source[..idx].bytes().filter(|&b| b == b'\n').count() + 1;
            self.status = format!("找到 · 约第 {line} 行");
        } else {
            self.status = "未找到".into();
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
        self.status = format!("已替换 {n} 处");
    }

    fn panel_frame() -> Frame {
        Frame::new()
            .fill(PANEL)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12))
    }
}

impl eframe::App for MdEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.need_refresh && self.last_edit.elapsed() > Duration::from_millis(120) {
            self.refresh_preview();
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.file_title()));

        // Top bar
        egui::TopBottomPanel::top("menu")
            .frame(
                Frame::new()
                    .fill(PANEL)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(12, 6)),
            )
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    ui.menu_button(RichText::new("文件").color(TEXT), |ui| {
                        if ui.button("打开…    Ctrl+O").clicked() {
                            self.open_file();
                            ui.close_menu();
                        }
                        if ui.button("保存      Ctrl+S").clicked() {
                            self.save();
                            ui.close_menu();
                        }
                        if ui.button("另存为…").clicked() {
                            self.save_as();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("退出").clicked() {
                            self.request_quit(ctx);
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(RichText::new("编辑").color(TEXT), |ui| {
                        if ui.button("查找 / 替换    Ctrl+F").clicked() {
                            self.find_open = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(RichText::new("视图").color(TEXT), |ui| {
                        ui.add(
                            egui::Slider::new(&mut self.split, 0.25..=0.75).text("分栏比例"),
                        );
                        if ui
                            .button(if self.dark {
                                "浅色主题"
                            } else {
                                "深色主题"
                            })
                            .clicked()
                        {
                            self.dark = !self.dark;
                            self.apply_theme(ctx);
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    if self.dirty {
                        ui.label(RichText::new("● 未保存").color(WARN).small());
                    } else {
                        ui.label(RichText::new("○ 已保存").color(MUTED).small());
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Enter 确认输入 · Ctrl+Enter 换行")
                                .small()
                                .color(MUTED),
                        );
                    });
                });
            });

        // Status
        egui::TopBottomPanel::bottom("status")
            .frame(
                Frame::new()
                    .fill(PANEL)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(12, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status).color(TEXT).small());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} 字 · {} 行",
                                self.source.chars().count(),
                                self.source.lines().count().max(1)
                            ))
                            .small()
                            .color(MUTED),
                        );
                    });
                });
            });

        if self.find_open {
            egui::Window::new("查找 / 替换")
                .collapsible(false)
                .resizable(false)
                .frame(Self::panel_frame())
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("查找");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.find_query)
                                .desired_width(220.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("替换");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.replace_with)
                                .desired_width(220.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        if ui.button("查找下一处").clicked() {
                            self.find_next();
                        }
                        if ui.button("全部替换").clicked() {
                            self.replace_all();
                        }
                        if ui.button("关闭").clicked() {
                            self.find_open = false;
                        }
                    });
                });
        }

        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG).inner_margin(Margin::same(10)))
            .show(ctx, |ui| {
                let full = ui.available_width();
                let gap = 10.0;
                let left_w = (full - gap) * self.split;
                let right_w = full - gap - left_w;
                let h = ui.available_height();

                ui.horizontal(|ui| {
                    // Editor card
                    ui.allocate_ui(Vec2::new(left_w, h), |ui| {
                        Self::panel_frame().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("源码")
                                        .strong()
                                        .color(ACCENT)
                                        .size(15.0),
                                );
                                ui.label(
                                    RichText::new("Markdown")
                                        .small()
                                        .color(MUTED),
                                );
                            });
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(4.0);
                            egui::ScrollArea::both()
                                .id_salt("editor_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    // Enter = IME confirm only; newline only via Ctrl+Enter
                                    let te = egui::TextEdit::multiline(&mut self.source)
                                        .code_editor()
                                        .frame(false)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(32)
                                        .text_color(TEXT)
                                        .return_key(Some(egui::KeyboardShortcut::new(
                                            egui::Modifiers::CTRL,
                                            egui::Key::Enter,
                                        )));
                                    let response = ui.add(te);
                                    if response.changed() {
                                        self.dirty = true;
                                        self.quit_confirm = false;
                                        self.need_refresh = true;
                                        self.last_edit = Instant::now();
                                    }
                                });
                        });
                    });

                    ui.add_space(gap);

                    // Preview card
                    ui.allocate_ui(Vec2::new(right_w, h), |ui| {
                        Self::panel_frame().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("预览")
                                        .strong()
                                        .color(OK)
                                        .size(15.0),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} blocks · {} reused",
                                        self.preview.blocks.len(),
                                        self.last_reuse
                                    ))
                                    .small()
                                    .color(MUTED),
                                );
                            });
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(4.0);
                            egui::ScrollArea::vertical()
                                .id_salt("preview_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 4.0;
                                    for line in self.preview.all_lines() {
                                        let rich = match line.kind {
                                            LineKind::Heading(1) => RichText::new(&line.text)
                                                .size(26.0)
                                                .strong()
                                                .color(Color32::from_rgb(210, 175, 255)),
                                            LineKind::Heading(2) => RichText::new(&line.text)
                                                .size(22.0)
                                                .strong()
                                                .color(Color32::from_rgb(150, 190, 255)),
                                            LineKind::Heading(_) => RichText::new(&line.text)
                                                .size(18.0)
                                                .strong()
                                                .color(Color32::from_rgb(130, 220, 220)),
                                            LineKind::Code => RichText::new(&line.text)
                                                .monospace()
                                                .size(13.5)
                                                .color(Color32::from_rgb(170, 230, 180)),
                                            LineKind::Quote => RichText::new(format!(
                                                "│ {}",
                                                line.text
                                            ))
                                            .italics()
                                            .color(Color32::from_rgb(150, 210, 160)),
                                            LineKind::Table => RichText::new(&line.text)
                                                .monospace()
                                                .color(Color32::from_rgb(160, 200, 255)),
                                            LineKind::Meta => RichText::new(&line.text)
                                                .small()
                                                .color(MUTED),
                                            LineKind::Normal => {
                                                RichText::new(&line.text).color(TEXT).size(14.5)
                                            }
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
