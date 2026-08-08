use std::collections::HashSet;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::components;
use crate::ui::model::{ChoiceRow, CreateView};
use crate::ui::Theme;

pub fn draw(frame: &mut Frame, view: &CreateView, theme: &Theme) {
    match view {
        CreateView::Form {
            directory,
            name,
            directory_focused,
            error,
        } => draw_form(
            frame,
            directory,
            name,
            *directory_focused,
            error.as_deref(),
            theme,
        ),
        CreateView::DirectoryChoices {
            query,
            rows,
            selected,
            filtered_count,
            total_count,
        } => draw_choices(
            frame,
            query,
            rows,
            *selected,
            *filtered_count,
            *total_count,
            theme,
        ),
        CreateView::WorktreeForm {
            session_name,
            repository,
            base_branch,
            branch_name,
            focused,
            error,
        } => draw_worktree_form(
            frame,
            session_name,
            repository,
            base_branch,
            branch_name,
            *focused,
            error.as_deref(),
            theme,
        ),
        CreateView::WorktreeProgress {
            stage,
            error,
            spinner_tick,
        } => draw_worktree_progress(frame, *stage, error.as_deref(), *spinner_tick, theme),
    }
}

fn draw_worktree_form(
    frame: &mut Frame,
    session_name: &str,
    repository: &str,
    base_branch: &str,
    branch_name: &str,
    focused: usize,
    error: Option<&str>,
    theme: &Theme,
) {
    let [fields, error_area, hints] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(fields);
    field(
        frame,
        rows[0],
        "Session name",
        session_name,
        focused == 0,
        theme,
    );
    field(
        frame,
        rows[1],
        "Repository",
        repository,
        focused == 1,
        theme,
    );
    field(
        frame,
        rows[2],
        "Branch from",
        base_branch,
        focused == 2,
        theme,
    );
    field(
        frame,
        rows[3],
        "Branch name",
        branch_name,
        focused == 3,
        theme,
    );
    if let Some(error) = error {
        frame.render_widget(
            Paragraph::new(Line::styled(error, Style::default().fg(theme.error_fg))),
            error_area,
        );
    }
    frame.render_widget(
        Paragraph::new(Line::from(components::hint_spans(
            &[
                ("↑/↓ ctrl-j/k", "field"),
                ("enter", "next/create"),
                ("esc", "cancel"),
            ],
            theme,
        ))),
        hints,
    );
}

fn draw_worktree_progress(
    frame: &mut Frame,
    stage: crate::create::worktree::Stage,
    error: Option<&str>,
    spinner_tick: usize,
    theme: &Theme,
) {
    const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let [steps, error_area, hints] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let active = match stage {
        crate::create::worktree::Stage::Checking => 0,
        crate::create::worktree::Stage::Creating => 1,
        crate::create::worktree::Stage::Failed => 0,
    };
    let labels = [
        "Check repository, branch, and destination",
        "Create worktree and branch",
        "Start Zellij session",
        "Switch to the new session",
    ];
    let lines = labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let (glyph, glyph_style) = if index < active {
                ("✓", Style::default().fg(theme.agent_done_fg))
            } else if index == active && stage != crate::create::worktree::Stage::Failed {
                (
                    SPINNER[spinner_tick % SPINNER.len()],
                    Style::default().fg(theme.agent_working_fg),
                )
            } else {
                ("○", Style::default().fg(theme.list_inactive_fg))
            };
            let text_style = if index <= active && stage != crate::create::worktree::Stage::Failed {
                Style::default().fg(theme.list_normal_fg)
            } else {
                Style::default().fg(theme.list_inactive_fg)
            };
            Line::from(vec![
                Span::styled(format!("{glyph}  "), glyph_style),
                Span::styled(*label, text_style),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines), steps);
    if let Some(error) = error {
        frame.render_widget(
            Paragraph::new(Line::styled(error, Style::default().fg(theme.error_fg))),
            error_area,
        );
    }
    if stage == crate::create::worktree::Stage::Failed {
        frame.render_widget(
            Paragraph::new(Line::from(components::hint_spans(
                &[("esc", "return to form")],
                theme,
            ))),
            hints,
        );
    }
}

fn draw_form(
    frame: &mut Frame,
    directory: &str,
    name: &str,
    directory_focused: bool,
    error: Option<&str>,
    theme: &Theme,
) {
    let [fields, error_area, hints] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(fields);
    field(
        frame,
        rows[0],
        "Directory",
        directory,
        directory_focused,
        theme,
    );
    field(frame, rows[1], "Name", name, !directory_focused, theme);
    if let Some(error) = error {
        frame.render_widget(
            Paragraph::new(Line::styled(error, Style::default().fg(theme.error_fg))),
            error_area,
        );
    }
    frame.render_widget(
        Paragraph::new(Line::from(components::hint_spans(
            &[
                ("tab", "next field"),
                ("enter", "open/submit"),
                ("esc", "back"),
            ],
            theme,
        ))),
        hints,
    );
}

fn field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool, theme: &Theme) {
    let color = if focused {
        theme.hint_key_fg
    } else {
        theme.list_inactive_fg
    };
    let mut spans = vec![
        Span::styled(
            format!("{label}: "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(theme.list_normal_fg)),
    ];
    if focused {
        spans.push(Span::styled(
            "▏",
            Style::default()
                .fg(theme.query_fg)
                .add_modifier(Modifier::BOLD),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_choices(
    frame: &mut Frame,
    query: &str,
    rows: &[ChoiceRow],
    selected: Option<usize>,
    filtered_count: usize,
    total_count: usize,
    theme: &Theme,
) {
    let [search, separator, list, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    components::search(frame, search, query, theme);
    components::separator(frame, separator, theme);
    let items = rows
        .iter()
        .map(|row| choice_item(row, theme))
        .collect::<Vec<_>>();
    let mut state = ListState::default().with_selected(selected);
    frame.render_stateful_widget(
        List::new(items)
            .highlight_style(Style::default().bg(theme.list_selected_bg))
            .highlight_symbol("> "),
        list,
        &mut state,
    );
    frame.render_widget(
        Paragraph::new(format!("{filtered_count}/{total_count}"))
            .style(Style::default().fg(theme.status_count_fg))
            .alignment(Alignment::Right),
        status,
    );
}

fn choice_item<'a>(row: &'a ChoiceRow, theme: &Theme) -> ListItem<'a> {
    let matched: HashSet<_> = row.matched.iter().copied().collect();
    let spans = row
        .display
        .chars()
        .enumerate()
        .map(|(index, character)| {
            let style = if matched.contains(&index) {
                Style::default()
                    .fg(theme.list_match_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.list_normal_fg)
            };
            Span::styled(character.to_string(), style)
        })
        .collect::<Vec<_>>();
    ListItem::new(Line::from(spans))
}
