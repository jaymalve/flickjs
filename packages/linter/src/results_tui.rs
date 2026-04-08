//! Interactive TUI for browsing `flint check` results.

use crate::{
    category_display_name, diagnostic_counts, diagnostic_help_text, group_results_for_display,
    print_pretty, PrettyEntry, Summary,
};
use crate::rule_catalog::why_for_rule;
use crate::rules::{FixSafety, LintResult, Severity};
use crate::tui_common::{
    self, ERROR_COLOR, PANEL_BG, PANEL_BG_SUBTLE, SCROLLBAR_THUMB, SCROLLBAR_TRACK, SELECT_BG,
    TEXT_FAINT, TEXT_MUTED, TEXT_PRIMARY, WARN_COLOR,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::tty::IsTty;
use miette::{IntoDiagnostic, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub(crate) fn print_or_fallback(
    results: &[LintResult],
    elapsed: std::time::Duration,
    scan_root: &Path,
    files_scanned: usize,
    show_score: bool,
) -> Summary {
    if !io::stdout().is_tty() {
        eprintln!("flint: stdout is not a tty; using pretty output instead of interactive TUI");
        return print_pretty(results, elapsed, files_scanned, show_score);
    }

    let score = if show_score {
        Some(crate::scoring::HealthScore::compute(results, files_scanned))
    } else {
        None
    };

    if let Err(e) = run(results, elapsed, scan_root, score.as_ref()) {
        eprintln!("flint: TUI failed ({e}); falling back to pretty output");
        return print_pretty(results, elapsed, files_scanned, show_score);
    }

    Summary {
        errors: diagnostic_counts(results).0,
        score,
    }
}

fn run(
    results: &[LintResult],
    elapsed: std::time::Duration,
    scan_root: &Path,
    score: Option<&crate::scoring::HealthScore>,
) -> Result<()> {
    let categories = group_results_for_display(results);
    let mut app = ResultsApp::new(categories, elapsed, scan_root.to_path_buf(), score);
    let mut terminal = tui_common::TerminalSession::enter()?;

    loop {
        terminal
            .terminal
            .draw(|frame| render(frame, &mut app))
            .into_diagnostic()?;

        if !event::poll(Duration::from_millis(200)).into_diagnostic()? {
            continue;
        }

        match event::read().into_diagnostic()? {
            Event::Key(key) => {
                match app.handle_key(key) {
                    AppAction::Quit => break,
                    AppAction::OpenSelectedLocation => {
                        if let Some(entry) = app.selected_entry() {
                            match terminal.suspend_for_external_command(|| {
                                spawn_open_at_span(entry.file, &entry.diagnostic.span)
                            }) {
                                Ok(Ok(())) => {}
                                Ok(Err(e)) => {
                                    let _ = writeln!(io::stderr(), "flint: open in editor: {e}");
                                }
                                Err(e) => {
                                    let _ = writeln!(io::stderr(), "flint: TUI suspend failed: {e}");
                                }
                            }
                        }
                    }
                    AppAction::Continue => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Issues,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppAction {
    Continue,
    Quit,
    OpenSelectedLocation,
}

struct ResultsApp<'a> {
    categories: Vec<(String, Vec<PrettyEntry<'a>>)>,
    elapsed: std::time::Duration,
    scan_root: PathBuf,
    score: Option<crate::scoring::HealthScore>,
    selected_issue: usize,
    focus: FocusPane,
    search: String,
    search_mode: bool,
    errors_only: bool,
    detail_scroll: u16,
    issues_scrollbar: ScrollbarState,
    detail_scrollbar: ScrollbarState,
}

impl<'a> ResultsApp<'a> {
    fn new(
        categories: Vec<(String, Vec<PrettyEntry<'a>>)>,
        elapsed: std::time::Duration,
        scan_root: PathBuf,
        score: Option<&crate::scoring::HealthScore>,
    ) -> Self {
        let mut app = Self {
            categories,
            elapsed,
            scan_root,
            score: score.cloned(),
            selected_issue: 0,
            focus: FocusPane::Issues,
            search: String::new(),
            search_mode: false,
            errors_only: false,
            detail_scroll: 0,
            issues_scrollbar: ScrollbarState::default(),
            detail_scrollbar: ScrollbarState::default(),
        };
        app.normalize_selection();
        app
    }

    fn handle_key(&mut self, key: KeyEvent) -> AppAction {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return AppAction::Quit;
        }

        if self.search_mode {
            self.handle_search_key(key);
            return AppAction::Continue;
        }

        match key.code {
            KeyCode::Char('q') => AppAction::Quit,
            KeyCode::Tab | KeyCode::BackTab => {
                self.focus = match self.focus {
                    FocusPane::Issues => FocusPane::Detail,
                    FocusPane::Detail => FocusPane::Issues,
                };
                AppAction::Continue
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = FocusPane::Issues;
                AppAction::Continue
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.focus = FocusPane::Detail;
                AppAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.focus {
                    FocusPane::Issues => self.move_issue(-1),
                    FocusPane::Detail => self.scroll_detail(-1),
                }
                AppAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.focus {
                    FocusPane::Issues => self.move_issue(1),
                    FocusPane::Detail => self.scroll_detail(1),
                }
                AppAction::Continue
            }
            KeyCode::Home => {
                self.jump_to_start();
                AppAction::Continue
            }
            KeyCode::End => {
                self.jump_to_end();
                AppAction::Continue
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                AppAction::Continue
            }
            KeyCode::Char('g') => {
                self.reset_filters();
                AppAction::Continue
            }
            KeyCode::Char('e') => {
                self.errors_only = !self.errors_only;
                self.selected_issue = 0;
                self.detail_scroll = 0;
                self.normalize_selection();
                AppAction::Continue
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                if self.selected_entry().is_some() {
                    AppAction::OpenSelectedLocation
                } else {
                    AppAction::Continue
                }
            }
            _ => AppAction::Continue,
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => self.search_mode = false,
            KeyCode::Backspace => {
                self.search.pop();
                self.selected_issue = 0;
                self.detail_scroll = 0;
                self.normalize_selection();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.search.push(ch);
                self.selected_issue = 0;
                self.detail_scroll = 0;
                self.normalize_selection();
            }
            _ => {}
        }
    }

    fn move_issue(&mut self, delta: isize) {
        let visible = self.visible_issues();
        let len = visible.len();
        if len == 0 {
            self.selected_issue = 0;
            return;
        }
        self.selected_issue = shift_index(self.selected_issue, len, delta);
        self.detail_scroll = 0;
    }

    fn scroll_detail(&mut self, delta: isize) {
        let new = self.detail_scroll as isize + delta;
        self.detail_scroll = new.max(0) as u16;
    }

    fn jump_to_start(&mut self) {
        match self.focus {
            FocusPane::Issues => self.selected_issue = 0,
            FocusPane::Detail => self.detail_scroll = 0,
        }
        self.normalize_selection();
    }

    fn jump_to_end(&mut self) {
        match self.focus {
            FocusPane::Issues => {
                let len = self.visible_issues().len();
                if len > 0 {
                    self.selected_issue = len - 1;
                }
            }
            FocusPane::Detail => {
                self.detail_scroll = self.detail_scroll.saturating_add(100);
            }
        }
        self.normalize_selection();
    }

    fn reset_filters(&mut self) {
        self.selected_issue = 0;
        self.focus = FocusPane::Issues;
        self.search.clear();
        self.search_mode = false;
        self.errors_only = false;
        self.detail_scroll = 0;
        self.normalize_selection();
    }

    fn normalize_selection(&mut self) {
        if self.categories.is_empty() {
            self.selected_issue = 0;
            return;
        }

        let visible_len = self.visible_issues().len();
        if visible_len == 0 {
            self.selected_issue = 0;
        } else if self.selected_issue >= visible_len {
            self.selected_issue = visible_len - 1;
        }
    }

    fn visible_issues(&self) -> Vec<&PrettyEntry<'a>> {
        let query = self.search.trim().to_ascii_lowercase();
        self.categories
            .iter()
            .flat_map(|(_, entries)| entries.iter())
            .filter(|entry| {
                if self.errors_only && entry.diagnostic.severity != Severity::Error {
                    return false;
                }
                if query.is_empty() {
                    return true;
                }
                let path = entry.file.to_string_lossy().to_ascii_lowercase();
                let message = entry.diagnostic.message.to_ascii_lowercase();
                let rule = entry.diagnostic.rule_name.to_ascii_lowercase();
                path.contains(&query) || message.contains(&query) || rule.contains(&query)
            })
            .collect()
    }

    fn selected_entry(&self) -> Option<&PrettyEntry<'a>> {
        let visible = self.visible_issues();
        visible.get(self.selected_issue).copied()
    }

    fn total_issues(&self) -> usize {
        self.categories.iter().map(|(_, entries)| entries.len()).sum()
    }

    fn error_stats(&self) -> (usize, usize) {
        let mut errors = 0;
        let mut warnings = 0;
        for (_, entries) in &self.categories {
            for e in entries {
                match e.diagnostic.severity {
                    Severity::Error => errors += 1,
                    Severity::Warning => warnings += 1,
                }
            }
        }
        (errors, warnings)
    }

    fn category_key_for_entry(&self, entry: &PrettyEntry<'a>) -> Option<&str> {
        self.categories.iter().find_map(|(key, entries)| {
            entries
                .iter()
                .find(|candidate| std::ptr::eq(*candidate, entry))
                .map(|_| key.as_str())
        })
    }

    fn display_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.scan_root)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

fn parse_line_col(span: &str) -> (u32, u32) {
    let mut parts = span.split(':');
    let line = parts
        .next()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(1);
    let col = parts
        .next()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(1);
    (line, col)
}

/// Opens `path` at `line:col` from the diagnostic span.
///
/// Resolution order:
/// 1. `FLINT_OPEN` — executable name or path, tried with `-g` / `--goto` / bare `path:line:col`.
/// 2. On macOS, bundled CLIs inside `.app` (`Contents/Resources/app/bin/cursor|code`).  
///    Using `open -a Cursor --args …` is **not** reliable: it launches the GUI binary, which often
///    drops VS Code–style CLI flags; the `bin/cursor` shim is what understands `-g`.
/// 3. `cursor` / `code` on `PATH`.
///
/// VS Code–family CLIs are invoked **once** with `-r -g path:line:col` (`-r` / `--reuse-window` targets an
/// already running instance and avoids multiple CLI attempts that make the macOS dock flash).
fn spawn_open_at_span(path: &Path, span: &str) -> io::Result<()> {
    let (line, col) = parse_line_col(span);
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let spec = format!("{}:{line}:{col}", abs.display());

    let mut candidates = Vec::new();
    let mut push = |s: &str| {
        if !candidates.iter().any(|e: &String| e == s) {
            candidates.push(s.to_string());
        }
    };
    if let Ok(p) = std::env::var("FLINT_OPEN") {
        let p = p.trim();
        if !p.is_empty() {
            push(p);
        }
    }

    #[cfg(target_os = "macos")]
    {
        for bundled in macos_editor_cli_candidates() {
            push(bundled.as_str());
        }
    }

    push("cursor");
    push("code");

    let mut last_err = None;
    for program in candidates {
        match try_open_vscode_goto(program.as_str(), &spec) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
        }
    }

    let _ = writeln!(
        io::stderr(),
        "flint: could not open an editor at {} (set FLINT_OPEN to your editor CLI, e.g. cursor or code)",
        spec
    );
    Err(last_err.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "no editor command succeeded")
    }))
}

/// Standard install locations for the real VS Code–compatible CLIs (not `open -a`).
#[cfg(target_os = "macos")]
fn macos_editor_cli_candidates() -> Vec<String> {
    let mut v = Vec::new();
    for rel in [
        "Cursor.app/Contents/Resources/app/bin/cursor",
        "Visual Studio Code.app/Contents/Resources/app/bin/code",
        "Visual Studio Code - Insiders.app/Contents/Resources/app/bin/code",
    ] {
        let p = Path::new("/Applications").join(rel);
        if p.is_file() {
            v.push(p.to_string_lossy().into_owned());
        }
    }
    v
}

fn is_vscode_family_cli(program: &str) -> bool {
    let lower = program.to_ascii_lowercase();
    let base = Path::new(program)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    base == "cursor"
        || base == "code"
        || base == "code-insiders"
        || lower.contains("visual studio code")
        || lower.contains("/resources/app/bin/cursor")
        || lower.contains("/resources/app/bin/code")
}

/// Single spawn per candidate so we do not run `-g` / `--goto` / bare retries (each can briefly
/// activate the app in the macOS dock).
fn try_open_vscode_goto(program: &str, path_line_col: &str) -> io::Result<()> {
    if is_vscode_family_cli(program) {
        try_spawn_detached(program, &["-r", "-g", path_line_col])
    } else {
        // Custom `FLINT_OPEN` (e.g. a wrapper): keep one `-g` attempt; avoid `-r` (e.g. vim uses `-r` for recovery).
        try_spawn_detached(program, &["-g", path_line_col])
    }
}

fn try_spawn_detached(program: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> io::Result<()> {
    let mut child = Command::new(program.as_ref())
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

fn shift_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    let next = current as isize + delta;
    next.clamp(0, len as isize - 1) as usize
}

fn render(frame: &mut Frame, app: &mut ResultsApp<'_>) {
    let header_height = if app.score.is_some() { 3 } else { 2 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    render_issues(frame, body[0], app);
    render_detail(frame, body[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &ResultsApp<'_>) {
    let (errors, warnings) = app.error_stats();
    let scan = app.display_path(&app.scan_root.clone());

    let line = Line::from(vec![
        Span::styled(
            " Flint Check",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {:.2?}", app.elapsed),
            Style::default().fg(TEXT_MUTED),
        ),
        Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
        Span::styled(
            format!("{errors} errors"),
            Style::default()
                .fg(if errors > 0 { ERROR_COLOR } else { TEXT_MUTED })
                .add_modifier(if errors > 0 {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{warnings} warnings"),
            Style::default()
                .fg(if warnings > 0 { WARN_COLOR } else { TEXT_MUTED })
                .add_modifier(if warnings > 0 {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
        ),
        Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
        Span::styled(scan, Style::default().fg(TEXT_MUTED)),
    ]);

    if let Some(ref health) = app.score {
        let score_color = match health.score {
            90..=100 => ratatui::style::Color::Green,
            70..=89 => ratatui::style::Color::Yellow,
            _ => ratatui::style::Color::Red,
        };
        let score_line = Line::from(vec![
            Span::styled(
                format!(" Score: {}/100", health.score),
                Style::default()
                    .fg(score_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", health.progress_bar()),
                Style::default().fg(score_color),
            ),
            Span::styled(
                format!(" {}", health.ascii_face()),
                Style::default()
                    .fg(score_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", health.label()),
                Style::default().fg(TEXT_MUTED),
            ),
        ]);
        let header = Paragraph::new(Text::from(vec![line, score_line, Line::from("")]))
            .style(Style::default().bg(PANEL_BG));
        frame.render_widget(header, area);
    } else {
        let header = Paragraph::new(Text::from(vec![line, Line::from("")]))
            .style(Style::default().bg(PANEL_BG));
        frame.render_widget(header, area);
    }
}

fn highlight_spans(text: &str, query: &str, base_style: Style) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    let lower = text.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();
    let mut spans = Vec::new();
    let mut last = 0;

    for (start, _) in lower.match_indices(&query_lower) {
        if start > last {
            spans.push(Span::styled(text[last..start].to_string(), base_style));
        }
        spans.push(Span::styled(
            text[start..start + query.len()].to_string(),
            base_style.add_modifier(Modifier::UNDERLINED),
        ));
        last = start + query.len();
    }
    if last < text.len() {
        spans.push(Span::styled(text[last..].to_string(), base_style));
    }
    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), base_style));
    }
    spans
}

fn render_issues(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut ResultsApp<'_>) {
    let visible = app.visible_issues();
    if visible.is_empty() {
        let msg = if app.total_issues() == 0 {
            "No issues to display."
        } else {
            "No issues match filters."
        };
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(msg, Style::default().fg(TEXT_PRIMARY))),
            Line::from(Span::styled(
                "/ filter · e errors-only · g reset",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Issues", app.focus == FocusPane::Issues))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    }

    let query = app.search.trim().to_string();
    let items: Vec<ListItem> = visible
        .iter()
        .map(|entry| {
            let (label, fg, bg) = tui_common::severity_badge(&entry.diagnostic.severity);
            let path = app.display_path(entry.file);

            let mut line1 = vec![
                Span::styled(
                    format!(" {label} "),
                    Style::default()
                        .fg(fg)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default()),
            ];
            line1.extend(highlight_spans(
                &entry.diagnostic.rule_name,
                &query,
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ));

            let mut line2 = vec![Span::styled("        ", Style::default())];
            line2.extend(highlight_spans(
                &format!("{}:{}", path, entry.diagnostic.span),
                &query,
                Style::default().fg(TEXT_MUTED),
            ));

            ListItem::new(Text::from(vec![Line::from(line1), Line::from(line2)]))
        })
        .collect();

    let total = items.len();
    let list = List::new(items)
        .block(tui_common::block("Issues", app.focus == FocusPane::Issues))
        .highlight_style(
            Style::default()
                .bg(SELECT_BG)
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default();
    state.select(Some(app.selected_issue));
    frame.render_stateful_widget(list, area, &mut state);

    app.issues_scrollbar = app
        .issues_scrollbar
        .content_length(total)
        .position(app.selected_issue);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .track_style(Style::default().fg(SCROLLBAR_TRACK))
        .thumb_style(Style::default().fg(SCROLLBAR_THUMB));
    let scrollbar_area = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 0,
    });
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut app.issues_scrollbar);
}

fn render_detail(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut ResultsApp<'_>) {
    let is_focused = app.focus == FocusPane::Detail;

    let Some(entry) = app.selected_entry() else {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Select an issue for detail.",
                Style::default().fg(TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                "Rule, help text, and fix info appear here.",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Detail", is_focused))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    };

    let (severity_label, severity_fg, severity_bg) =
        tui_common::severity_badge(&entry.diagnostic.severity);
    let group_title = app
        .category_key_for_entry(entry)
        .map(category_display_name)
        .unwrap_or("Other");

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                format!(" {severity_label} "),
                Style::default()
                    .fg(severity_fg)
                    .bg(severity_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                entry.diagnostic.rule_name.as_str(),
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            entry.diagnostic.message.as_str(),
            Style::default().fg(TEXT_PRIMARY),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Why",
            Style::default()
                .fg(TEXT_MUTED)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            why_for_rule(&entry.diagnostic.rule_name),
            Style::default().fg(TEXT_MUTED),
        )),
        Line::from(""),
        detail_line(
            "File",
            &format!("{}:{}", app.display_path(entry.file), entry.diagnostic.span),
        ),
        detail_line("Category", group_title),
    ];

    if let Some(kind) = entry.diagnostic.node_kind.as_deref() {
        lines.push(detail_line("Node", kind));
    }
    if let Some(sym) = entry.diagnostic.symbol.as_deref() {
        lines.push(detail_line("Symbol", sym));
    }

    if let Some(fix) = &entry.diagnostic.fix {
        let safety = match fix.safety {
            FixSafety::Safe => "safe",
            FixSafety::SemanticSafe => "semantic-safe",
            FixSafety::Risky => "risky",
            FixSafety::SuppressOnly => "suppress-only",
        };
        let desc = fix
            .description
            .as_deref()
            .unwrap_or("Quick fix available");
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Fix",
            Style::default()
                .fg(TEXT_MUTED)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            desc,
            Style::default().fg(TEXT_PRIMARY),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "Safety: {safety} · replacement {} chars",
                fix.replacement.len()
            ),
            Style::default().fg(TEXT_FAINT),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Help",
        Style::default()
            .fg(TEXT_MUTED)
            .add_modifier(Modifier::BOLD),
    )));
    if let Some(help) = diagnostic_help_text(entry.diagnostic) {
        lines.push(Line::from(Span::styled(
            help,
            Style::default().fg(TEXT_PRIMARY),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "—",
            Style::default().fg(TEXT_FAINT),
        )));
    }

    let total_lines = lines.len();
    let panel = Paragraph::new(Text::from(lines))
        .block(tui_common::block("Detail", is_focused))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0))
        .style(Style::default().bg(PANEL_BG_SUBTLE));
    frame.render_widget(panel, area);

    app.detail_scrollbar = app
        .detail_scrollbar
        .content_length(total_lines)
        .position(app.detail_scroll as usize);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .track_style(Style::default().fg(SCROLLBAR_TRACK))
        .thumb_style(Style::default().fg(SCROLLBAR_THUMB));
    let scrollbar_area = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 0,
    });
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut app.detail_scrollbar);
}

