use std::io::{self, Write};

use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use ratatui::style::{Color, Modifier, Style};

pub struct StdoutBackend {
    rows: u16,
    cols: u16,
    buf: Vec<u8>,
}

impl StdoutBackend {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self { rows, cols, buf: Vec::with_capacity(8192) }
    }

    fn write_style(&mut self, style: &Style) {
        write!(self.buf, "\x1b[0m").ok();
        if style.add_modifier.contains(Modifier::BOLD) {
            write!(self.buf, "\x1b[1m").ok();
        }
        if style.add_modifier.contains(Modifier::ITALIC) {
            write!(self.buf, "\x1b[3m").ok();
        }
        if style.add_modifier.contains(Modifier::UNDERLINED) {
            write!(self.buf, "\x1b[4m").ok();
        }
        Self::write_color(&mut self.buf, style.fg, 38, 30);
        Self::write_color(&mut self.buf, style.bg, 48, 40);
    }

    fn write_color(buf: &mut Vec<u8>, color: Option<Color>, rgb_base: u8, named_base: u8) {
        match color {
            Some(Color::Rgb(r, g, b)) => write!(buf, "\x1b[{rgb_base};2;{r};{g};{b}m").ok(),
            Some(Color::Indexed(n)) => write!(buf, "\x1b[{rgb_base};5;{n}m").ok(),
            Some(Color::Black) => write!(buf, "\x1b[{}m", named_base).ok(),
            Some(Color::Red) => write!(buf, "\x1b[{}m", named_base + 1).ok(),
            Some(Color::Green) => write!(buf, "\x1b[{}m", named_base + 2).ok(),
            Some(Color::Yellow) => write!(buf, "\x1b[{}m", named_base + 3).ok(),
            Some(Color::Blue) => write!(buf, "\x1b[{}m", named_base + 4).ok(),
            Some(Color::Magenta) => write!(buf, "\x1b[{}m", named_base + 5).ok(),
            Some(Color::Cyan) => write!(buf, "\x1b[{}m", named_base + 6).ok(),
            Some(Color::White) => write!(buf, "\x1b[{}m", named_base + 7).ok(),
            Some(Color::DarkGray) => write!(buf, "\x1b[{}m", named_base + 60).ok(),
            Some(Color::LightRed) => write!(buf, "\x1b[{}m", named_base + 61).ok(),
            Some(Color::LightGreen) => write!(buf, "\x1b[{}m", named_base + 62).ok(),
            Some(Color::LightYellow) => write!(buf, "\x1b[{}m", named_base + 63).ok(),
            Some(Color::LightBlue) => write!(buf, "\x1b[{}m", named_base + 64).ok(),
            Some(Color::LightMagenta) => write!(buf, "\x1b[{}m", named_base + 65).ok(),
            Some(Color::LightCyan) => write!(buf, "\x1b[{}m", named_base + 66).ok(),
            Some(Color::Gray) => write!(buf, "\x1b[{}m", named_base + 67).ok(),
            _ => None,
        };
    }
}

impl Backend for StdoutBackend {
    type Error = io::Error;

    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let mut last_pos: Option<(u16, u16)> = None;
        for (x, y, cell) in content {
            if last_pos != Some((x, y)) {
                write!(self.buf, "\x1b[{};{}H", y + 1, x + 1).ok();
            }
            self.write_style(&cell.style());
            let sym = cell.symbol();
            self.buf.extend_from_slice(sym.as_bytes());
            last_pos = Some((x + sym.chars().count() as u16, y));
        }
        write!(self.buf, "\x1b[0m").ok();
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.buf, "\x1b[?25l").ok();
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.buf, "\x1b[?25h").ok();
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::ORIGIN)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, _pos: P) -> io::Result<()> {
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        write!(self.buf, "\x1b[2J\x1b[H").ok();
        Ok(())
    }

    fn clear_region(&mut self, _clear_type: ClearType) -> io::Result<()> {
        self.clear()
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size { width: self.cols, height: self.rows })
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: Size { width: self.cols, height: self.rows },
            pixels: Size::default(),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().write_all(&self.buf)?;
        io::stdout().flush()?;
        self.buf.clear();
        Ok(())
    }
}
