//! App state
use crate::buffer::Buffer;
use crate::highlight::Highlighter;
use crate::preview::PreviewCache;
use anyhow::Result;
use ratatui::text::Line;
use std::fs;
use std::path::PathBuf;

pub struct App {
    pub buffer: Buffer,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub status: String,
    pub show_help: bool,
    pub quit_confirm: bool,
    pub highlighter: Highlighter,
    pub preview: PreviewCache,
    pub preview_scroll: usize,
    pub scroll_sync: bool,
    pub focus: u8,
    pub last_reuse: usize,
    pub last_render: usize,
    pub preview_lines: Vec<Line<'static>>,
    pub need_preview_refresh: bool,
    /// Search UI
    pub finding: bool,
    pub find_query: String,
    pub find_count: usize,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let (content, path) = if let Some(p) = path {
            let content = fs::read_to_string(&p).unwrap_or_default();
            (content, Some(p))
        } else {
            (
                String::from(
                    "# Hello, Streamdown-style Editor\n\n\
                     这是一个**轻量级** Rust Markdown 编辑器。\n\n\
                     按 **Ctrl+F** 可搜索，例如搜索 `streaming`。\n\n\
                     ```rust\n\
                     fn main() {\n\
                         println!(\"Hello, world!\");\n\
                     }\n\
                     ```\n\n\
                     > 未闭合的代码块会显示 *streaming…*\n",
                ),
                None,
            )
        };

        let mut app = Self {
            buffer: Buffer::new(&content),
            path,
            dirty: false,
            status: "Ready · Ctrl+S save · Ctrl+F find · Ctrl+Q quit · ? help".into(),
            show_help: false,
            quit_confirm: false,
            highlighter: Highlighter::new(),
            preview: PreviewCache::new(),
            preview_scroll: 0,
            scroll_sync: true,
            focus: 0,
            last_reuse: 0,
            last_render: 0,
            preview_lines: Vec::new(),
            need_preview_refresh: true,
            finding: false,
            find_query: String::new(),
            find_count: 0,
        };
        app.refresh_preview();
        Ok(app)
    }

    pub fn refresh_preview(&mut self) {
        let (reused, rendered) = self
            .preview
            .update(&self.buffer.content(), &self.highlighter);
        self.last_reuse = reused;
        self.last_render = rendered;
        self.preview_lines = self.preview.all_lines();
        self.need_preview_refresh = false;
        if !self.quit_confirm && !self.finding {
            self.status = format!(
                "blocks: {} reused / {} rendered · sync:{} · focus:{}",
                reused,
                rendered,
                if self.scroll_sync { "ON" } else { "OFF" },
                if self.focus == 0 { "edit" } else { "preview" }
            );
        }
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(ref path) = self.path {
            fs::write(path, self.buffer.content())?;
            self.dirty = false;
            self.quit_confirm = false;
            self.status = format!("Saved: {}", path.display());
        } else {
            let default = PathBuf::from("untitled.md");
            fs::write(&default, self.buffer.content())?;
            self.path = Some(default.clone());
            self.dirty = false;
            self.quit_confirm = false;
            self.status = format!("Saved: {}", default.display());
        }
        Ok(())
    }

    pub fn on_edit(&mut self) {
        self.dirty = true;
        self.quit_confirm = false;
        self.need_preview_refresh = true;
    }

    pub fn request_quit(&mut self) -> bool {
        if !self.dirty || self.quit_confirm {
            return true;
        }
        self.quit_confirm = true;
        self.status = "Unsaved changes — Ctrl+Q again to quit, Ctrl+S to save".into();
        false
    }

    pub fn sync_preview_from_editor(&mut self) {
        if !self.scroll_sync {
            return;
        }
        let elen = self.buffer.line_count().max(1);
        let plen = self.preview_lines.len().max(1);
        self.preview_scroll = (self.buffer.scroll * plen) / elen;
    }

    pub fn sync_editor_from_preview(&mut self) {
        if !self.scroll_sync {
            return;
        }
        let elen = self.buffer.line_count().max(1);
        let plen = self.preview_lines.len().max(1);
        self.buffer.scroll = (self.preview_scroll * elen) / plen;
    }

    pub fn start_find(&mut self) {
        self.finding = true;
        self.focus = 0;
        self.status = format!("Find: {}_  (Enter/n next · N prev · Esc)", self.find_query);
    }

    pub fn stop_find(&mut self) {
        self.finding = false;
        self.status = "Find closed".into();
    }

    /// Case-insensitive substring search. `forward` chooses direction.
    pub fn find_next(&mut self, forward: bool) {
        let q = self.find_query.to_lowercase();
        if q.is_empty() {
            self.status = "Find: (empty query)".into();
            return;
        }

        let rows = self.buffer.lines.len();
        if rows == 0 {
            return;
        }

        let start_row = self.buffer.cursor_row;
        let start_col = self.buffer.cursor_col;

        let mut checked = 0usize;
        let mut row = start_row;
        let mut col = if forward {
            // start after current position
            let line = &self.buffer.lines[row];
            let mut c = start_col;
            if c < line.len() {
                c += 1;
                while c < line.len() && !line.is_char_boundary(c) {
                    c += 1;
                }
            }
            c
        } else {
            start_col
        };

        while checked <= rows {
            let line = &self.buffer.lines[row];
            let lower = line.to_lowercase();

            if forward {
                if col <= lower.len() {
                    if let Some(rel) = lower[col.min(lower.len())..].find(&q) {
                        let abs = col + rel;
                        self.buffer.goto(row, abs);
                        self.find_count = self.count_matches(&q);
                        self.status = format!(
                            "Find: '{}' · hit at {}:{} · {} total · n/N",
                            self.find_query,
                            row + 1,
                            abs + 1,
                            self.find_count
                        );
                        return;
                    }
                }
                checked += 1;
                row = (row + 1) % rows;
                col = 0;
            } else {
                // search backward in this line before col
                let end = col.min(lower.len());
                if let Some(rel) = lower[..end].rfind(&q) {
                    self.buffer.goto(row, rel);
                    self.find_count = self.count_matches(&q);
                    self.status = format!(
                        "Find: '{}' · hit at {}:{} · {} total · n/N",
                        self.find_query,
                        row + 1,
                        rel + 1,
                        self.find_count
                    );
                    return;
                }
                checked += 1;
                if row == 0 {
                    row = rows - 1;
                } else {
                    row -= 1;
                }
                col = self.buffer.lines[row].len();
            }
        }

        self.status = format!("Find: '{}' · no match", self.find_query);
    }

    fn count_matches(&self, q_lower: &str) -> usize {
        self.buffer
            .lines
            .iter()
            .map(|l| l.to_lowercase().matches(q_lower).count())
            .sum()
    }
}
