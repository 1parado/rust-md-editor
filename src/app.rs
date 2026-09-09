//! Desktop UI — light/dark glass style (ChatGPT-inspired)
use crate::preview::{CodeSpan, LineKind, PreviewCache, PreviewLine};
use eframe::egui;
use egui::{
    text::LayoutJob, Color32, CursorIcon, FontId, Frame, Margin, RichText, Rounding, Sense, Shadow,
    Stroke, TextFormat, Vec2,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const EDITOR_ID: &str = "editor_text";

#[derive(Clone, Copy)]
struct Palette {
    bg: Color32,
    glass: Color32,
    glass_soft: Color32,
    border: Color32,
    border_soft: Color32,
    accent: Color32,
    accent_soft: Color32,
    on_accent: Color32,
    text: Color32,
    muted: Color32,
    code_bg: Color32,
    code_card: Color32,
    code_text: Color32,
    code_border: Color32,
    warn: Color32,
    shadow: [u8; 4],
}

const fn c(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Palette {
    const LIGHT: Self = Self {
        bg: c(245, 246, 250),
        glass: c(255, 255, 255),
        glass_soft: c(252, 252, 254),
        border: c(226, 230, 239),
        border_soft: c(236, 239, 245),
        accent: c(16, 163, 127),
        accent_soft: c(220, 245, 236),
        on_accent: c(255, 255, 255),
        text: c(32, 35, 42),
        muted: c(110, 118, 135),
        code_bg: c(246, 248, 252),
        code_card: c(246, 248, 252),
        code_text: c(31, 98, 73),
        code_border: c(230, 234, 242),
        warn: c(200, 140, 20),
        shadow: [30, 40, 60, 22],
    };

    const DARK: Self = Self {
        bg: c(24, 26, 31),
        glass: c(32, 35, 42),
        glass_soft: c(38, 41, 49),
        border: c(56, 60, 72),
        border_soft: c(46, 50, 60),
        accent: c(63, 185, 149),
        accent_soft: c(38, 66, 56),
        on_accent: c(8, 20, 16),
        text: c(229, 232, 240),
        muted: c(148, 155, 172),
        code_bg: c(20, 22, 27),
        code_card: c(22, 25, 31),
        code_text: c(140, 200, 175),
        code_border: c(44, 48, 58),
        warn: c(226, 178, 92),
        shadow: [0, 0, 0, 90],
    };
}

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
    dark: bool,
    split: f32,
    find_open: bool,
    focus_find: bool,
    find_query: String,
    find_pos: Option<usize>,
    replace_with: String,
    new_confirm: bool,
    drop_confirm: bool,
    quit_confirm: bool,
}

impl MdEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        let (source, path) = if let Some(p) = path {
            (std::fs::read_to_string(&p).unwrap_or_default(), Some(p))
        } else {
            (
                String::from(
                    "# 你好，rust-md-editor\n\n\
                    轻量 **玻璃** 风格桌面 Markdown 编辑器。\n\n\
                    - 拖动中间分隔条调整分栏\n\
                    - 拖入 .md 文件即可打开\n\
                    - Ctrl+N 新建 · Ctrl+S 保存 · Ctrl+O 打开 · Ctrl+F 查找\n\n\
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
            status: "就绪 · Ctrl+S 保存 · Ctrl+F 查找".into(),
            dark: false,
            split: 0.5,
            find_open: false,
            focus_find: false,
            find_query: String::new(),
            find_pos: None,
            replace_with: String::new(),
            new_confirm: false,
            drop_confirm: false,
            quit_confirm: false,
        };
        app.apply_theme(&cc.egui_ctx);
        app.refresh_preview();
        app
    }

    fn palette(&self) -> Palette {
        if self.dark {
            Palette::DARK
        } else {
            Palette::LIGHT
        }
    }

    fn theme_name(&self) -> &'static str {
        if self.dark {
            "base16-ocean.dark"
        } else {
            "base16-ocean.light"
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let p = self.palette();
        let mut style = (*ctx.style()).clone();
        style.visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        style.visuals.panel_fill = p.bg;
        style.visuals.window_fill = p.glass;
        style.visuals.extreme_bg_color = p.code_bg;
        style.visuals.faint_bg_color = p.glass_soft;
        style.visuals.code_bg_color = p.code_bg;
        style.visuals.override_text_color = Some(p.text);
        style.visuals.widgets.noninteractive.bg_fill = p.glass_soft;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.muted);
        style.visuals.widgets.inactive.bg_fill = p.glass;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border_soft);
        style.visuals.widgets.hovered.bg_fill = p.accent_soft;
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.accent);
        style.visuals.widgets.active.bg_fill = p.accent_soft;
        style.visuals.selection.bg_fill =
            Color32::from_rgba_unmultiplied(p.accent.r(), p.accent.g(), p.accent.b(), 50);
        style.visuals.window_rounding = Rounding::same(16.0);
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
        let theme = self.theme_name();
        let (r, n) = self.preview.update(&self.source, theme);
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
                    self.find_pos = None;
                    self.new_confirm = false;
                    self.drop_confirm = false;
                    self.status = "已打开".into();
                }
                Err(e) => self.status = format!("打开失败: {e}"),
            }
        }
    }

    fn new_file(&mut self) {
        if self.dirty && !self.new_confirm {
            self.new_confirm = true;
            self.status = "有未保存更改 — 再次新建将丢弃，或 Ctrl+S 保存".into();
            return;
        }
        self.new_confirm = false;
        self.source = String::new();
        self.path = None;
        self.dirty = false;
        self.quit_confirm = false;
        self.need_refresh = true;
        self.find_pos = None;
        self.drop_confirm = false;
        self.status = "新建文档".into();
    }

    fn open_dropped(&mut self, file: &PathBuf) {
        match std::fs::read_to_string(file) {
            Ok(s) => {
                self.source = s;
                self.path = Some(file.clone());
                self.dirty = false;
                self.quit_confirm = false;
                self.need_refresh = true;
                self.find_pos = None;
                self.drop_confirm = false;
                self.status = format!("已打开 · {}", file.display());
            }
            Err(e) => self.status = format!("打开失败: {e}"),
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

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.dark = !self.dark;
        self.apply_theme(ctx);
        self.need_refresh = true;
        self.status = if self.dark {
            "已切换到暗色主题"
        } else {
            "已切换到亮色主题"
        }
        .into();
    }

    fn find_matches(&self) -> Vec<usize> {
        let q = self.find_query.to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let hay = self.source.to_lowercase();
        let mut out = Vec::new();
        let mut off = 0usize;
        while let Some(p) = hay[off..].find(&q) {
            out.push(off + p);
            off += p + q.len();
        }
        out
    }

    /// Move the editor cursor (and its scroll) to a char index.
    fn jump_to(&self, ctx: &egui::Context, char_idx: usize) {
        let id = egui::Id::new(EDITOR_ID);
        if let Some(mut st) = egui::TextEdit::load_state(ctx, id) {
            st.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                egui::text::CCursor::new(char_idx),
            )));
            egui::TextEdit::store_state(ctx, id, st);
        }
    }

    fn cursor_line_col(&self, ctx: &egui::Context) -> (usize, usize) {
        let id = egui::Id::new(EDITOR_ID);
        let Some(st) = egui::TextEdit::load_state(ctx, id) else {
            return (1, 1);
        };
        let Some(r) = st.cursor.char_range() else {
            return (1, 1);
        };
        let idx = r.primary.index;
        let mut line = 1usize;
        let mut col = 1usize;
        for (i, ch) in self.source.chars().enumerate() {
            if i >= idx {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    /// Cycle to the previous / next match and jump the cursor there.
    fn find_step(&mut self, ctx: &egui::Context, forward: bool) {
        let matches = self.find_matches();
        if matches.is_empty() {
            self.find_pos = None;
            self.status = if self.find_query.is_empty() {
                "查找为空".into()
            } else {
                "未找到".into()
            };
            return;
        }
        let pos = match self.find_pos {
            None => {
                if forward {
                    0
                } else {
                    matches.len() - 1
                }
            }
            Some(cur) => match matches.iter().position(|&m| m == cur) {
                Some(i) => {
                    if forward {
                        (i + 1) % matches.len()
                    } else {
                        (i + matches.len() - 1) % matches.len()
                    }
                }
                None => matches.iter().position(|&m| m > cur).unwrap_or(if forward {
                    0
                } else {
                    matches.len() - 1
                }),
            },
        };
        let byte_idx = matches[pos];
        self.find_pos = Some(byte_idx);
        let char_idx = self.source[..byte_idx].chars().count();
        let line = self.source[..byte_idx]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;
        self.status = format!(
            "找到 · 第 {line} 行 · 第 {} / {} 处",
            pos + 1,
            matches.len()
        );
        self.jump_to(ctx, char_idx);
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
        self.find_pos = None;
        self.last_edit = Instant::now();
        self.status = format!("已替换 {n} 处");
    }

    /// Frosted-glass card
    fn glass_card(p: Palette) -> Frame {
        Frame::none()
            .fill(p.glass)
            .stroke(Stroke::new(1.0, p.border))
            .rounding(Rounding::same(16.0))
            .shadow(Shadow {
                offset: Vec2::new(0.0, 4.0),
                blur: 18.0,
                spread: 0.0,
                color: Color32::from_rgba_unmultiplied(
                    p.shadow[0],
                    p.shadow[1],
                    p.shadow[2],
                    p.shadow[3],
                ),
            })
            .inner_margin(Margin::same(14.0))
    }

    fn top_bar_frame(p: Palette) -> Frame {
        Frame::none()
            .fill(Color32::from_rgba_unmultiplied(
                p.glass.r(),
                p.glass.g(),
                p.glass.b(),
                242,
            ))
            .stroke(Stroke::new(1.0, p.border_soft))
            .inner_margin(Margin::symmetric(16.0, 8.0))
    }
}

fn tool_btn(ui: &mut egui::Ui, p: Palette, label: &str) -> bool {
    ui.add(egui::Button::new(
        RichText::new(label).size(13.0).color(p.text),
    ))
    .clicked()
}

/// Render a line of normal text, styling `inline code` spans distinctly.
fn show_inline(ui: &mut egui::Ui, text: &str, size: f32, color: Color32, code_color: Color32) {
    if !text.contains('`') {
        ui.label(RichText::new(text).size(size).color(color));
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (i, part) in text.split('`').enumerate() {
            if part.is_empty() {
                continue;
            }
            if i % 2 == 1 {
                ui.label(
                    RichText::new(part)
                        .monospace()
                        .size(size - 1.5)
                        .color(code_color),
                );
            } else {
                ui.label(RichText::new(part).size(size).color(color));
            }
        }
    });
}

/// Build a syntax-colored `LayoutJob` for one code line.
fn code_line_job(spans: &[CodeSpan], fallback: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    if spans.is_empty() {
        job.append(
            "",
            0.0,
            TextFormat::simple(FontId::monospace(13.0), fallback),
        );
        return job;
    }
    for s in spans {
        let color = Color32::from_rgb(s.color[0], s.color[1], s.color[2]);
        job.append(
            &s.text.clone(),
            0.0,
            TextFormat::simple(FontId::monospace(13.0), color),
        );
    }
    job
}

/// Rounded dark card with a language chip and colored code lines.
fn code_card(ui: &mut egui::Ui, p: &Palette, lang: &str, lines: &[&PreviewLine]) {
    Frame::none()
        .fill(p.code_card)
        .stroke(Stroke::new(1.0, p.code_border))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                RichText::new(lang.to_uppercase())
                    .size(10.0)
                    .strong()
                    .color(p.accent),
            );
            ui.add_space(3.0);
            ui.spacing_mut().item_spacing.y = 1.0;
            let w = ui.available_width();
            for l in lines {
                let mut job = code_line_job(&l.spans, p.code_text);
                job.wrap.max_width = w;
                ui.label(job);
            }
        });
}

