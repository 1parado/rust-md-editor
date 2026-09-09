//! Desktop UI — light glass / liquid style (ChatGPT-inspired)
use crate::preview::{LineKind, PreviewCache};
use eframe::egui;
use egui::{Color32, Frame, Margin, RichText, Rounding, Shadow, Stroke, Vec2};
use std::path::PathBuf;
use std::time::{Duration, Instant};

// —— Light glass palette ——
fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

const BG: Color32 = Color32::from_rgb(245, 246, 250); // soft gray-blue canvas
const GLASS: Color32 = Color32::from_rgb(255, 255, 255);
const GLASS_SOFT: Color32 = Color32::from_rgb(252, 252, 254);
const BORDER: Color32 = Color32::from_rgb(226, 230, 239);
const BORDER_SOFT: Color32 = Color32::from_rgb(236, 239, 245);
const ACCENT: Color32 = Color32::from_rgb(16, 163, 127); // ChatGPT-ish teal
const ACCENT_SOFT: Color32 = Color32::from_rgb(220, 245, 236);
const TEXT: Color32 = Color32::from_rgb(32, 35, 42);
const MUTED: Color32 = Color32::from_rgb(110, 118, 135);
const CODE_BG: Color32 = Color32::from_rgb(246, 248, 252);
const WARN: Color32 = Color32::from_rgb(200, 140, 20);

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
    quit_confirm: bool,
}

impl MdEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        Self::apply_light_glass(cc.egui_ctx.clone());

        let (source, path) = if let Some(p) = path {
            (std::fs::read_to_string(&p).unwrap_or_default(), Some(p))
        } else {
            (
                String::from(
                    "# 你好，rust-md-editor\n\n\
浅色 **玻璃** 风格桌面 Markdown 编辑器。\n\n\
- Enter：确认输入法（不换行）\n\
- Ctrl+Enter：换行\n\
- Ctrl+S 保存 · Ctrl+O 打开 · Ctrl+F 查找\n\n\
```rust\n\
fn main() {\n\
    println!(\"你好，世界\");\n\
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
            status: "就绪 · Enter 确认输入 · Ctrl+Enter 换行".into(),
            find_open: false,
            find_query: String::new(),
            replace_with: String::new(),
            split: 0.5,
            quit_confirm: false,
        };
        app.refresh_preview();
        app
    }

    fn apply_light_glass(ctx: egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.window_fill = GLASS;
        style.visuals.panel_fill = BG;
        style.visuals.extreme_bg_color = CODE_BG;
        style.visuals.faint_bg_color = GLASS_SOFT;
        style.visuals.code_bg_color = CODE_BG;
        style.visuals.override_text_color = Some(TEXT);
        style.visuals.widgets.noninteractive.bg_fill = GLASS_SOFT;
        style.visuals.widgets.inactive.bg_fill = GLASS;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_SOFT);
        style.visuals.widgets.hovered.bg_fill = ACCENT_SOFT;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
        style.visuals.widgets.active.bg_fill = ACCENT_SOFT;
        style.visuals.selection.bg_fill = rgba(16, 163, 127, 40);
        style.visuals.window_rounding = Rounding::same(16.0);
        style.visuals.window_shadow = Shadow {
            offset: Vec2::new(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: rgba(20, 30, 50, 28),
        };
        style.visuals.menu_rounding = Rounding::same(12.0);
        style.visuals.button_frame = true;
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.spacing.window_margin = Margin::same(14.0);
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
            self.status = format!("增量预览 · 复用 {r} · 重绘 {n} · Ctrl+Enter 换行");
        }
    }

    fn save(&mut self) {
        if let Some(ref path) = self.path {
            if let Err(e) = std::fs::write(path, &self.source) {
                self.status = format!("保存失败: {e}");
                return;
            }
            self.dirty = false;
            self.quit_confirm = false;
            self.status = format!("已保存 · {}", path.display());
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
                self.status = format!("保存失败: {e}");
                return;
            }
            self.path = Some(path.clone());
            self.dirty = false;
            self.quit_confirm = false;
            self.status = format!("已保存 · {}", path.display());
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
                Err(e) => self.status = format!("打开失败: {e}"),
            }
        }
    }

    fn request_quit(&mut self, ctx: &egui::Context) {
        if !self.dirty || self.quit_confirm {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            self.quit_confirm = true;
            self.status = "未保存 — 再退出将丢弃，或 Ctrl+S 保存".into();
        }
    }

    fn find_next(&mut self) {
        let q = self.find_query.to_lowercase();
        if q.is_empty() {
            self.status = "查找为空".into();
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

    /// Frosted-glass card
    fn glass_card() -> Frame {
        Frame::none()
            .fill(GLASS)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(16.0))
            .shadow(Shadow {
                offset: Vec2::new(0.0, 4.0),
                blur: 18.0,
                spread: 0.0,
                color: rgba(30, 40, 60, 22),
            })
            .inner_margin(Margin::same(14.0))
    }

    fn top_bar_frame() -> Frame {
        Frame::none()
            .fill(rgba(255, 255, 255, 242))
            .stroke(Stroke::new(1.0, BORDER_SOFT))
            .inner_margin(Margin::symmetric(16.0, 8.0))
    }
}

