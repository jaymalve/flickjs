//! Interactive TUI for browsing `flint check` results.

use crate::{
    category_display_name, diagnostic_counts, diagnostic_help_text, group_results_for_display,
    print_pretty, PrettyEntry, Summary,
};
use crate::rules::{FixSafety, LintResult, Severity};
use crate::tui_common::{self, PANEL_BG, PANEL_BG_SUBTLE, SELECT_BG, TEXT_FAINT, TEXT_MUTED, TEXT_PRIMARY};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::tty::IsTty;
use miette::{IntoDiagnostic, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(crate) fn print_or_fallback(
    results: &[LintResult],
    elapsed: std::time::Duration,
    scan_root: &Path,
) -> Summary {
    let summary = Summary {
        errors: diagnostic_counts(results).0,
    };

    if !io::stdout().is_tty() {
        eprintln!("flint: stdout is not a tty; using pretty output instead of interactive TUI");
        print_pretty(results, elapsed);
        return summary;
    }

    if let Err(e) = run(results, elapsed, scan_root) {
        eprintln!("flint: TUI failed ({e}); falling back to pretty output");
        print_pretty(results, elapsed);
    }

    summary
}

fn run(results: &[LintResult], elapsed: std::time::Duration, scan_root: &Path) -> Result<()> {
    let categories = group_results_for_display(results);
    let mut app = ResultsApp::new(categories, elapsed, scan_root.to_path_buf());
    let mut terminal = tui_common::TerminalSession::enter()?;

    loop {
        terminal
            .terminal
            .draw(|frame| render(frame, &app))
            .into_diagnostic()?;

        if !event::poll(Duration::from_millis(200)).into_diagnostic()? {
            continue;
        }

        match event::read().into_diagnostic()? {
            Event::Key(key) => {
                if app.handle_key(key) == AppAction::Quit {
                    break;
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
    Categories,
    Issues,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppAction {
    Continue,
    Quit,
}

struct ResultsApp<'a> {
    categories: Vec<(String, Vec<PrettyEntry<'a>>)>,
    elapsed: std::time::Duration,
    scan_root: PathBuf,
    selected_category: usize,
    selected_issue: usize,
    focus: FocusPane,
    search: String,
    search_mode: bool,
    errors_only: bool,
}

impl<'a> ResultsApp<'a> {
    fn new(
        categories: Vec<(String, Vec<PrettyEntry<'a>>)>,
        elapsed: std::time::Duration,
        scan_root: PathBuf,
    ) -> Self {
        let mut app = Self {
            categories,
            elapsed,
            scan_root,
            selected_category: 0,
            selected_issue: 0,
            focus: FocusPane::Categories,
            search: String::new(),
            search_mode: false,
            errors_only: false,
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
            KeyCode::Tab => {
                self.focus = match self.focus {
                    FocusPane::Categories => FocusPane::Issues,
                    FocusPane::Issues => FocusPane::Categories,
                };
                AppAction::Continue
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = FocusPane::Categories;
                AppAction::Continue
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.focus = FocusPane::Issues;
                AppAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.focus {
                    FocusPane::Categories => self.move_category(-1),
                    FocusPane::Issues => self.move_issue(-1),
                }
                AppAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.focus {
                    FocusPane::Categories => self.move_category(1),
                    FocusPane::Issues => self.move_issue(1),
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
                self.normalize_selection();
                AppAction::Continue
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
                self.normalize_selection();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.search.push(ch);
                self.selected_issue = 0;
                self.normalize_selection();
            }
            _ => {}
        }
    }

    fn move_category(&mut self, delta: isize) {
        let len = self.categories.len();
        if len == 0 {
            return;
        }
        let next = shift_index(self.selected_category, len, delta);
        if next != self.selected_category {
            self.selected_category = next;
            self.selected_issue = 0;
            self.normalize_selection();
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
    }

    fn jump_to_start(&mut self) {
        match self.focus {
            FocusPane::Categories => {
                self.selected_category = 0;
                self.selected_issue = 0;
            }
            FocusPane::Issues => self.selected_issue = 0,
        }
        self.normalize_selection();
    }

    fn jump_to_end(&mut self) {
        match self.focus {
            FocusPane::Categories => {
                if !self.categories.is_empty() {
                    self.selected_category = self.categories.len() - 1;
                    self.selected_issue = 0;
                }
            }
            FocusPane::Issues => {
                let len = self.visible_issues().len();
                if len > 0 {
                    self.selected_issue = len - 1;
                }
            }
        }
        self.normalize_selection();
    }

    fn reset_filters(&mut self) {
        self.selected_category = 0;
        self.selected_issue = 0;
        self.focus = FocusPane::Categories;
        self.search.clear();
        self.search_mode = false;
        self.errors_only = false;
        self.normalize_selection();
    }

    fn normalize_selection(&mut self) {
        if self.categories.is_empty() {
            self.selected_category = 0;
            self.selected_issue = 0;
            return;
        }

        if self.selected_category >= self.categories.len() {
            self.selected_category = self.categories.len() - 1;
        }

        let visible_len = self.visible_issues().len();
        if visible_len == 0 {
            self.selected_issue = 0;
        } else if self.selected_issue >= visible_len {
            self.selected_issue = visible_len - 1;
        }
    }

    fn selected_category_key(&self) -> Option<&str> {
        self.categories
            .get(self.selected_category)
            .map(|(key, _)| key.as_str())
    }

    fn visible_issues(&self) -> Vec<&PrettyEntry<'a>> {
        let Some((_, entries)) = self.categories.get(self.selected_category) else {
            return Vec::new();
        };

        let query = self.search.trim().to_ascii_lowercase();
        entries
            .iter()
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

    fn total_issues_in_category(&self) -> usize {
        self.categories
            .get(self.selected_category)
            .map(|(_, e)| e.len())
            .unwrap_or(0)
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

    fn display_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.scan_root)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

fn shift_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    let next = current as isize + delta;
    next.clamp(0, len as isize - 1) as usize
}

fn render(frame: &mut Frame, app: &ResultsApp<'_>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(chunks[1]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(body[1]);

    render_categories(frame, body[0], app);
    render_issues(frame, right[0], app);
    render_detail(frame, right[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &ResultsApp<'_>) {
    let (errors, warnings) = app.error_stats();
    let scan = app.scan_root.display();
    let stats_line = if errors == 0 && warnings == 0 {
        format!("{} errors, {} warnings — clean", errors, warnings)
    } else {
        format!("{errors} errors, {warnings} warnings")
    };

    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                "Flint Check",
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  finished in {:.2?}", app.elapsed),
                Style::default().fg(TEXT_MUTED),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!("Path: {scan}"), Style::default().fg(TEXT_MUTED)),
            Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
            Span::styled(stats_line, Style::default().fg(TEXT_MUTED)),
        ]),
    ]);

    let header = Paragraph::new(text)
        .block(tui_common::block("Results", false))
        .style(Style::default().bg(PANEL_BG));
    frame.render_widget(header, area);
}

fn render_categories(frame: &mut Frame, area: ratatui::layout::Rect, app: &ResultsApp<'_>) {
    if app.categories.is_empty() {
        let empty = Paragraph::new(Text::from(vec![Line::from(Span::styled(
            "No issues — nothing to browse.",
            Style::default().fg(TEXT_PRIMARY),
        ))]))
        .block(tui_common::block("Categories", app.focus == FocusPane::Categories))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .categories
        .iter()
        .map(|(key, entries)| {
            let title = category_display_name(key);
            let count = entries.len();
            ListItem::new(Line::from(vec![
                Span::styled(
                    title,
                    Style::default()
                        .fg(TEXT_PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(format!("{count}"), Style::default().fg(TEXT_MUTED)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(tui_common::block("Categories", app.focus == FocusPane::Categories))
        .highlight_style(
            Style::default()
                .bg(SELECT_BG)
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default();
    state.select(Some(app.selected_category));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_issues(frame: &mut Frame, area: ratatui::layout::Rect, app: &ResultsApp<'_>) {
    let visible = app.visible_issues();
    if visible.is_empty() {
        let msg = if app.total_issues_in_category() == 0 {
            "No issues in this category."
        } else {
            "No issues match filters — change search or press e to show warnings."
        };
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                msg,
                Style::default().fg(TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                "Use / to filter, e for errors-only, g to reset.",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Issues", app.focus == FocusPane::Issues))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = visible
        .iter()
        .map(|entry| {
            let (label, color) = tui_common::severity_badge(&entry.diagnostic.severity);
            let path = app.display_path(entry.file);
            let msg: String = entry
                .diagnostic
                .message
                .chars()
                .take(64)
                .collect();
            let ellipsis = if entry.diagnostic.message.chars().count() > 64 {
                "…"
            } else {
                ""
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {label} "),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{}:{} ", path, entry.diagnostic.span),
                    Style::default().fg(TEXT_MUTED),
                ),
                Span::styled(
                    format!("{msg}{ellipsis}"),
                    Style::default().fg(TEXT_PRIMARY),
                ),
                Span::styled(
                    format!("  [{}]", entry.diagnostic.rule_name),
                    Style::default().fg(TEXT_FAINT),
                ),
            ]))
        })
        .collect();

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
}

fn render_detail(frame: &mut Frame, area: ratatui::layout::Rect, app: &ResultsApp<'_>) {
    let Some(entry) = app.selected_entry() else {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Select an issue for full detail.",
                Style::default().fg(TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                "Rule id, help text, and optional fix metadata appear here.",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Detail", false))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    };

    let (severity_label, severity_color) = tui_common::severity_badge(&entry.diagnostic.severity);
    let group_title = app
        .selected_category_key()
        .map(category_display_name)
        .unwrap_or("Other");

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                format!(" {severity_label} "),
                Style::default()
                    .fg(severity_color)
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
        detail_line("File", &app.display_path(entry.file)),
        detail_line("Span", &entry.diagnostic.span),
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
            Style::default().fg(TEXT_MUTED).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(desc, Style::default().fg(TEXT_PRIMARY))));
        lines.push(Line::from(Span::styled(
            format!("Safety: {safety} · replacement {} chars", fix.replacement.len()),
            Style::default().fg(TEXT_FAINT),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Help",
        Style::default().fg(TEXT_MUTED).add_modifier(Modifier::BOLD),
    )));
    if let Some(help) = diagnostic_help_text(entry.diagnostic) {
        lines.push(Line::from(Span::styled(help, Style::default().fg(TEXT_PRIMARY))));
    } else {
        lines.push(Line::from(Span::styled(
            "—",
            Style::default().fg(TEXT_FAINT),
        )));
    }

    let panel = Paragraph::new(Text::from(lines))
        .block(tui_common::block("Detail", false))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
    frame.render_widget(panel, area);
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
        "type to filter · backspace edit · enter/esc close · q quit"
    } else {
        "j/k move · tab switch pane · / search · e errors-only · g reset · q quit"
    };

    let footer = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled(search_line, Style::default().fg(TEXT_PRIMARY)),
            Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
            Span::styled(format!("Filter: {filter}"), Style::default().fg(TEXT_MUTED)),
        ]),
        Line::from(Span::styled(legend, Style::default().fg(TEXT_MUTED))),
    ]))
    .block(tui_common::block("Keys", false))
    .wrap(Wrap { trim: false })
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
    fn category_change_resets_issue_index() {
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
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"));
        app.focus = FocusPane::Categories;
        app.move_category(1);
        app.focus = FocusPane::Issues;
        app.move_issue(1);
        assert_eq!(app.selected_issue, 1);
        app.focus = FocusPane::Categories;
        app.move_category(-1);
        assert_eq!(app.selected_issue, 0);
    }

    #[test]
    fn search_filters_issues() {
        let results = sample_results();
        let cats = group_results_for_display(&results);
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"));
        app.search = "console".to_string();
        let visible = app.visible_issues();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].diagnostic.rule_name, "no-console");
    }

    #[test]
    fn reset_clears_search_and_errors_only() {
        let results = sample_results();
        let cats = group_results_for_display(&results);
        let mut app = ResultsApp::new(cats, std::time::Duration::ZERO, PathBuf::from("/proj"));
        app.errors_only = true;
        app.search = "noop".to_string();
        app.reset_filters();
        assert!(!app.errors_only);
        assert!(app.search.is_empty());
    }
}
