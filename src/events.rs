//! Event loop
use crate::app::App;
use crate::draw::ui;
use anyhow::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Stdout;
use std::time::{Duration, Instant};

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut app: App,
) -> Result<()> {
    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.show_help {
                        app.show_help = false;
                        continue;
                    }

                    // ---- Find mode ----
                    if app.finding {
                        match (key.code, key.modifiers) {
                            (KeyCode::Esc, _) => app.stop_find(),
                            (KeyCode::Enter, _) => app.find_next(true),
                            (KeyCode::Char('n'), KeyModifiers::NONE) => app.find_next(true),
                            (KeyCode::Char('N'), KeyModifiers::SHIFT) => app.find_next(false),
                            (KeyCode::Backspace, _) => {
                                app.find_query.pop();
                                app.status = format!(
                                    "Find: {}_  (Enter/n next · N prev · Esc)",
                                    app.find_query
                                );
                            }
                            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                                app.find_query.push(c);
                                app.status = format!(
                                    "Find: {}_  (Enter/n next · N prev · Esc)",
                                    app.find_query
                                );
                            }
                            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                                // keep open, re-run
                                app.find_next(true);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            if app.request_quit() {
                                return Ok(());
                            }
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            if let Err(e) = app.save() {
                                app.status = format!("Save error: {}", e);
                            }
                        }
                        (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                            app.start_find();
                        }
                        (KeyCode::Char('h'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('?'), _) => {
                            app.show_help = true;
                        }
                        (KeyCode::Tab, _) => {
                            app.focus = 1 - app.focus;
                        }
                        (KeyCode::Char('S'), KeyModifiers::SHIFT) if app.focus == 1 => {
                            app.scroll_sync = !app.scroll_sync;
                            app.status = format!(
                                "scroll-sync {}",
                                if app.scroll_sync { "ON" } else { "OFF" }
                            );
                        }
                        (KeyCode::Home, _) if app.focus == 0 => app.buffer.home(),
                        (KeyCode::End, _) if app.focus == 0 => app.buffer.end(),
                        (KeyCode::Left, KeyModifiers::CONTROL) if app.focus == 0 => {
                            app.buffer.move_word_left();
                        }
                        (KeyCode::Right, KeyModifiers::CONTROL) if app.focus == 0 => {
                            app.buffer.move_word_right();
                        }
                        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT)
                            if app.focus == 0 =>
                        {
                            app.buffer.insert_char(c);
                            app.on_edit();
                        }
                        (KeyCode::Enter, _) if app.focus == 0 => {
                            app.buffer.insert_newline();
                            app.on_edit();
                        }
                        (KeyCode::Backspace, _) if app.focus == 0 => {
                            app.buffer.backspace();
                            app.on_edit();
                        }
                        (KeyCode::Delete, _) if app.focus == 0 => {
                            app.buffer.delete();
                            app.on_edit();
                        }
                        (KeyCode::Left, _) if app.focus == 0 => app.buffer.move_left(),
                        (KeyCode::Right, _) if app.focus == 0 => app.buffer.move_right(),
                        (KeyCode::Up, _) => {
                            if app.focus == 0 {
                                app.buffer.move_up();
                                app.sync_preview_from_editor();
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_sub(1);
                                app.sync_editor_from_preview();
                            }
                        }
                        (KeyCode::Down, _) => {
                            if app.focus == 0 {
                                app.buffer.move_down();
                                app.sync_preview_from_editor();
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_add(1);
                                app.sync_editor_from_preview();
                            }
                        }
                        (KeyCode::PageUp, _) => {
                            if app.focus == 0 {
                                app.buffer.scroll_by(-10, 20);
                                app.sync_preview_from_editor();
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_sub(10);
                                app.sync_editor_from_preview();
                            }
                        }
                        (KeyCode::PageDown, _) => {
                            if app.focus == 0 {
                                app.buffer.scroll_by(10, 20);
                                app.sync_preview_from_editor();
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_add(10);
                                app.sync_editor_from_preview();
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(me) => {
                    if app.finding {
                        continue;
                    }
                    let size = terminal.size()?;
                    let mid_x = size.width / 2;
                    match me.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            if me.column < mid_x {
                                app.focus = 0;
                                let row =
                                    (me.row as usize).saturating_sub(1) + app.buffer.scroll;
                                if row < app.buffer.lines.len() {
                                    app.buffer.cursor_row = row;
                                    let col = (me.column as usize).saturating_sub(6)
                                        + app.buffer.h_scroll;
                                    let line = &app.buffer.lines[row];
                                    let mut c = col.min(line.len());
                                    while c > 0 && !line.is_char_boundary(c) {
                                        c -= 1;
                                    }
                                    app.buffer.cursor_col = c;
                                }
                            } else {
                                app.focus = 1;
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if app.focus == 0 || me.column < mid_x {
                                app.buffer.scroll_by(-3, 20);
                                app.sync_preview_from_editor();
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_sub(3);
                                app.sync_editor_from_preview();
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if app.focus == 0 || me.column < mid_x {
                                app.buffer.scroll_by(3, 20);
                                app.sync_preview_from_editor();
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_add(3);
                                app.sync_editor_from_preview();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if app.need_preview_refresh && last_tick.elapsed() >= Duration::from_millis(80) {
            app.refresh_preview();
            last_tick = Instant::now();
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}
