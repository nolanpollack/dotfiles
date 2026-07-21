use std::collections::HashSet;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::create::directory::{Candidate, DirectoryForm};
use crate::create::form::Field;
use crate::create::CreateFlow;
use crate::picker::{Picker, View};
use crate::sessions::SessionInfo;

use super::Theme;

pub fn draw_ui(
    frame: &mut Frame,
    view: &View<SessionInfo>,
    hints: &[(&str, &str)],
    renaming: Option<&str>,
    theme: &Theme,
) {
    let [search_area, sep_area, list_area, status_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_search(frame, search_area, view.query, theme);
    draw_separator(frame, sep_area, theme);
    draw_list(frame, list_area, view, renaming, theme);
    draw_status(frame, status_area, view, hints, theme);
}

fn draw_search(frame: &mut Frame, area: Rect, query: &str, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("> {query}"), Style::default().fg(theme.query_fg)),
            Span::styled("▏", Style::default().fg(theme.query_fg).add_modifier(Modifier::BOLD)),
        ])),
        area,
    );
}

fn draw_separator(frame: &mut Frame, area: Rect, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(Line::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(theme.separator_fg),
        )),
        area,
    );
}

fn draw_list(frame: &mut Frame, area: Rect, view: &View<SessionInfo>, renaming: Option<&str>, theme: &Theme) {
    let list_items: Vec<ListItem> = view
        .items
        .iter()
        .enumerate()
        .map(|(idx, (session, hit_indices))| match (session.is_current, renaming) {
            (true, Some(draft)) => rename_list_item(draft, theme),
            _ => {
                // A worktree sibling is the last in its run if the next visible item isn't also
                // one — groups are always contiguous, so this holds even under filtering.
                let is_last_sibling = session.nested_worktree
                    && view.items.get(idx + 1).map_or(true, |(next, _)| !next.nested_worktree);
                session_list_item(session, hit_indices, is_last_sibling, theme)
            }
        })
        .collect();

    let mut list_state = ListState::default().with_selected(view.selected);
    frame.render_stateful_widget(
        List::new(list_items)
            .highlight_style(Style::default().bg(theme.list_selected_bg))
            .highlight_symbol("> "),
        area,
        &mut list_state,
    );
}

fn session_list_item<'a>(
    session: &'a SessionInfo,
    hit_indices: &[usize],
    is_last_sibling: bool,
    theme: &Theme,
) -> ListItem<'a> {
    if !session.is_active {
        return ListItem::new(Line::from(vec![
            Span::styled("↺ ", Style::default().fg(theme.list_inactive_fg)),
            Span::styled(session.name.as_str(), Style::default().fg(theme.list_inactive_fg)),
            Span::styled("  inactive", Style::default().fg(theme.list_inactive_fg)),
        ]));
    }

    let hit_set: HashSet<usize> = hit_indices.iter().copied().collect();
    let mut spans: Vec<Span> = Vec::new();
    if session.nested_worktree {
        let glyph = if is_last_sibling { "└─ " } else { "├─ " };
        spans.push(Span::styled(format!("  {glyph}"), Style::default().fg(theme.list_inactive_fg)));
    }
    spans.extend(session.name.chars().enumerate().map(|(i, c)| {
        if hit_set.contains(&i) {
            Span::styled(
                c.to_string(),
                Style::default().fg(theme.list_match_fg).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(c.to_string(), Style::default().fg(theme.list_normal_fg))
        }
    }));
    if let Some(branch) = &session.branch {
        spans.push(Span::styled(
            format!(" \u{f126} {branch}"),
            Style::default().fg(theme.list_inactive_fg),
        ));
    }
    if session.is_current {
        spans.push(Span::styled(" ●", Style::default().fg(theme.list_current_marker_fg)));
    }
    ListItem::new(Line::from(spans))
}

fn rename_list_item(draft: &str, theme: &Theme) -> ListItem<'static> {
    ListItem::new(Line::from(vec![
        Span::styled(draft.to_string(), Style::default().fg(theme.query_fg)),
        Span::styled("▏", Style::default().fg(theme.query_fg).add_modifier(Modifier::BOLD)),
    ]))
}