fn detail_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), Style::default().fg(TEXT_MUTED)),
        Span::styled(value.to_string(), Style::default().fg(TEXT_PRIMARY)),
    ])
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &ResultsApp<'_>) {
    let filter = if app.errors_only {
        "errors only"
    } else {
        "all severities"
    };
    let search_line = if app.search_mode {
        format!("Search: {}_", app.search)
    } else if app.search.is_empty() {
        "Search: /".to_string()
    } else {
        format!("Search: {}", app.search)
    };
    let legend = if app.search_mode {
        "type to filter · backspace · enter/esc close · q quit"
    } else {
        "j/k move · tab pane · h/l focus · o open · / search · e errors · g reset · q quit"
    };

    let footer = Paragraph::new(Text::from(vec![Line::from(vec![
        Span::styled(format!(" {search_line}"), Style::default().fg(TEXT_PRIMARY)),
        Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
        Span::styled(format!("Filter: {filter}"), Style::default().fg(TEXT_MUTED)),
        Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
        Span::styled(legend, Style::default().fg(TEXT_MUTED)),
    ])]))
    .style(Style::default().bg(PANEL_BG));
    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{LintDiagnostic, RuleOrigin};
    use std::path::PathBuf;

    fn sample_results() -> Vec<LintResult> {
        vec![
            LintResult {
                file: PathBuf::from("/proj/src/a.ts"),
                diagnostics: vec![LintDiagnostic {
                    rule_name: "no-console".to_string(),
                    message: "Unexpected console".to_string(),
                    span: "1:1".to_string(),
                    severity: Severity::Warning,
                    origin: RuleOrigin::BuiltIn,
                    fix: None,
                    byte_start: 0,
                    byte_end: 0,
                    node_kind: None,
                    symbol: None,
                }],
            },
            LintResult {
                file: PathBuf::from("/proj/src/b.ts"),
                diagnostics: vec![LintDiagnostic {
                    rule_name: "no-unused-vars".to_string(),
                    message: "unused x".to_string(),
                    span: "2:3".to_string(),
                    severity: Severity::Error,
                    origin: RuleOrigin::BuiltIn,
                    fix: None,
                    byte_start: 0,
                    byte_end: 0,
                    node_kind: None,
                    symbol: None,
                }],
            },
        ]
    }

    #[test]
    fn visible_issues_include_all_categories() {
        let results = vec![
            LintResult {
                file: PathBuf::from("/proj/src/a.ts"),
                diagnostics: vec![
                    LintDiagnostic {
                        rule_name: "no-console".to_string(),
                        message: "c".to_string(),
                        span: "1:1".to_string(),
                        severity: Severity::Warning,
                        origin: RuleOrigin::BuiltIn,
                        fix: None,
                        byte_start: 0,
                        byte_end: 0,
                        node_kind: None,
                        symbol: None,
                    },
                    LintDiagnostic {
                        rule_name: "prefer-const".to_string(),
                        message: "p".to_string(),
                        span: "2:1".to_string(),
                        severity: Severity::Warning,
                        origin: RuleOrigin::BuiltIn,
                        fix: None,
                        byte_start: 1,
                        byte_end: 1,
                        node_kind: None,
                        symbol: None,
                    },
                ],
            },
            LintResult {
                file: PathBuf::from("/proj/src/b.ts"),
                diagnostics: vec![LintDiagnostic {
                    rule_name: "parse-error".to_string(),
                    message: "syntax".to_string(),
                    span: "1:1".to_string(),
                    severity: Severity::Error,
                    origin: RuleOrigin::BuiltIn,
                    fix: None,
                    byte_start: 0,
                    byte_end: 0,
                    node_kind: None,
                    symbol: None,
                }],
            },
        ];
        let cats = group_results_for_display(&results);
        let app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"), None);
        let visible = app.visible_issues();
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn search_filters_issues() {
        let results = sample_results();
        let cats = group_results_for_display(&results);
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"), None);
        app.search = "console".to_string();
        let visible = app.visible_issues();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].diagnostic.rule_name, "no-console");
    }

    #[test]
    fn reset_clears_search_and_errors_only() {
        let results = sample_results();
        let cats = group_results_for_display(&results);
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"), None);
        app.errors_only = true;
        app.search = "noop".to_string();
        app.reset_filters();
        assert!(!app.errors_only);
        assert!(app.search.is_empty());
    }

    #[test]
    fn detail_scroll_does_not_go_negative() {
        let results = sample_results();
        let cats = group_results_for_display(&results);
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"), None);
        app.focus = FocusPane::Detail;
        app.scroll_detail(-5);
        assert_eq!(app.detail_scroll, 0);
        app.scroll_detail(3);
        assert_eq!(app.detail_scroll, 3);
        app.scroll_detail(-1);
        assert_eq!(app.detail_scroll, 2);
    }

    #[test]
    fn tab_cycles_through_all_panes() {
        let results = sample_results();
        let cats = group_results_for_display(&results);
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"), None);
        assert_eq!(app.focus, FocusPane::Issues);
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, FocusPane::Detail);
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, FocusPane::Issues);
    }

    #[test]
    fn parse_line_col_handles_common_spans() {
        assert_eq!(parse_line_col("1:1"), (1, 1));
        assert_eq!(parse_line_col("42:7"), (42, 7));
        assert_eq!(parse_line_col("10"), (10, 1));
    }

    #[test]
    fn parse_line_col_defaults_on_garbage() {
        assert_eq!(parse_line_col(""), (1, 1));
        assert_eq!(parse_line_col("0:0"), (1, 1));
        assert_eq!(parse_line_col("abc"), (1, 1));
    }

    #[test]
    fn vscode_family_cli_detection() {
        assert!(super::is_vscode_family_cli("cursor"));
        assert!(super::is_vscode_family_cli("code"));
        assert!(super::is_vscode_family_cli(
            "/Applications/Cursor.app/Contents/Resources/app/bin/cursor"
        ));
        assert!(!super::is_vscode_family_cli("vim"));
        assert!(!super::is_vscode_family_cli("/usr/bin/emacs"));
    }

    #[test]
    fn open_key_works_from_either_pane_with_selection() {
        let results = sample_results();
        let cats = group_results_for_display(&results);
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"), None);
        assert_eq!(app.focus, FocusPane::Issues);
        assert_eq!(
            app.handle_key(KeyEvent::from(KeyCode::Char('o'))),
            AppAction::OpenSelectedLocation
        );
        app.focus = FocusPane::Detail;
        assert_eq!(
            app.handle_key(KeyEvent::from(KeyCode::Char('o'))),
            AppAction::OpenSelectedLocation
        );
    }
}
