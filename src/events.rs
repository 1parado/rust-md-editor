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
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            if let Err(e) = app.save() {
                                app.status = format!("Save error: {}", e);
                            }
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
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_sub(1);
                            }
                        }
                        (KeyCode::Down, _) => {
                            if app.focus == 0 {
                                app.buffer.move_down();
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_add(1);
                            }
                        }
                        (KeyCode::PageUp, _) => {
                            if app.focus == 0 {
                                app.buffer.scroll_by(-10, 20);
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_sub(10);
                            }
                        }
                        (KeyCode::PageDown, _) => {
                            if app.focus == 0 {
                                app.buffer.scroll_by(10, 20);
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_add(10);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(me) => {
                    let size = terminal.size()?;
                    let mid_x = size.width / 2;
                    match me.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            if me.column < mid_x {
                                app.focus = 0;
                                let row = (me.row as usize).saturating_sub(1) + app.buffer.scroll;
                                if row < app.buffer.lines.len() {
                                    app.buffer.cursor_row = row;
                                    let col = (me.column as usize).saturating_sub(6);
                                    app.buffer.cursor_col =
                                        col.min(app.buffer.lines[row].len());
                                }
                            } else {
                                app.focus = 1;
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if app.focus == 0 || me.column < mid_x {
                                app.buffer.scroll_by(-3, 20);
                                if app.scroll_sync {
                                    app.preview_scroll = app.preview_scroll.saturating_sub(3);
                                }
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_sub(3);
                                if app.scroll_sync {
                                    app.buffer.scroll_by(-3, 20);
                                }
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if app.focus == 0 || me.column < mid_x {
                                app.buffer.scroll_by(3, 20);
                                if app.scroll_sync {
                                    app.preview_scroll = app.preview_scroll.saturating_add(3);
                                }
                            } else {
                                app.preview_scroll = app.preview_scroll.saturating_add(3);
                                if app.scroll_sync {
                                    app.buffer.scroll_by(3, 20);
                                }
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