fn draw_status(
    frame: &mut Frame,
    area: Rect,
    view: &View<SessionInfo>,
    hints: &[(&str, &str)],
    theme: &Theme,
) {
    let count_str = format!("{}/{}", view.filtered_count, view.total_count);
    let [hints_area, count_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(count_str.len() as u16)])
            .areas(area);

    frame.render_widget(Paragraph::new(Line::from(hint_spans(hints, theme))), hints_area);
    frame.render_widget(
        Paragraph::new(count_str).style(Style::default().fg(theme.status_count_fg)),
        count_area,
    );
}

pub fn draw_create_ui(frame: &mut Frame, flow: &CreateFlow, theme: &Theme) {
    match flow {
        CreateFlow::Directory(form) => draw_directory_form(frame, form, theme),
    }
}

fn draw_directory_form(frame: &mut Frame, form: &DirectoryForm, theme: &Theme) {
    // If the focused field's combobox is expanded, it takes over the whole screen with a
    // search+list view — same interaction model as the main picker's own list.
    if let Some(Field::Combobox(cb)) = form.form().fields().get(form.form().focus()) {
        if let Some(picker) = cb.picker() {
            draw_combobox_expanded(frame, picker, theme);
            return;
        }
    }

    let field_count = form.form().fields().len() as u16;
    let [fields_area, error_area, hint_area] = Layout::vertical([
        Constraint::Length(field_count),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let field_areas =
        Layout::vertical(vec![Constraint::Length(1); form.form().fields().len()]).split(fields_area);
    const LABELS: [&str; 2] = ["Directory", "Name"];
    for (i, field) in form.form().fields().iter().enumerate() {
        let focused = i == form.form().focus();
        draw_field_row(frame, field_areas[i], LABELS.get(i).copied().unwrap_or(""), field, focused, theme);
    }

    if let Some(err) = form.error() {
        frame.render_widget(Paragraph::new(Line::styled(err, Style::default().fg(theme.error_fg))), error_area);
    }

    let hints = [("tab", "next field"), ("enter", "open/submit"), ("esc", "back")];
    frame.render_widget(Paragraph::new(Line::from(hint_spans(&hints, theme))), hint_area);
}

fn draw_field_row(frame: &mut Frame, area: Rect, label: &str, field: &Field<Candidate>, focused: bool, theme: &Theme) {
    let value = match field {
        Field::Combobox(cb) => cb.display(),
        Field::Text(t) => t.value.clone(),
    };
    let label_fg = if focused { theme.hint_key_fg } else { theme.list_inactive_fg };
    let mut spans = vec![
        Span::styled(format!("{label}: "), Style::default().fg(label_fg).add_modifier(Modifier::BOLD)),
        Span::styled(value, Style::default().fg(theme.list_normal_fg)),
    ];
    if focused {
        spans.push(Span::styled("▏", Style::default().fg(theme.query_fg).add_modifier(Modifier::BOLD)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_combobox_expanded(frame: &mut Frame, picker: &Picker<Candidate>, theme: &Theme) {
    let view = picker.view();
    let [search_area, sep_area, list_area, status_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_search(frame, search_area, view.query, theme);
    draw_separator(frame, sep_area, theme);

    let items: Vec<ListItem> =
        view.items.iter().map(|(candidate, hit_indices)| candidate_list_item(candidate, hit_indices, theme)).collect();
    let mut list_state = ListState::default().with_selected(view.selected);
    frame.render_stateful_widget(
        List::new(items).highlight_style(Style::default().bg(theme.list_selected_bg)).highlight_symbol("> "),
        list_area,
        &mut list_state,
    );

    frame.render_widget(
        Paragraph::new(format!("{}/{}", view.filtered_count, view.total_count))
            .style(Style::default().fg(theme.status_count_fg))
            .alignment(Alignment::Right),
        status_area,
    );
}

fn candidate_list_item<'a>(candidate: &'a Candidate, hit_indices: &[usize], theme: &Theme) -> ListItem<'a> {
    let hit_set: HashSet<usize> = hit_indices.iter().copied().collect();
    let spans: Vec<Span> = candidate
        .display
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if hit_set.contains(&i) {
                Span::styled(c.to_string(), Style::default().fg(theme.list_match_fg).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(c.to_string(), Style::default().fg(theme.list_normal_fg))
            }
        })
        .collect();
    ListItem::new(Line::from(spans))
}

fn hint_spans<'a>(hints: &'a [(&'a str, &'a str)], theme: &Theme) -> Vec<Span<'a>> {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            *key,
            Style::default().fg(theme.hint_key_fg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {desc}"), Style::default().fg(theme.hint_desc_fg)));
    }
    spans
}