impl eframe::App for MdEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.need_refresh && self.last_edit.elapsed() > Duration::from_millis(120) {
            self.refresh_preview();
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.file_title()));

        // —— Top bar ——
        egui::TopBottomPanel::top("menu")
            .frame(Self::top_bar_frame())
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    ui.label(
                        RichText::new("✦")
                            .color(ACCENT)
                            .size(16.0)
                            .strong(),
                    );
                    ui.label(RichText::new("rust-md-editor").strong().color(TEXT).size(14.0));
                    ui.add_space(8.0);

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
                        ui.add(egui::Slider::new(&mut self.split, 0.28..=0.72).text("分栏"));
                    });

                    ui.separator();
                    if self.dirty {
                        ui.label(RichText::new("● 未保存").color(WARN).small());
                    } else {
                        ui.label(RichText::new("○ 已同步").color(MUTED).small());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Enter 上屏 · Ctrl+Enter 换行")
                                .small()
                                .color(MUTED),
                        );
                    });
                });
            });

        // —— Status ——
        egui::TopBottomPanel::bottom("status")
            .frame(Self::top_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status).color(MUTED).small());
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
                .frame(Self::glass_card())
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("查找").color(MUTED));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.find_query).desired_width(240.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("替换").color(MUTED));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.replace_with).desired_width(240.0),
                        );
                    });
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Button::new(RichText::new("查找").color(TEXT)))
                            .clicked()
                        {
                            self.find_next();
                        }
                        if ui
                            .add(egui::Button::new(
                                RichText::new("全部替换").color(Color32::WHITE),
                            ).fill(ACCENT))
                            .clicked()
                        {
                            self.replace_all();
                        }
                        if ui.button("关闭").clicked() {
                            self.find_open = false;
                        }
                    });
                });
        }

        // —— Main ——
        egui::CentralPanel::default()
            .frame(Frame::none().fill(BG).inner_margin(Margin::same(14.0)))
            .show(ctx, |ui| {
                let full = ui.available_width();
                let gap = 14.0;
                let left_w = (full - gap) * self.split;
                let right_w = full - gap - left_w;
                let h = ui.available_height();

                ui.horizontal(|ui| {
                    // Source card
                    ui.allocate_ui(Vec2::new(left_w, h), |ui| {
                        Self::glass_card().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("源码")
                                        .strong()
                                        .color(TEXT)
                                        .size(14.0),
                                );
                                ui.label(RichText::new("Markdown").small().color(MUTED));
                            });
                            ui.add_space(8.0);
                            Frame::none()
                                .fill(CODE_BG)
                                .stroke(Stroke::new(1.0, BORDER_SOFT))
                                .rounding(Rounding::same(12.0))
                                .inner_margin(Margin::same(10.0))
                                .show(ui, |ui| {
                                    egui::ScrollArea::both()
                                        .id_salt("editor_scroll")
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            let te = egui::TextEdit::multiline(&mut self.source)
                                                .code_editor()
                                                .frame(false)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(28)
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
                    });

                    ui.add_space(gap);

                    // Preview card
                    ui.allocate_ui(Vec2::new(right_w, h), |ui| {
                        Self::glass_card().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("预览").strong().color(TEXT).size(14.0),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} 块 · 复用 {}",
                                        self.preview.blocks.len(),
                                        self.last_reuse
                                    ))
                                    .small()
                                    .color(MUTED),
                                );
                            });
                            ui.add_space(8.0);
                            egui::ScrollArea::vertical()
                                .id_salt("preview_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 6.0;
                                    for line in self.preview.all_lines() {
                                        let rich = match line.kind {
                                            LineKind::Heading(1) => RichText::new(&line.text)
                                                .size(26.0)
                                                .strong()
                                                .color(TEXT),
                                            LineKind::Heading(2) => RichText::new(&line.text)
                                                .size(21.0)
                                                .strong()
                                                .color(TEXT),
                                            LineKind::Heading(_) => RichText::new(&line.text)
                                                .size(17.0)
                                                .strong()
                                                .color(TEXT),
                                            LineKind::Code => RichText::new(&line.text)
                                                .monospace()
                                                .size(13.0)
                                                .color(Color32::from_rgb(40, 90, 70)),
                                            LineKind::Quote => {
                                                RichText::new(format!("│ {}", line.text))
                                                    .italics()
                                                    .color(MUTED)
                                            }
                                            LineKind::Table => RichText::new(&line.text)
                                                .monospace()
                                                .color(Color32::from_rgb(50, 90, 140)),
                                            LineKind::Meta => {
                                                RichText::new(&line.text).small().color(MUTED)
                                            }
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
