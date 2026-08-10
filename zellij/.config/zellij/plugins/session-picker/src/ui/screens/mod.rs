pub mod create;
pub mod list;

use ratatui::Frame;

use super::Theme;

pub enum ScreenView {
    List(list::ListView),
    Create(create::CreateView),
}

pub fn draw(frame: &mut Frame, view: &ScreenView, theme: &Theme) {
    match view {
        ScreenView::List(view) => list::render::draw(frame, view, theme),
        ScreenView::Create(view) => create::render::draw(frame, view, theme),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_refresh::RefreshView;
    use crate::picker_refresh::RefreshView as PickerRefreshView;
    use crate::ui::screens::list::{AgentRow, AgentState, Focus, ListView, SessionRow};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render(width: u16, focus: Focus, agent_refresh: RefreshView) -> String {
        let refresh = match agent_refresh {
            RefreshView::Refreshing { .. } => PickerRefreshView::Refreshing,
            RefreshView::Failed { .. } => PickerRefreshView::Failed,
            _ => PickerRefreshView::Ready,
        };
        let view = ScreenView::List(ListView {
            query: String::new(),
            sessions: vec![SessionRow {
                name: "dotfiles".into(),
                matched: Vec::new(),
                active: true,
                current: true,
                branch: None,
                nested: false,
                last_sibling: false,
                agent: Some(AgentState::Working),
                rename_draft: None,
            }],
            selected_session: Some(0),
            filtered_count: 1,
            total_count: 1,
            agents: vec![AgentRow {
                session_name: "dotfiles".into(),
                label: "codex".into(),
                pane_id: 7,
                preview: "cargo test --workspace".into(),
                state: AgentState::Working,
            }],
            selected_agent: Some(0),
            focus,
            spinner_tick: 0,
            agent_refresh,
            refresh,
            hints: vec![("enter", "jump")],
        });
        let backend = TestBackend::new(width, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| draw(frame, &view, &Theme::test_default()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn wide_layout_renders_both_presentation_surfaces() {
        let rendered = render(100, Focus::Sessions, RefreshView::Ready);
        assert!(rendered.contains("dotfiles"));
        assert!(rendered.contains("codex"));
        assert!(rendered.contains("cargo test --workspace"));
        assert!(rendered.contains('│'));
    }

    #[test]
    fn narrow_layout_renders_only_the_focused_surface() {
        let rendered = render(60, Focus::Agents, RefreshView::Ready);
        assert!(rendered.contains("codex"));
        assert!(!rendered.contains('│'));
    }

    #[test]
    fn refreshing_snapshot_is_visible_in_the_status_line() {
        let rendered = render(
            100,
            Focus::Sessions,
            RefreshView::Refreshing { cached: true },
        );
        assert!(rendered.contains("⠋ refreshing"));
    }
}
