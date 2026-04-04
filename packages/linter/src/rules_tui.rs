use crate::cli::RulesArgs;
use crate::project::ProjectInfo;
use crate::rule_catalog::{build_rule_catalog, RuleCatalog, RuleCatalogEntry};
use crate::tui_common::{self, PANEL_BG, PANEL_BG_SUBTLE, SELECT_BG, TEXT_FAINT, TEXT_MUTED, TEXT_PRIMARY};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use miette::{IntoDiagnostic, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::time::Duration;

pub fn run(args: &RulesArgs) -> Result<()> {
    let catalog = build_rule_catalog()?;
    let project = ProjectInfo::detect(std::path::Path::new("."));
    let mut app = RulesApp::new(
        catalog,
        project,
        args.group.as_deref(),
        args.search.as_deref(),
    );
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
    Groups,
    Rules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppAction {
    Continue,
    Quit,
}

#[derive(Debug, Clone)]
struct RulesApp {
    catalog: RuleCatalog,
    project: ProjectInfo,
    selected_group: usize,
    selected_rule: usize,
    focus: FocusPane,
    search: String,
    search_mode: bool,
}

impl RulesApp {
    fn new(
        catalog: RuleCatalog,
        project: ProjectInfo,
        initial_group: Option<&str>,
        initial_search: Option<&str>,
    ) -> Self {
        let mut app = Self {
            selected_group: initial_group
                .and_then(|group| catalog.group_index(group))
                .unwrap_or(0),
            selected_rule: 0,
            focus: FocusPane::Groups,
            search: initial_search.unwrap_or_default().to_string(),
            search_mode: false,
            catalog,
            project,
        };
        if !app.search.is_empty() && app.visible_entries().is_empty() {
            if let Some(index) = app.first_group_with_matches() {
                app.selected_group = index;
            }
        }
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
                    FocusPane::Groups => FocusPane::Rules,
                    FocusPane::Rules => FocusPane::Groups,
                };
                AppAction::Continue
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = FocusPane::Groups;
                AppAction::Continue
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.focus = FocusPane::Rules;
                AppAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.focus {
                    FocusPane::Groups => self.move_group(-1),
                    FocusPane::Rules => self.move_rule(-1),
                }
                AppAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.focus {
                    FocusPane::Groups => self.move_group(1),
                    FocusPane::Rules => self.move_rule(1),
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
                self.reset();
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
                self.selected_rule = 0;
                self.normalize_selection();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.search.push(ch);
                self.selected_rule = 0;
                self.normalize_selection();
            }
            _ => {}
        }
    }

    fn move_group(&mut self, delta: isize) {
        let next = shift_index(self.selected_group, self.catalog.groups.len(), delta);
        if next != self.selected_group {
            self.selected_group = next;
            self.selected_rule = 0;
            self.normalize_selection();
        }
    }

    fn move_rule(&mut self, delta: isize) {
        let len = self.visible_entries().len();
        if len == 0 {
            self.selected_rule = 0;
            return;
        }
        self.selected_rule = shift_index(self.selected_rule, len, delta);
    }

    fn jump_to_start(&mut self) {
        match self.focus {
            FocusPane::Groups => {
                self.selected_group = 0;
                self.selected_rule = 0;
            }
            FocusPane::Rules => self.selected_rule = 0,
        }
        self.normalize_selection();
    }

    fn jump_to_end(&mut self) {
        match self.focus {
            FocusPane::Groups => {
                if !self.catalog.groups.is_empty() {
                    self.selected_group = self.catalog.groups.len() - 1;
                    self.selected_rule = 0;
                }
            }
            FocusPane::Rules => {
                let len = self.visible_entries().len();
                if len > 0 {
                    self.selected_rule = len - 1;
                }
            }
        }
        self.normalize_selection();
    }

    fn reset(&mut self) {
        self.selected_group = 0;
        self.selected_rule = 0;
        self.focus = FocusPane::Groups;
        self.search.clear();
        self.search_mode = false;
        self.normalize_selection();
    }

    fn normalize_selection(&mut self) {
        if self.catalog.groups.is_empty() {
            self.selected_group = 0;
            self.selected_rule = 0;
            return;
        }

        if self.selected_group >= self.catalog.groups.len() {
            self.selected_group = self.catalog.groups.len() - 1;
        }

        let visible = self.visible_entries();
        if visible.is_empty() {
            self.selected_rule = 0;
        } else if self.selected_rule >= visible.len() {
            self.selected_rule = visible.len() - 1;
        }
    }

    fn first_group_with_matches(&self) -> Option<usize> {
        (0..self.catalog.groups.len()).find(|index| self.group_count(*index) > 0)
    }

    fn group_count(&self, index: usize) -> usize {
        self.entries_for_group(index).len()
    }

    fn visible_entries(&self) -> Vec<&RuleCatalogEntry> {
        self.entries_for_group(self.selected_group)
    }

    fn selected_entry(&self) -> Option<&RuleCatalogEntry> {
        let entries = self.visible_entries();
        entries.get(self.selected_rule).copied()
    }

    fn entries_for_group(&self, index: usize) -> Vec<&RuleCatalogEntry> {
        let Some(group) = self.catalog.groups.get(index) else {
            return Vec::new();
        };

        let query = self.search.trim().to_ascii_lowercase();
        let mut entries = self
            .catalog
            .entries
            .iter()
            .filter(|entry| entry.group_key == group.key)
            .filter(|entry| {
                if query.is_empty() {
                    return true;
                }
                let id = entry.id.to_ascii_lowercase();
                let summary = entry.summary.to_ascii_lowercase();
                id.contains(&query) || summary.contains(&query)
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            tui_common::severity_rank(&left.default_severity)
                .cmp(&tui_common::severity_rank(&right.default_severity))
                .then_with(|| left.id.cmp(&right.id))
        });
        entries
    }
}

