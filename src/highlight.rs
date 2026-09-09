//! Syntax highlighting (syntect + light source coloring)
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SynStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

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

    pub fn highlight_code(&self, code: &str, lang: &str) -> Vec<Line<'static>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);
        let mut lines = Vec::new();
        for line in LinesWithEndings::from(code) {
            let ranges = h.highlight_line(line, &self.syntax_set).unwrap_or_default();
            let spans: Vec<Span> = ranges
                .into_iter()
                .map(|(style, text)| Span::styled(text.to_string(), syn_to_ratatui(style)))
                .collect();
            lines.push(Line::from(spans));
        }
        if lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines
    }

    pub fn highlight_source_line(&self, line: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            let color = match level {
                1 => Color::Magenta,
                2 => Color::Blue,
                3 => Color::Cyan,
                _ => Color::Gray,
            };
            spans.push(Span::styled(
                line.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        } else if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            spans.push(Span::styled(line.to_string(), Style::default().fg(Color::Yellow)));
        } else if trimmed.starts_with('>') {
            spans.push(Span::styled(line.to_string(), Style::default().fg(Color::Green)));
        } else if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
        {
            spans.push(Span::styled(line.to_string(), Style::default().fg(Color::Cyan)));
        } else {
            spans.extend(highlight_inline(line));
        }
        spans
    }
}

fn syn_to_ratatui(s: SynStyle) -> Style {
    let fg = Color::Rgb(s.foreground.r, s.foreground.g, s.foreground.b);
    let mut style = Style::default().fg(fg);
    if s.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
        style = style.add_modifier(Modifier::BOLD);
    }
    if s.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
        style = style.add_modifier(Modifier::ITALIC);
    }
    style
}

fn highlight_inline(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut buf = String::new();
    while i < chars.len() {
        if chars[i] == '`' {
            if !buf.is_empty() {
                spans.push(Span::raw(std::mem::take(&mut buf)));
            }
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '`' {
                i += 1;
            }
            let code: String = chars[start..i].iter().collect();
            spans.push(Span::styled(
                format!("`{}`", code),
                Style::default().fg(Color::Yellow),
            ));
            if i < chars.len() {
                i += 1;
            }
        } else if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if !buf.is_empty() {
                spans.push(Span::raw(std::mem::take(&mut buf)));
            }
            i += 2;
            let start = i;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            spans.push(Span::styled(
                format!("**{}**", text),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ));
            if i + 1 < chars.len() {
                i += 2;
            }
        } else {
            buf.push(chars[i]);
            i += 1;
        }
    }
    if !buf.is_empty() {
        spans.push(Span::raw(buf));
    }
    if spans.is_empty() {
        spans.push(Span::raw(line.to_string()));
    }
    spans
}
