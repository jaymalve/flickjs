//! Shared ratatui styling and terminal setup for interactive TUIs.

use crate::rules::Severity;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{IntoDiagnostic, Result};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
    Terminal,
};
use std::io::{self, Stdout};

pub const BORDER_ACTIVE: Color = Color::Rgb(245, 245, 245);
pub const BORDER_IDLE: Color = Color::Rgb(72, 72, 72);
pub const PANEL_BG: Color = Color::Rgb(10, 10, 10);
pub const PANEL_BG_SUBTLE: Color = Color::Rgb(18, 18, 18);
pub const TEXT_PRIMARY: Color = Color::Rgb(244, 244, 244);
pub const TEXT_MUTED: Color = Color::Rgb(160, 160, 160);
pub const TEXT_FAINT: Color = Color::Rgb(104, 104, 104);
pub const SELECT_BG: Color = Color::Rgb(42, 42, 42);
pub const WARN_COLOR: Color = Color::Rgb(184, 184, 184);
pub const ERROR_COLOR: Color = Color::Rgb(255, 255, 255);
pub const OK_COLOR: Color = Color::Rgb(222, 222, 222);

pub struct TerminalSession {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    pub fn enter() -> Result<Self> {
        enable_raw_mode().into_diagnostic()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).into_diagnostic()?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).into_diagnostic()?;
        terminal.clear().into_diagnostic()?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

pub fn block(title: &'static str, active: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(if active { BORDER_ACTIVE } else { TEXT_MUTED })
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(if active { BORDER_ACTIVE } else { BORDER_IDLE }))
        .style(Style::default().bg(PANEL_BG))
}

pub fn severity_badge(severity: &Severity) -> (&'static str, Color) {
    match severity {
        Severity::Error => ("error", ERROR_COLOR),
        Severity::Warning => ("warn", WARN_COLOR),
    }
}

pub fn severity_rank(severity: &Severity) -> usize {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
    }
}
