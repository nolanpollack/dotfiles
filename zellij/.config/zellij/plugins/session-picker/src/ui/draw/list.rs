use std::collections::HashSet;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::components;
use crate::agent_refresh::RefreshView;
use crate::picker_refresh::RefreshView as PickerRefreshView;
use crate::ui::model::{AgentRow, AgentState, Focus, ListView, SessionRow};
use crate::ui::Theme;

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw(frame: &mut Frame, view: &ListView, theme: &Theme) {
    let [search, separator, columns, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    components::search(frame, search, &view.query, theme);
    components::separator(frame, separator, theme);

    if is_wide(columns.width) {
        let [sessions, divider, agents] = Layout::horizontal([
            Constraint::Percentage(55),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(columns);
        draw_sessions(frame, sessions, view, view.focus == Focus::Sessions, theme);
        draw_divider(frame, divider, theme);
        draw_agents(frame, agents, view, view.focus == Focus::Agents, theme);
    } else if view.focus == Focus::Agents && !view.agents.is_empty() {
        draw_agents(frame, columns, view, true, theme);
    } else {
        draw_sessions(frame, columns, view, true, theme);
    }
    draw_status(frame, status, view, theme);
}

fn is_wide(width: u16) -> bool {
    let sessions = width.saturating_mul(55) / 100;
    sessions >= 32 && width.saturating_sub(sessions + 1) >= 34
}

fn draw_sessions(frame: &mut Frame, area: Rect, view: &ListView, focused: bool, theme: &Theme) {
    let gutter = view
        .sessions
        .iter()
        .any(|session| session.agent.is_some())
        .then_some(1)
        .unwrap_or_default();
    let items: Vec<_> = view
        .sessions
        .iter()
        .map(|row| session_item(row, gutter, view.spinner_tick, theme))
        .collect();
    let mut state = ListState::default().with_selected(view.selected_session);
    frame.render_stateful_widget(
        List::new(items)
            .highlight_style(if focused {
                Style::default().bg(theme.list_selected_bg)
            } else {
                Style::default()
            })
            .highlight_symbol(if focused { "> " } else { "  " }),
        area,
        &mut state,
    );
}

fn session_item<'a>(
    row: &'a SessionRow,
    gutter: usize,
    tick: usize,
    theme: &Theme,
) -> ListItem<'a> {
    if let Some(draft) = &row.rename_draft {
        return ListItem::new(Line::from(vec![
            Span::styled(draft, Style::default().fg(theme.query_fg)),
            Span::styled(
                "▏",
                Style::default()
                    .fg(theme.query_fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    let mut spans = gutter_spans(row.agent, gutter, tick, theme);
    if !row.active {
        spans.extend([
            Span::styled("↺ ", Style::default().fg(theme.list_inactive_fg)),
            Span::styled(&row.name, Style::default().fg(theme.list_inactive_fg)),
            Span::styled("  inactive", Style::default().fg(theme.list_inactive_fg)),
        ]);
        return ListItem::new(Line::from(spans));
    }
    if row.nested {
        let glyph = if row.last_sibling {
            "└─ "
        } else {
            "├─ "
        };
        spans.push(Span::styled(
            format!("  {glyph}"),
            Style::default().fg(theme.list_inactive_fg),
        ));
    }
    let matched: HashSet<_> = row.matched.iter().copied().collect();
    spans.extend(row.name.chars().enumerate().map(|(index, character)| {
        let style = if matched.contains(&index) {
            Style::default()
                .fg(theme.list_match_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.list_normal_fg)
        };
        Span::styled(character.to_string(), style)
    }));
    if let Some(branch) = &row.branch {
        spans.push(Span::styled(
            format!(" \u{f126} {branch}"),
            Style::default().fg(theme.list_inactive_fg),
        ));
    }
    if row.current {
        spans.push(Span::styled(
            " ●",
            Style::default().fg(theme.list_current_marker_fg),
        ));
    }
    ListItem::new(Line::from(spans))
}

fn gutter_spans(
    state: Option<AgentState>,
    width: usize,
    tick: usize,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    if let Some(state) = state {
        let (glyph, color) = state_glyph(state, tick, theme);
        spans.push(Span::styled(glyph, Style::default().fg(color)));
        spans.push(Span::raw(" "));
    }
    for _ in usize::from(state.is_some())..width {
        spans.push(Span::raw("  "));
    }
    spans
}

fn state_glyph(state: AgentState, tick: usize, theme: &Theme) -> (&'static str, Color) {
    match state {
        AgentState::Blocked => ("●", theme.agent_blocked_fg),
        AgentState::Working => (spinner(tick), theme.agent_working_fg),
        AgentState::Done => ("✓", theme.agent_done_fg),
        AgentState::Idle => ("○", theme.agent_idle_fg),
        AgentState::Unknown => ("·", theme.agent_unknown_fg),
    }
}

fn spinner(tick: usize) -> &'static str {
    SPINNER[tick % SPINNER.len()]
}

fn draw_agents(frame: &mut Frame, area: Rect, view: &ListView, focused: bool, theme: &Theme) {
    if view.agents.is_empty() {
        let message = match view.agent_refresh {
            RefreshView::Loading | RefreshView::Refreshing { cached: false } => {
                "Loading agent status…"
            }
            RefreshView::Failed { cached: false } => "Agent status unavailable",
            _ => "No active Codex or Claude agents",
        };
        frame.render_widget(
            Paragraph::new(message).style(Style::default().fg(theme.list_inactive_fg)),
            area,
        );
        return;
    }
    let mut items = Vec::new();
    let mut selected_item = None;
    let mut last_session: Option<&str> = None;
    for (index, agent) in view.agents.iter().enumerate() {
        if last_session != Some(&agent.session_name) {
            items.push(ListItem::new(Line::styled(
                agent.session_name.clone(),
                Style::default()
                    .fg(theme.list_normal_fg)
                    .add_modifier(Modifier::BOLD),
            )));
            last_session = Some(&agent.session_name);
        }
        if view.selected_agent == Some(index) {
            selected_item = Some(items.len());
        }
        items.push(agent_item(agent, view.spinner_tick, theme));
    }
    let mut state = ListState::default().with_selected(focused.then_some(selected_item).flatten());
    frame.render_stateful_widget(
        List::new(items).highlight_style(if focused {
            Style::default().bg(theme.list_selected_bg)
        } else {
            Style::default()
        }),
        area,
        &mut state,
    );
}

fn agent_item<'a>(agent: &'a AgentRow, tick: usize, theme: &Theme) -> ListItem<'a> {
    let (glyph, color) = state_glyph(agent.state, tick, theme);
    ListItem::new(vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled(glyph, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(&agent.label, Style::default().fg(theme.list_normal_fg)),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(&agent.preview, Style::default().fg(theme.hint_desc_fg)),
        ]),
    ])
}

fn draw_divider(frame: &mut Frame, area: Rect, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(
            std::iter::repeat_n("│", area.height as usize)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .style(Style::default().fg(theme.panel_divider_fg)),
        area,
    );
}

fn draw_status(frame: &mut Frame, area: Rect, view: &ListView, theme: &Theme) {
    let count = format!("{}/{}", view.filtered_count, view.total_count);
    let refresh = match view.refresh {
        PickerRefreshView::Refreshing => format!("{} refreshing  ", spinner(view.spinner_tick)),
        PickerRefreshView::Failed => "refresh incomplete  ".to_string(),
        _ => String::new(),
    };
    let [hints, refresh_area, count_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(refresh.chars().count() as u16),
        Constraint::Length(count.len() as u16),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Line::from(components::hint_spans(&view.hints, theme))),
        hints,
    );
    frame.render_widget(
        Paragraph::new(refresh).style(Style::default().fg(theme.agent_working_fg)),
        refresh_area,
    );
    frame.render_widget(
        Paragraph::new(count).style(Style::default().fg(theme.status_count_fg)),
        count_area,
    );
}
