//! Lightweight terminal Markdown editor v0.2
//!
//! Features:
//! - Incremental block-based preview (streamdown-style: only re-render changed blocks)
//! - Syntax highlighting via syntect (source + code fences in preview)
//! - Scroll sync between editor & preview + mouse support

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pulldown_cmark::{CodeBlockKind, Event as MdEvent, Options, Parser, Tag, TagEnd};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env,
    fs,
    io::{self, Stdout},
    path::PathBuf,
    time::{Duration, Instant},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SynStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

// PLACEHOLDER_SEE_FULL_FILE
fn main() -> Result<()> {
    eprintln!("Please use the full source from the repository after complete push.");
    Ok(())
}
