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
    pub highlighter: Highlighter,
    pub preview: PreviewCache,
    pub preview_scroll: usize,
    pub scroll_sync: bool,
    /// Focus: 0 = editor, 1 = preview
    pub focus: u8,
    pub last_reuse: usize,
    pub last_render: usize,
    pub preview_lines: Vec<Line<'static>>,
    pub need_preview_refresh: bool,
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
                     这是一个**轻量级** Rust Markdown 编辑器，支持：\n\n\
                     - **增量解析**：只重渲染变化的块\n\
                     - **语法高亮**（syntect）\n\
                     - **滚动同步** + 鼠标支持\n\n\
                     ```rust\n\
                     fn main() {\n\
                         println!(\"Hello, world!\");\n\
                     }\n\
                     ```\n\n\
                     > 未闭合的代码块会显示 *streaming…*\n\n\
                     试试继续输入一个未闭合的 fence：\n\n\
                     ```python\n\
                     def hello():\n\
                         print(\"still streaming\")\n",
                ),
                None,
            )
        };

        let mut app = Self {
            buffer: Buffer::new(&content),
            path,
            dirty: false,
            status: "Ready · Ctrl+S save · Ctrl+Q quit · ? help · Tab focus".into(),
            show_help: false,
            highlighter: Highlighter::new(),
            preview: PreviewCache::new(),
            preview_scroll: 0,
            scroll_sync: true,
            focus: 0,
            last_reuse: 0,
            last_render: 0,
            preview_lines: Vec::new(),
            need_preview_refresh: true,
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
        self.status = format!(
            "blocks: {} reused / {} rendered · sync:{} · focus:{}",
            reused,
            rendered,
            if self.scroll_sync { "ON" } else { "OFF" },
            if self.focus == 0 { "edit" } else { "preview" }
        );
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(ref path) = self.path {
            fs::write(path, self.buffer.content())?;
            self.dirty = false;
            self.status = format!("Saved: {}", path.display());
        } else {
            let default = PathBuf::from("untitled.md");
            fs::write(&default, self.buffer.content())?;
            self.path = Some(default.clone());
            self.dirty = false;
            self.status = format!("Saved: {}", default.display());
        }
        Ok(())
    }

    pub fn on_edit(&mut self) {
        self.dirty = true;
        self.need_preview_refresh = true;
    }
}
