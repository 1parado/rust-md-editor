//! Incremental markdown blocks (streamdown-style)
use crate::highlight::Highlighter;
use pulldown_cmark::{CodeBlockKind, Event as MdEvent, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug)]
pub struct MdBlock {
    pub source: String,
    pub id: String,
    pub rendered: Vec<Line<'static>>,
    pub incomplete: bool,
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

fn pad_cell(s: &str, width: usize) -> String {
    let w = s.width();
    if w >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - w))
    }
}

fn flush_table(rows: &[Vec<String>], header_rows: usize, lines: &mut Vec<Line<'static>>) {
    if rows.is_empty() {
        return;
    }
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return;
    }
    let mut widths = vec![3usize; cols];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.width().max(1));
        }
    }
    for (ri, row) in rows.iter().enumerate() {
        let mut cells = Vec::new();
        for i in 0..cols {
            let text = row.get(i).map(|s| s.as_str()).unwrap_or("");
            cells.push(pad_cell(text, widths[i]));
        }
        let line = format!("│ {} │", cells.join(" │ "));
        if ri < header_rows {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
            let sep: String = widths
                .iter()
                .map(|w| "─".repeat(*w))
                .collect::<Vec<_>>()
                .join("─┼─");
            lines.push(Line::from(Span::styled(
                format!("├─{}─┤", sep),
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(line));
        }
    }
    lines.push(Line::from(""));
}

fn render_block(src: &str, highlighter: &Highlighter) -> Vec<Line<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(src, options);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut list_depth = 0i32;
    let mut heading_level = 0u8;

    // table state
    let mut in_table = false;
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut table_header_rows = 0usize;
    let mut current_row: Vec<String> = Vec::new();
    let mut cell_buf = String::new();
    let mut in_header = false;

    let flush_line = |spans: &mut Vec<Span<'static>>, lines: &mut Vec<Line<'static>>| {
        if !spans.is_empty() {
            lines.push(Line::from(std::mem::take(spans)));
        } else {
            lines.push(Line::from(""));
        }
    };

    for event in parser {
        match event {
            MdEvent::Start(Tag::Heading { level, .. }) => {
                heading_level = level as u8;
            }
            MdEvent::End(TagEnd::Heading(_)) => {
                if !current_spans.is_empty() {
                    let text: String = current_spans.iter().map(|s| s.content.as_ref()).collect();
                    let color = match heading_level {
                        1 => Color::Magenta,
                        2 => Color::Blue,
                        3 => Color::Cyan,
                        _ => Color::Gray,
                    };
                    lines.push(Line::from(Span::styled(
                        text,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    )));
                    current_spans.clear();
                }
                heading_level = 0;
            }
            MdEvent::Start(Tag::CodeBlock(kind)) => {
                in_code = true;
                code_lang = match kind {
                    CodeBlockKind::Indented => String::new(),
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                };
                code_buf.clear();
            }
            MdEvent::End(TagEnd::CodeBlock) => {
                in_code = false;
                let lang = if code_lang.is_empty() { "txt" } else { &code_lang };
                let hl = highlighter.highlight_code(&code_buf, lang);
                lines.push(Line::from(Span::styled(
                    format!("┌─ {} ", lang),
                    Style::default().fg(Color::DarkGray),
                )));
                for l in hl {
                    lines.push(l);
                }
                lines.push(Line::from(Span::styled(
                    "└─",
                    Style::default().fg(Color::DarkGray),
                )));
                code_buf.clear();
            }
            MdEvent::Start(Tag::Table(_)) => {
                in_table = true;
                table_rows.clear();
                table_header_rows = 0;
            }
            MdEvent::End(TagEnd::Table) => {
                flush_table(&table_rows, table_header_rows, &mut lines);
                in_table = false;
            }
            MdEvent::Start(Tag::TableHead) => {
                in_header = true;
            }
            MdEvent::End(TagEnd::TableHead) => {
                in_header = false;
            }
            MdEvent::Start(Tag::TableRow) => {
                current_row.clear();
            }
            MdEvent::End(TagEnd::TableRow) => {
                if in_header {
                    table_header_rows += 1;
                }
                table_rows.push(std::mem::take(&mut current_row));
            }
            MdEvent::Start(Tag::TableCell) => {
                cell_buf.clear();
            }
            MdEvent::End(TagEnd::TableCell) => {
                current_row.push(std::mem::take(&mut cell_buf));
            }
            MdEvent::Text(text) => {
                if in_code {
                    code_buf.push_str(&text);
                } else if in_table {
                    cell_buf.push_str(&text);
                } else {
                    current_spans.push(Span::raw(text.to_string()));
                }
            }
            MdEvent::Code(text) => {
                if in_code {
                    code_buf.push_str(&text);
                } else if in_table {
                    cell_buf.push_str(&text);
                } else {
                    current_spans.push(Span::styled(
                        text.to_string(),
                        Style::default().fg(Color::Yellow),
                    ));
                }
            }
            MdEvent::SoftBreak | MdEvent::HardBreak => {
                if !in_table {
                    flush_line(&mut current_spans, &mut lines);
                }
            }
            MdEvent::Start(Tag::List(_)) => {
                list_depth += 1;
            }
            MdEvent::End(TagEnd::List(_)) => {
                list_depth = (list_depth - 1).max(0);
            }
            MdEvent::Start(Tag::Item) => {
                let indent = "  ".repeat(list_depth.saturating_sub(1) as usize);
                current_spans.push(Span::styled(
                    format!("{}• ", indent),
                    Style::default().fg(Color::Cyan),
                ));
            }
            MdEvent::End(TagEnd::Item) => {
                flush_line(&mut current_spans, &mut lines);
            }
            MdEvent::Start(Tag::BlockQuote(_)) => {
                current_spans.push(Span::styled("│ ", Style::default().fg(Color::Green)));
            }
            MdEvent::End(TagEnd::BlockQuote(_)) => {
                flush_line(&mut current_spans, &mut lines);
            }
            MdEvent::Rule => {
                lines.push(Line::from(Span::styled(
                    "─".repeat(40),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            MdEvent::Start(Tag::Paragraph) => {}
            MdEvent::End(TagEnd::Paragraph) => {
                flush_line(&mut current_spans, &mut lines);
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }
    if !current_spans.is_empty() {
        flush_line(&mut current_spans, &mut lines);
    }
    if lines.is_empty() {
        lines.push(Line::from(""));
    }
    if is_incomplete(src) {
        lines.push(Line::from(Span::styled(
            " ⋯ (streaming…)",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
    }
    lines
}

pub struct PreviewCache {
    pub blocks: Vec<MdBlock>,
    prev_map: HashMap<String, usize>,
}

impl PreviewCache {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            prev_map: HashMap::new(),
        }
    }

    pub fn update(&mut self, md: &str, highlighter: &Highlighter) -> (usize, usize) {
        let sources = split_into_blocks(md);
        let mut new_blocks = Vec::with_capacity(sources.len());
        let mut reused = 0usize;
        let mut rendered = 0usize;

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
            let rendered_lines = render_block(&src, highlighter);
            new_blocks.push(MdBlock {
                source: src.clone(),
                id: id.clone(),
                rendered: rendered_lines,
                incomplete: is_incomplete(&src),
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

    pub fn all_lines(&self) -> Vec<Line<'static>> {
        let mut out = Vec::new();
        for b in &self.blocks {
            out.extend(b.rendered.clone());
        }
        out
    }
}
