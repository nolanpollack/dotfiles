mod draw;
pub mod model;
mod theme;

pub use theme::{Theme, ThemeOverrides, ThemePalette};

use ratatui::Terminal;

use crate::backend::StdoutBackend;
use model::ScreenView;

pub fn render(view: ScreenView, theme: &Theme, rows: usize, cols: usize) {
    let Ok(mut terminal) = Terminal::new(StdoutBackend::new(rows as u16, cols as u16)) else {
        return;
    };
    terminal.draw(|frame| draw::draw(frame, &view, theme)).ok();
}
