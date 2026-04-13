use crate::cli::RulesArgs;
use crate::project::ProjectInfo;
use crate::rule_catalog::{build_rule_catalog, RuleCatalog, RuleCatalogEntry};
use crate::tui_common::{
    self, PANEL_BG, PANEL_BG_SUBTLE, SCROLLBAR_THUMB, SCROLLBAR_TRACK, SELECT_BG, TEXT_FAINT,
    TEXT_MUTED, TEXT_PRIMARY,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
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
            .draw(|frame| render(frame, &mut app))
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
    Details,
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
    detail_scroll: u16,
    rules_scrollbar: ScrollbarState,
    detail_scrollbar: ScrollbarState,
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
            detail_scroll: 0,
            rules_scrollbar: ScrollbarState::default(),
            detail_scrollbar: ScrollbarState::default(),
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
                    FocusPane::Rules => FocusPane::Details,
                    FocusPane::Details => FocusPane::Groups,
                };
                AppAction::Continue
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    FocusPane::Groups => FocusPane::Details,
                    FocusPane::Rules => FocusPane::Groups,
                    FocusPane::Details => FocusPane::Rules,
                };
                AppAction::Continue
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = FocusPane::Groups;
                AppAction::Continue
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.focus == FocusPane::Groups {
                    self.focus = FocusPane::Rules;
                }
                AppAction::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.focus {
                    FocusPane::Groups => self.move_group(-1),
                    FocusPane::Rules => self.move_rule(-1),
                    FocusPane::Details => self.scroll_detail(-1),
                }
                AppAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.focus {
                    FocusPane::Groups => self.move_group(1),
                    FocusPane::Rules => self.move_rule(1),
                    FocusPane::Details => self.scroll_detail(1),
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
                self.detail_scroll = 0;
                self.normalize_selection();
            }
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.search.push(ch);
                self.selected_rule = 0;
                self.detail_scroll = 0;
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
            self.detail_scroll = 0;
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
        self.detail_scroll = 0;
    }

    fn scroll_detail(&mut self, delta: isize) {
        let new = self.detail_scroll as isize + delta;
        self.detail_scroll = new.max(0) as u16;
    }

    fn jump_to_start(&mut self) {
        match self.focus {
            FocusPane::Groups => {
                self.selected_group = 0;
                self.selected_rule = 0;
            }
            FocusPane::Rules => self.selected_rule = 0,
            FocusPane::Details => self.detail_scroll = 0,
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
            FocusPane::Details => {
                self.detail_scroll = self.detail_scroll.saturating_add(100);
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
        self.detail_scroll = 0;
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

fn render(frame: &mut Frame, app: &mut RulesApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(chunks[1]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body[1]);

    render_groups(frame, body[0], app);
    render_rules(frame, right[0], app);
    render_details(frame, right[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, app: &RulesApp) {
    let project_label = if project_labels(&app.project).is_empty() {
        "none detected".to_string()
    } else {
        project_labels(&app.project).join(", ")
    };

    let line = Line::from(vec![
        Span::styled(
            " Flick Scan Rules",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} rules", app.catalog.entries.len()),
            Style::default().fg(TEXT_MUTED),
        ),
        Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
        Span::styled(
            format!("project: {project_label}"),
            Style::default().fg(TEXT_MUTED),
        ),
    ]);

    let header = Paragraph::new(Text::from(vec![line, Line::from("")]))
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

fn render_rules(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut RulesApp) {
    let entries = app.visible_entries();
    if entries.is_empty() {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "No rules match this filter.",
                Style::default().fg(TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                "Change the query or switch groups.",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Rules", app.focus == FocusPane::Rules))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    }

    let query = app.search.trim().to_string();
    let items = entries
        .iter()
        .map(|entry| {
            let (label, fg, bg) = tui_common::severity_badge(&entry.default_severity);
            let mut spans = vec![
                Span::styled(
                    format!(" {label} "),
                    Style::default()
                        .fg(fg)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ];
            spans.extend(highlight_spans(
                entry.id.as_str(),
                &query,
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
            spans.extend(highlight_spans(
                entry.summary.as_str(),
                &query,
                Style::default().fg(TEXT_MUTED),
            ));
            ListItem::new(Line::from(spans))
        })
        .collect::<Vec<_>>();

    let total = items.len();
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

    app.rules_scrollbar = app
        .rules_scrollbar
        .content_length(total)
        .position(app.selected_rule);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .track_style(Style::default().fg(SCROLLBAR_TRACK))
        .thumb_style(Style::default().fg(SCROLLBAR_THUMB));
    let scrollbar_area = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 0,
    });
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut app.rules_scrollbar);
}

fn render_details(frame: &mut Frame, area: ratatui::layout::Rect, app: &mut RulesApp) {
    let is_focused = app.focus == FocusPane::Details;

    let Some(entry) = app.selected_entry() else {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Pick a rule to inspect its details.",
                Style::default().fg(TEXT_PRIMARY),
            )),
            Line::from(Span::styled(
                "Detail pane shows rule info and config.",
                Style::default().fg(TEXT_MUTED),
            )),
        ]))
        .block(tui_common::block("Details", is_focused))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(PANEL_BG_SUBTLE));
        frame.render_widget(empty, area);
        return;
    };

    let (severity_label, severity_fg, severity_bg) =
        tui_common::severity_badge(&entry.default_severity);
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
        (
            "Inactive for current project",
            Style::default().fg(TEXT_MUTED),
        )
    };

    let lines = vec![
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
        Line::from(Span::styled(
            "Why",
            Style::default()
                .fg(TEXT_MUTED)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            entry.why.as_str(),
            Style::default().fg(TEXT_MUTED),
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
            Style::default()
                .fg(TEXT_MUTED)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            entry.config_snippet(),
            Style::default().fg(TEXT_PRIMARY),
        )),
        Line::from(Span::styled(
            entry.disable_snippet(),
            Style::default().fg(TEXT_FAINT),
        )),
    ];

    let total_lines = lines.len();
    let panel = Paragraph::new(Text::from(lines))
        .block(tui_common::block("Details", is_focused))
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

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &RulesApp) {
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
        "j/k move · tab pane · / search · g reset · q quit"
    };

    let footer = Paragraph::new(Text::from(vec![Line::from(vec![
        Span::styled(format!(" {search_line}"), Style::default().fg(TEXT_PRIMARY)),
        Span::styled("  •  ", Style::default().fg(TEXT_FAINT)),
        Span::styled(legend, Style::default().fg(TEXT_MUTED)),
    ])]))
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

    #[test]
    fn detail_scroll_does_not_go_negative() {
        let mut app = app_with("react-hooks", None);
        app.focus = FocusPane::Details;
        app.scroll_detail(-5);
        assert_eq!(app.detail_scroll, 0);
        app.scroll_detail(3);
        assert_eq!(app.detail_scroll, 3);
        app.scroll_detail(-1);
        assert_eq!(app.detail_scroll, 2);
    }

    #[test]
    fn tab_cycles_through_all_panes() {
        let mut app = app_with("react-hooks", None);
        assert_eq!(app.focus, FocusPane::Groups);
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, FocusPane::Rules);
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, FocusPane::Details);
        app.handle_key(KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, FocusPane::Groups);
    }
}
