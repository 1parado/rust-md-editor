//! Incremental markdown blocks → plain preview lines (streamdown-style)
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[derive(Clone, Debug)]
pub struct MdBlock {
    pub source: String,
    pub id: String,
    pub lines: Vec<PreviewLine>,
    pub incomplete: bool,
}

#[derive(Clone, Debug)]
pub struct PreviewLine {
    pub text: String,
    pub kind: LineKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Normal,
    Heading(u8),
    Code,
    Quote,
    Table,
    Meta,
}

fn hash_str(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex::encode(h.finalize())[..12].to_string()
}

fn split_into_blocks(md: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut in_fence = false;
    let mut fence_char = '`';
    let mut fence_len = 0usize;

    for line in md.lines() {
        let trimmed = line.trim_start();
        if !in_fence {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                fence_char = trimmed.chars().next().unwrap();
                fence_len = trimmed.chars().take_while(|c| *c == fence_char).count();
                in_fence = true;
                current.push_str(line);
                current.push('\n');
                continue;
            }
            if trimmed.is_empty() {
                if !current.trim().is_empty() {
                    blocks.push(std::mem::take(&mut current));
                }
                current.clear();
                continue;
            }
            current.push_str(line);
            current.push('\n');
        } else {
            current.push_str(line);
            current.push('\n');
            if trimmed.starts_with(fence_char) {
                let close_len = trimmed.chars().take_while(|c| *c == fence_char).count();
                if close_len >= fence_len && trimmed[close_len..].trim().is_empty() {
                    in_fence = false;
                    blocks.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    if blocks.is_empty() {
        blocks.push(String::new());
    }
    blocks
}

fn is_incomplete(src: &str) -> bool {
    let mut fence = 0i32;
    for line in src.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            let ch = t.chars().next().unwrap();
            let n = t.chars().take_while(|c| *c == ch).count();
            if fence == 0 {
                fence = n as i32;
            } else if n as i32 >= fence {
                fence = 0;
            }
        }
    }
    fence != 0
}

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_code_plain(&self, code: &str, lang: &str) -> Vec<String> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);
        let mut out = Vec::new();
        for line in LinesWithEndings::from(code) {
            let ranges = h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            let text: String = ranges.into_iter().map(|(_, t)| t).collect();
            out.push(text.trim_end_matches('\n').to_string());
        }
        if out.is_empty() {
            out.push(String::new());
        }
        out
    }
}

fn render_block(src: &str, hl: &Highlighter) -> Vec<PreviewLine> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(src, options);
    let mut lines: Vec<PreviewLine> = Vec::new();
    let mut buf = String::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut heading: u8 = 0;
    let mut in_quote = false;
    let mut list_depth = 0i32;

    let flush = |buf: &mut String, lines: &mut Vec<PreviewLine>, kind: LineKind| {
        lines.push(PreviewLine {
            text: std::mem::take(buf),
            kind,
        });
    };

    for ev in parser {
        match ev {
            Event::Start(Tag::Heading { level, .. }) => {
                heading = level as u8;
            }
            Event::End(TagEnd::Heading(_)) => {
                flush(&mut buf, &mut lines, LineKind::Heading(heading));
                heading = 0;
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_lang = match kind {
                    CodeBlockKind::Indented => String::new(),
                    CodeBlockKind::Fenced(l) => l.to_string(),
                };
                code_buf.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code = false;
                let lang = if code_lang.is_empty() {
                    "txt"
                } else {
                    &code_lang
                };
                lines.push(PreviewLine {
                    text: format!("┌─ {lang}"),
                    kind: LineKind::Meta,
                });
                for l in hl.highlight_code_plain(&code_buf, lang) {
                    lines.push(PreviewLine {
                        text: l,
                        kind: LineKind::Code,
                    });
                }
                lines.push(PreviewLine {
                    text: "└─".into(),
                    kind: LineKind::Meta,
                });
            }
            Event::Start(Tag::BlockQuote(_)) => in_quote = true,
            Event::End(TagEnd::BlockQuote(_)) => {
                if !buf.is_empty() {
                    flush(&mut buf, &mut lines, LineKind::Quote);
                }
                in_quote = false;
            }
            Event::Start(Tag::List(_)) => list_depth += 1,
            Event::End(TagEnd::List(_)) => list_depth = (list_depth - 1).max(0),
            Event::Start(Tag::Item) => {
                let ind = "  ".repeat(list_depth.saturating_sub(1) as usize);
                buf.push_str(&format!("{ind}• "));
            }
            Event::End(TagEnd::Item) => {
                flush(&mut buf, &mut lines, LineKind::Normal);
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                let kind = if in_quote {
                    LineKind::Quote
                } else {
                    LineKind::Normal
                };
                flush(&mut buf, &mut lines, kind);
                lines.push(PreviewLine {
                    text: String::new(),
                    kind: LineKind::Normal,
                });
            }
            Event::Text(t) | Event::Code(t) => {
                if in_code {
                    code_buf.push_str(&t);
                } else {
                    buf.push_str(&t);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if !in_code {
                    buf.push(' ');
                }
            }
            Event::Rule => {
                lines.push(PreviewLine {
                    text: "─".repeat(40),
                    kind: LineKind::Meta,
                });
            }
            Event::Start(Tag::Table(_)) => {}
            Event::End(TagEnd::Table) => {
                lines.push(PreviewLine {
                    text: String::new(),
                    kind: LineKind::Normal,
                });
            }
            Event::Start(Tag::TableCell) => buf.push_str("│ "),
            Event::End(TagEnd::TableCell) => buf.push(' '),
            Event::End(TagEnd::TableRow) => {
                buf.push('│');
                flush(&mut buf, &mut lines, LineKind::Table);
            }
            _ => {}
        }
    }
    if !buf.is_empty() {
        flush(&mut buf, &mut lines, LineKind::Normal);
    }
    if is_incomplete(src) {
        lines.push(PreviewLine {
            text: " ⋯ (streaming…)".into(),
            kind: LineKind::Meta,
        });
    }
    if lines.is_empty() {
        lines.push(PreviewLine {
            text: String::new(),
            kind: LineKind::Normal,
        });
    }
    lines
}

pub struct PreviewCache {
    pub blocks: Vec<MdBlock>,
    prev_map: HashMap<String, usize>,
    pub highlighter: Highlighter,
}

impl PreviewCache {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            prev_map: HashMap::new(),
            highlighter: Highlighter::new(),
        }
    }

    pub fn update(&mut self, md: &str) -> (usize, usize) {
        let sources = split_into_blocks(md);
        let mut new_blocks = Vec::with_capacity(sources.len());
        let mut reused = 0;
        let mut rendered = 0;
        for src in sources {
            let id = hash_str(&src);
            if let Some(&idx) = self.prev_map.get(&id) {
                if let Some(old) = self.blocks.get(idx) {
                    if old.id == id {
                        new_blocks.push(old.clone());
                        reused += 1;
                        continue;
                    }
                }
            }
            let lines = render_block(&src, &self.highlighter);
            new_blocks.push(MdBlock {
                incomplete: is_incomplete(&src),
                source: src,
                id,
                lines,
            });
            rendered += 1;
        }
        self.prev_map.clear();
        for (i, b) in new_blocks.iter().enumerate() {
            self.prev_map.insert(b.id.clone(), i);
        }
        self.blocks = new_blocks;
        (reused, rendered)
    }

    pub fn all_lines(&self) -> Vec<&PreviewLine> {
        self.blocks.iter().flat_map(|b| b.lines.iter()).collect()
    }
}
