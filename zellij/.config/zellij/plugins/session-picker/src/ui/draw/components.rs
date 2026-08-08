use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::super::Theme;

pub fn search(frame: &mut Frame, area: Rect, query: &str, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("> {query}"), Style::default().fg(theme.query_fg)),
            Span::styled(
                "▏",
                Style::default()
                    .fg(theme.query_fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        area,
    );
}

pub fn separator(frame: &mut Frame, area: Rect, theme: &Theme) {
    frame.render_widget(
        Paragraph::new(Line::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(theme.separator_fg),
        )),
        area,
    );
}

pub fn hint_spans<'a>(hints: &'a [(&'a str, &'a str)], theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    for (index, (key, description)) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(theme.hint_key_fg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {description}"),
            Style::default().fg(theme.hint_desc_fg),
        ));
    }
    spans
}
