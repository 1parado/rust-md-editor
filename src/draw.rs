//! Drawing
use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(size);
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);
    let editor_area = main[0];
    let preview_area = main[1];
    let editor_height = editor_area.height.saturating_sub(2) as usize;
    let preview_height = preview_area.height.saturating_sub(2) as usize;
    app.buffer.ensure_visible(editor_height.max(1));

    let mut source_lines: Vec<Line> = Vec::new();
    let start = app.buffer.scroll;
    let end = (start + editor_height).min(app.buffer.lines.len());
    for i in start..end {
        let line = &app.buffer.lines[i];
        let mut spans = vec![Span::styled(
            format!("{:>4} ", i + 1),
            Style::default().fg(Color::DarkGray),
        )];
        let content_spans = app.highlighter.highlight_source_line(line);
        if i == app.buffer.cursor_row && app.focus == 0 {
            let mut col = 0usize;
            let mut before = Vec::new();
            let mut after = Vec::new();
            let mut found = false;
            for sp in content_spans {
                let len = sp.content.len();
                if !found && col + len >= app.buffer.cursor_col {
                    let rel = app.buffer.cursor_col - col;
                    let (b, a) = sp.content.split_at(rel.min(sp.content.len()));
                    if !b.is_empty() {
                        before.push(Span::styled(b.to_string(), sp.style));
                    }
                    before.push(Span::styled(
                        "│",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ));
                    if !a.is_empty() {
                        after.push(Span::styled(a.to_string(), sp.style));
                    }
                    found = true;
                } else if !found {
                    before.push(sp);
                    col += len;
                } else {
                    after.push(sp);
                }
            }
            if !found {
                before.push(Span::styled(
                    "│",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ));
            }
            spans.extend(before);
            spans.extend(after);
        } else {
            spans.extend(content_spans);
        }
        source_lines.push(Line::from(spans));
    }

    let title = if app.dirty { " Source * " } else { " Source " };
    let border_color = if app.focus == 0 { Color::Cyan } else { Color::DarkGray };
    f.render_widget(
        Paragraph::new(source_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color)),
            )
            .wrap(Wrap { trim: false }),
        editor_area,
    );

    if app.need_preview_refresh {
        app.refresh_preview();
    }
    let preview_total = app.preview_lines.len();
    let p_start = app.preview_scroll.min(preview_total.saturating_sub(1));
    let p_end = (p_start + preview_height).min(preview_total);
    let visible: Vec<Line> = app.preview_lines[p_start..p_end].to_vec();
    let preview_title = format!(
        " Preview ({} blocks, {} reused) ",
        app.preview.blocks.len(),
        app.last_reuse
    );
    let p_border = if app.focus == 1 { Color::Green } else { Color::DarkGray };
    f.render_widget(
        Paragraph::new(visible)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(preview_title)
                    .border_style(Style::default().fg(p_border)),
            )
            .wrap(Wrap { trim: false }),
        preview_area,
    );

    let path_str = app
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "untitled.md".into());
    let status_text = format!(
        " {} │ {} │ Ln {}, Col {} │ scroll E:{} P:{} ",
        app.status,
        path_str,
        app.buffer.cursor_row + 1,
        app.buffer.cursor_col + 1,
        app.buffer.scroll,
        app.preview_scroll
    );
    f.render_widget(
        Paragraph::new(status_text).style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        chunks[1],
    );

    if app.show_help {
        let help_area = centered_rect(70, 70, size);
        let help_text = vec![
            Line::from(Span::styled(" Shortcuts ", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  Ctrl+S / Ctrl+Q     Save / Quit"),
            Line::from("  Tab                 Switch focus"),
            Line::from("  Mouse wheel         Scroll"),
            Line::from("  Press any key to close"),
        ];
        f.render_widget(
            Paragraph::new(help_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Help ")
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .style(Style::default().bg(Color::Black)),
            help_area,
        );
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