impl eframe::App for MdEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let pal = self.palette();

        if self.need_refresh && self.last_edit.elapsed() > Duration::from_millis(120) {
            self.refresh_preview();
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.file_title()));

        // —— Drag & drop open ——
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            if self.dirty && !self.drop_confirm {
                self.drop_confirm = true;
                self.status = "有未保存更改 — 再次拖放将替换当前文档，或 Ctrl+S 保存".into();
            } else if let Some(path) = dropped.iter().find_map(|f| f.path.clone()) {
                self.open_dropped(&path);
            }
        }

        // —— Top bar ——
        egui::TopBottomPanel::top("menu")
            .frame(Self::top_bar_frame(pal))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    ui.label(RichText::new("✦").color(pal.accent).size(16.0).strong());
                    ui.label(
                        RichText::new("rust-md-editor")
                            .strong()
                            .color(pal.text)
                            .size(14.0),
                    );
                    ui.separator();

                    if tool_btn(ui, pal, "新建") {
                        self.new_file();
                    }
                    if tool_btn(ui, pal, "打开") {
                        self.open_file();
                    }
                    if tool_btn(ui, pal, "保存") {
                        self.save();
                    }
                    if tool_btn(ui, pal, "查找") {
                        self.find_open = true;
                        self.focus_find = true;
                    }
                    ui.separator();

                    ui.menu_button(RichText::new("文件").color(pal.text), |ui| {
                        if ui.button("新建      Ctrl+N").clicked() {
                            self.new_file();
                            ui.close_menu();
                        }
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
                        if ui.button("退出      Ctrl+Q").clicked() {
                            self.request_quit(ctx);
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(RichText::new("编辑").color(pal.text), |ui| {
                        if ui.button("查找 / 替换    Ctrl+F").clicked() {
                            self.find_open = true;
                            self.focus_find = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(RichText::new("视图").color(pal.text), |ui| {
                        ui.add(egui::Slider::new(&mut self.split, 0.28..=0.72).text("分栏"));
                        if ui
                            .button(if self.dark {
                                "切换亮色 ☀️"
                            } else {
                                "切换暗色 🌙"
                            })
                            .clicked()
                        {
                            self.toggle_theme(ctx);
                            ui.close_menu();
                        }
                    });

                    ui.separator();
                    if self.dirty {
                        ui.label(RichText::new("● 未保存").color(pal.warn).small());
                    } else {
                        ui.label(RichText::new("○ 已同步").color(pal.muted).small());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(RichText::new(if self.dark { "☀️" } else { "🌙" }).size(14.0))
                            .clicked()
                        {
                            self.toggle_theme(ctx);
                        }
                        ui.label(
                            RichText::new("Enter 上屏 · Ctrl+Enter 换行")
                                .small()
                                .color(pal.muted),
                        );
                    });
                });
            });

        // —— Status ——
        egui::TopBottomPanel::bottom("status")
            .frame(Self::top_bar_frame(pal))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status).color(pal.muted).small());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (ln, col) = self.cursor_line_col(ctx);
                        ui.label(
                            RichText::new(format!(
                                "Ln {ln}, Col {col} · {} 字 · {} 行",
                                self.source.chars().count(),
                                self.source.lines().count().max(1)
                            ))
                            .small()
                            .color(pal.muted),
                        );
                    });
                });
            });

        if self.find_open {
            egui::Window::new("查找 / 替换")
                .collapsible(false)
                .resizable(false)
                .frame(Self::glass_card(pal))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("查找").color(pal.muted));
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut self.find_query)
                                .desired_width(240.0)
                                .hint_text("输入要查找的文本…"),
                        );
                        if r.changed() {
                            self.find_pos = None;
                        }
                        if self.focus_find {
                            r.request_focus();
                            self.focus_find = false;
                        }
                        if r.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.find_step(ctx, true);
                            r.request_focus();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("替换").color(pal.muted));
                        let r = ui.add(
                            egui::TextEdit::singleline(&mut self.replace_with)
                                .desired_width(240.0)
                                .hint_text("替换为…"),
                        );
                        if r.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.replace_all();
                        }
                    });
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("◀ 上一处").clicked() {
                            self.find_step(ctx, false);
                        }
                        if ui.button("下一处 ▶").clicked() {
                            self.find_step(ctx, true);
                        }
                        if ui
                            .add(
                                egui::Button::new(RichText::new("全部替换").color(pal.on_accent))
                                    .fill(pal.accent),
                            )
                            .clicked()
                        {
                            self.replace_all();
                        }
                        if ui.button("关闭").clicked() {
                            self.find_open = false;
                        }
                    });
                    ui.add_space(2.0);
                    let n = self.find_matches().len();
                    let count_label = if n == 0 {
                        "无匹配".to_string()
                    } else {
                        format!("{n} 处匹配")
                    };
                    ui.label(RichText::new(count_label).small().color(pal.muted));
                });
        }

        // —— Main ——
        egui::CentralPanel::default()
            .frame(Frame::none().fill(pal.bg).inner_margin(Margin::same(14.0)))
            .show(ctx, |ui| {
                let full = ui.available_width();
                let gap = 14.0;
                let left_w = (full - gap) * self.split;
                let right_w = full - gap - left_w;
                let h = ui.available_height();

                ui.horizontal(|ui| {
                    // Source card
                    ui.allocate_ui(Vec2::new(left_w, h), |ui| {
                        Self::glass_card(pal).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("源码").strong().color(pal.text).size(14.0));
                                ui.label(RichText::new("Markdown").small().color(pal.muted));
                            });
                            ui.add_space(8.0);
                            Frame::none()
                                .fill(pal.code_bg)
                                .stroke(Stroke::new(1.0, pal.border_soft))
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
                                                .id_salt(EDITOR_ID)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(28)
                                                .text_color(pal.text)
                                                .return_key(Some(egui::KeyboardShortcut::new(
                                                    egui::Modifiers::CTRL,
                                                    egui::Key::Enter,
                                                )));
                                            let response = ui.add(te);
                                            if response.changed() {
                                                self.dirty = true;
                                                self.quit_confirm = false;
                                                self.need_refresh = true;
                                                self.find_pos = None;
                                                self.last_edit = Instant::now();
                                            }
                                        });
                                });
                        });
                    });

                    // Draggable divider
                    let (handle_rect, handle_resp) =
                        ui.allocate_exact_size(Vec2::new(gap, h), Sense::click_and_drag());
                    let hot = handle_resp.hovered() || handle_resp.dragged();
                    if handle_resp.dragged() {
                        self.split =
                            (self.split + handle_resp.drag_delta().x / (full - gap).max(1.0))
                                .clamp(0.28, 0.72);
                    }
                    ui.painter().rect_filled(
                        egui::Rect::from_center_size(
                            handle_rect.center(),
                            Vec2::new(if hot { 4.0 } else { 3.0 }, if hot { 36.0 } else { 24.0 }),
                        ),
                        Rounding::same(2.0),
                        if hot { pal.accent } else { pal.border },
                    );
                    handle_resp.on_hover_cursor(CursorIcon::ResizeHorizontal);

                    // Preview card
                    ui.allocate_ui(Vec2::new(right_w, h), |ui| {
                        Self::glass_card(pal).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("预览").strong().color(pal.text).size(14.0));
                                ui.label(
                                    RichText::new(format!(
                                        "{} 块 · 复用 {}",
                                        self.preview.blocks.len(),
                                        self.last_reuse
                                    ))
                                    .small()
                                    .color(pal.muted),
                                );
                            });
                            ui.add_space(8.0);
                            egui::ScrollArea::vertical()
                                .id_salt("preview_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.y = 6.0;
                                    if self.source.trim().is_empty() {
                                        ui.label(
                                            RichText::new("（空文档 — 在左侧开始输入）")
                                                .color(pal.muted),
                                        );
                                        return;
                                    }
                                    for block in &self.preview.blocks {
                                        let lines = &block.lines;
                                        let mut i = 0usize;
                                        while i < lines.len() {
                                            let line = &lines[i];
                                            match line.kind {
                                                LineKind::Meta if line.text.starts_with("┌─ ") =>
                                                {
                                                    let lang = line.text["┌─ ".len()..].to_string();
                                                    i += 1;
                                                    let start = i;
                                                    while i < lines.len()
                                                        && lines[i].kind == LineKind::Code
                                                    {
                                                        i += 1;
                                                    }
                                                    let code_lines: Vec<&PreviewLine> =
                                                        lines[start..i].iter().collect();
                                                    if i < lines.len() && lines[i].text == "└─"
                                                    {
                                                        i += 1;
                                                    }
                                                    code_card(ui, &pal, &lang, &code_lines);
                                                }
                                                LineKind::Heading(1) => {
                                                    ui.label(
                                                        RichText::new(&line.text)
                                                            .size(26.0)
                                                            .strong()
                                                            .color(pal.text),
                                                    );
                                                }
                                                LineKind::Heading(2) => {
                                                    ui.label(
                                                        RichText::new(&line.text)
                                                            .size(21.0)
                                                            .strong()
                                                            .color(pal.text),
                                                    );
                                                }
                                                LineKind::Heading(_) => {
                                                    ui.label(
                                                        RichText::new(&line.text)
                                                            .size(17.0)
                                                            .strong()
                                                            .color(pal.text),
                                                    );
                                                }
                                                LineKind::Quote => {
                                                    ui.horizontal_wrapped(|ui| {
                                                        ui.spacing_mut().item_spacing.x = 4.0;
                                                        ui.label(
                                                            RichText::new("▎")
                                                                .strong()
                                                                .color(pal.accent)
                                                                .size(14.0),
                                                        );
                                                        ui.label(
                                                            RichText::new(&line.text)
                                                                .italics()
                                                                .color(pal.muted)
                                                                .size(14.0),
                                                        );
                                                    });
                                                }
                                                LineKind::Table => {
                                                    ui.label(
                                                        RichText::new(&line.text)
                                                            .monospace()
                                                            .size(13.0)
                                                            .color(pal.text),
                                                    );
                                                }
                                                LineKind::Meta => {
                                                    if !line.text.is_empty()
                                                        && line.text.chars().all(|c| c == '─')
                                                    {
                                                        ui.separator();
                                                    } else {
                                                        ui.label(
                                                            RichText::new(&line.text)
                                                                .small()
                                                                .color(pal.muted),
                                                        );
                                                    }
                                                }
                                                LineKind::Code => {
                                                    ui.label(
                                                        RichText::new(&line.text)
                                                            .monospace()
                                                            .size(13.0)
                                                            .color(pal.code_text),
                                                    );
                                                }
                                                LineKind::Normal => {
                                                    show_inline(
                                                        ui,
                                                        &line.text,
                                                        14.5,
                                                        pal.text,
                                                        pal.code_text,
                                                    );
                                                }
                                            }
                                            i += 1;
                                        }
                                        ui.add_space(6.0);
                                    }
                                });
                        });
                    });
                });
            });

        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            self.new_file();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open_file();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
            self.find_open = true;
            self.focus_find = true;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Q)) {
            self.request_quit(ctx);
        }
        if self.find_open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.find_open = false;
        }

        if self.need_refresh {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}
