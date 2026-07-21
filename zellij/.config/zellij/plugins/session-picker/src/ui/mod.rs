mod draw;
mod theme;

pub use theme::{Theme, ThemeOverrides};

use ratatui::Terminal;

use crate::backend::StdoutBackend;
use crate::create::CreateFlow;
use crate::picker::View;
use crate::sessions::SessionInfo;

pub fn render_create(flow: &CreateFlow, theme: &Theme, rows: usize, cols: usize) {
    let Ok(mut terminal) = Terminal::new(StdoutBackend::new(rows as u16, cols as u16)) else {
        return;
    };
    terminal.draw(|f| draw::draw_create_ui(f, flow, theme)).ok();
}

pub fn render(
    view: View<SessionInfo>,
    theme: &Theme,
    hints: &[(&str, &str)],
    renaming: Option<&str>,
    rows: usize,
    cols: usize,
) {
    let Ok(mut terminal) = Terminal::new(StdoutBackend::new(rows as u16, cols as u16)) else {
        return;
    };
    terminal.draw(|f| draw::draw_ui(f, &view, hints, renaming, theme)).ok();
}