fn render(frame: &mut Frame, app: &RulesApp) {
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

    render_groups(frame, body[0], app);
    render_rules(frame, right[0], app);
    render_details(frame, right[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &RulesApp) {
    let project_line = if project_labels(&app.project).is_empty() {
        "Current project: none detected".to_string()
    } else {
        format!(
            "Current project: {}",
            project_labels(&app.project).join(", ")
        )
    };
    let search_line = if app.search.is_empty() {
        "Search: none".to_string()
    } else {
        format!("Search: {}", app.search)
    };

    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(
                "Flint Rules",
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {} rules", app.catalog.entries.len()),
                Style::default().fg(TEXT_MUTED),
            ),
        ]),
        Line::from(vec![
            Span::styled(project_line, Style::default().fg(TEXT_MUTED)),
            Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
            Span::styled(search_line, Style::default().fg(TEXT_MUTED)),
        ]),
    ]);

    let header = Paragraph::new(text)
        .block(tui_common::block("Browser", false))
        .style(Style::default().bg(PANEL_BG));
    frame.render_widget(header, area);
}

fn render_groups(frame: &mut Frame, area: ratatui::layout::Rect, app: &RulesApp) {
    let items = app
        .catalog
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            let count = app.group_count(index);
            let title_color = if count == 0 { TEXT_FAINT } else { TEXT_PRIMARY };
            ListItem::new(Line::from(vec![
                Span::styled(
                    group.title,
                    Style::default()
                        .fg(title_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(format!("{count}"), Style::default().fg(TEXT_MUTED)),
            ]))
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(tui_common::block("Groups", app.focus == FocusPane::Groups))
        .highlight_style(
            Style::default()
                .bg(SELECT_BG)
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default();
    state.select(Some(app.selected_group));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_rules(frame: &mut Frame, area: ratatui::layout::Rect, app: &RulesApp) {
    let entries = app.visible_entries();
    if entries.is_empty() {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "No rules match this filter.",
                Style::default().fg(TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                "Change the query or switch groups to keep browsing.",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Rules", app.focus == FocusPane::Rules))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    }

    let items = entries
        .iter()
        .map(|entry| {
            let (label, color) = tui_common::severity_badge(&entry.default_severity);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {label} "),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    entry.id.as_str(),
                    Style::default()
                        .fg(TEXT_PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(entry.summary.as_str(), Style::default().fg(TEXT_MUTED)),
            ]))
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(tui_common::block("Rules", app.focus == FocusPane::Rules))
        .highlight_style(
            Style::default()
                .bg(SELECT_BG)
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default();
    state.select(Some(app.selected_rule));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_details(frame: &mut Frame, area: ratatui::layout::Rect, app: &RulesApp) {
    let Some(entry) = app.selected_entry() else {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Pick a rule to inspect its details.",
                Style::default().fg(TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                "The detail pane stays useful even when a group is empty.",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Details", false))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    };

    let (severity_label, severity_color) = tui_common::severity_badge(&entry.default_severity);
    let applies = entry.scope.applies_to_project(&app.project);
    let group_title = app
        .catalog
        .groups
        .iter()
        .find(|group| group.key == entry.group_key)
        .map(|group| group.title)
        .unwrap_or(entry.group_key);
    let applicability = if applies {
        (
            "Active in current project",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        ("Inactive for current project", Style::default().fg(TEXT_MUTED))
    };

    let details = Text::from(vec![
        Line::from(vec![
            Span::styled(
                format!(" {severity_label} "),
                Style::default()
                    .fg(severity_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                entry.id.as_str(),
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            entry.summary.as_str(),
            Style::default().fg(TEXT_PRIMARY),
        )),
        Line::from(""),
        detail_line("Group", group_title),
        detail_line("Applies To", entry.scope.label()),
        Line::from(vec![
            Span::styled("Current Project: ", Style::default().fg(TEXT_MUTED)),
            Span::styled(applicability.0, applicability.1),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Config",
            Style::default().fg(TEXT_MUTED).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            entry.config_snippet(),
            Style::default().fg(TEXT_PRIMARY),
        )),
        Line::from(Span::styled(
            entry.disable_snippet(),
            Style::default().fg(TEXT_FAINT),
        )),
    ]);

    let panel = Paragraph::new(details)
        .block(tui_common::block("Details", false))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
    frame.render_widget(panel, area);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &RulesApp) {
    let search_line = if app.search_mode {
        format!("Search: {}_", app.search)
    } else if app.search.is_empty() {
        "Search: /".to_string()
    } else {
        format!("Search: {}", app.search)
    };
    let legend = if app.search_mode {
        "type to filter • backspace edit • enter/esc close • q quit"
    } else {
        "j/k move • tab switch pane • / search • g reset • q quit"
    };

    let footer = Paragraph::new(Text::from(vec![
        Line::from(Span::styled(search_line, Style::default().fg(TEXT_PRIMARY))),
        Line::from(Span::styled(legend, Style::default().fg(TEXT_MUTED))),
    ]))
    .block(tui_common::block("Keys", false))
    .wrap(Wrap { trim: false })
    .style(Style::default().bg(PANEL_BG));
    frame.render_widget(footer, area);
}

fn detail_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), Style::default().fg(TEXT_MUTED)),
        Span::styled(value.to_string(), Style::default().fg(TEXT_PRIMARY)),
    ])
}

fn shift_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }
    let next = current as isize + delta;
    next.clamp(0, len as isize - 1) as usize
}

fn project_labels(project: &ProjectInfo) -> Vec<&'static str> {
    let mut labels = Vec::new();
    if project.has_next {
        labels.push("nextjs");
    } else if project.has_react {
        labels.push("react");
    }
    if project.has_server_framework() {
        labels.push("server");
    }
    if project.has_react_native {
        labels.push("react-native");
    }
    if project.has_expo {
        labels.push("expo");
    }
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with(group: &str, search: Option<&str>) -> RulesApp {
        RulesApp::new(
            build_rule_catalog().unwrap(),
            ProjectInfo::test_all(),
            Some(group),
            search,
        )
    }

    #[test]
    fn category_navigation_resets_rule_selection() {
        let mut app = app_with("react-hooks", None);
        app.focus = FocusPane::Rules;
        app.move_rule(3);
        assert_eq!(app.selected_rule, 3);
        app.focus = FocusPane::Groups;
        app.move_group(1);
        assert_eq!(app.selected_rule, 0);
        assert_eq!(
            app.catalog.groups[app.selected_group].key,
            "react-correctness"
        );
    }

    #[test]
    fn search_filters_rules_in_selected_group() {
        let app = app_with("react-hooks", Some("fetch"));
        let ids = app
            .visible_entries()
            .into_iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["react/no-fetch-in-effect"]);
    }

    #[test]
    fn unmatched_search_has_no_selected_rule() {
        let app = app_with("react-hooks", Some("definitely-missing"));
        assert!(app.visible_entries().is_empty());
        assert!(app.selected_entry().is_none());
    }

    #[test]
    fn reset_clears_search_and_returns_to_top() {
        let mut app = app_with("server-security", Some("sql"));
        app.focus = FocusPane::Rules;
        app.selected_rule = 1;
        app.reset();
        assert_eq!(app.selected_group, 0);
        assert_eq!(app.selected_rule, 0);
        assert_eq!(app.focus, FocusPane::Groups);
        assert!(app.search.is_empty());
    }
}
