use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::{Frame, Terminal};

const TEXT: Color = Color::Rgb(205, 214, 244);
const MUTED: Color = Color::Rgb(147, 153, 178);
const DIM: Color = Color::Rgb(88, 91, 112);
const BLUE: Color = Color::Rgb(137, 180, 250);
const GREEN: Color = Color::Rgb(166, 227, 161);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Form,
    Creating,
    Finished,
}

struct App {
    screen: Screen,
    session_name: String,
    repo_directory: String,
    base_branch: String,
    branch_name: String,
    worktree_directory: String,
    field: usize,
    creating_step: usize,
    last_step: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            screen: Screen::Form,
            session_name: "reader-reconnect".into(),
            repo_directory: "/Users/nolanpollack/stripe/pay-server".into(),
            base_branch: "master".into(),
            branch_name: "nolanpollack/reader-reconnect".into(),
            worktree_directory: "/Users/nolanpollack/stripe/worktrees".into(),
            field: 0,
            creating_step: 0,
            last_step: Instant::now(),
        }
    }

    fn sync_branch_default(&mut self) {
        self.branch_name = format!("nolanpollack/{}", self.session_name);
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        if code == KeyCode::Char('q') && self.screen != Screen::Form {
            return true;
        }
        match self.screen {
            Screen::Form => match code {
                KeyCode::Esc => return true,
                KeyCode::Enter if self.field == 3 => {
                    self.screen = Screen::Creating;
                    self.creating_step = 0;
                    self.last_step = Instant::now();
                }
                KeyCode::Enter => self.field += 1,
                KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.field = (self.field + 1) % 4;
                }
                KeyCode::Up | KeyCode::BackTab | KeyCode::Char('k') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.field = (self.field + 3) % 4;
                }
                KeyCode::Backspace => {
                    self.active_field_mut().pop();
                    if self.field == 0 {
                        self.sync_branch_default();
                    }
                }
                KeyCode::Char(c) if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT => {
                    self.active_field_mut().push(c);
                    if self.field == 0 {
                        self.sync_branch_default();
                    }
                }
                _ => {}
            },
            Screen::Creating => {
                if code == KeyCode::Esc {
                    self.screen = Screen::Form;
                }
            }
            Screen::Finished => match code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => return true,
                _ => {}
            },
        }
        false
    }

    fn active_field_mut(&mut self) -> &mut String {
        match self.field {
            0 => &mut self.session_name,
            1 => &mut self.repo_directory,
            2 => &mut self.base_branch,
            3 => &mut self.branch_name,
            _ => unreachable!(),
        }
    }

    fn advance(&mut self) {
        if self.screen == Screen::Creating && self.last_step.elapsed() > Duration::from_millis(850) {
            self.creating_step += 1;
            self.last_step = Instant::now();
            if self.creating_step >= STEPS.len() {
                self.screen = Screen::Finished;
            }
        }
    }
}

const STEPS: &[&str] = &[
    "Checking repository and branch availability",
    "Creating the worktree",
    "Creating the branch",
    "Starting the Zellij session",
    "Switching to the new session",
];

fn box_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(Span::styled(format!(" {title} "), Style::default().fg(MUTED)))
}

fn draw_form(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 82, 15);
    frame.render_widget(Clear, area);
    let block = box_block(" New worktree session ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [title, session, repo, base, branch, hints_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ]).areas(inner);
    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled("Create worktree session", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        Span::styled("  Session name is focused first", Style::default().fg(MUTED)),
    ])), title);
    field_line(frame, session, "Session name", &app.session_name, app.field == 0);
    field_line(frame, repo, "Repository", &app.repo_directory, app.field == 1);
    field_line(frame, base, "Branch from", &app.base_branch, app.field == 2);
    field_line(frame, branch, "Branch name", &app.branch_name, app.field == 3);
    hints(frame, hints_area, &[("↑/↓ ctrl-j/k", "field"), ("enter", "next/create"), ("esc", "cancel")]);
}

fn draw_creating(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 78, 15);
    frame.render_widget(Clear, area);
    let block = box_block(" Creating worktree session ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [heading, list, note] = Layout::vertical([Constraint::Length(2), Constraint::Fill(1), Constraint::Length(1)]).areas(inner);
    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(SPINNER[app.creating_step % SPINNER.len()], Style::default().fg(YELLOW)),
        Span::styled("  ", Style::default()),
        Span::styled(creation_step(app, app.creating_step.min(STEPS.len() - 1)), Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
    ])), heading);
    let items = STEPS.iter().enumerate().map(|(index, _)| {
        let (glyph, style) = if index < app.creating_step {
            ("✓", Style::default().fg(GREEN))
        } else if index == app.creating_step {
            (SPINNER[index % SPINNER.len()], Style::default().fg(YELLOW))
        } else {
            ("○", Style::default().fg(DIM))
        };
        ListItem::new(Line::from(vec![Span::styled(format!("  {glyph}  "), style), Span::styled(creation_step(app, index), if index <= app.creating_step { Style::default().fg(TEXT) } else { Style::default().fg(MUTED) })]))
    }).collect::<Vec<_>>();
    frame.render_widget(List::new(items), list);
    frame.render_widget(Paragraph::new(Span::styled("esc: return to details (prototype only)", Style::default().fg(MUTED))), note);
}

fn draw_finished(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 62, 9);
    frame.render_widget(Clear, area);
    let block = box_block(" Worktree session created ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [message, detail, hints_area] = Layout::vertical([Constraint::Length(2), Constraint::Length(2), Constraint::Length(1)]).areas(inner);
    frame.render_widget(Paragraph::new(Line::from(vec![Span::styled("✓", Style::default().fg(GREEN)), Span::styled(format!("  Switched to {}", app.session_name), Style::default().fg(TEXT).add_modifier(Modifier::BOLD))])), message);
    frame.render_widget(Paragraph::new(Span::styled(format!("{} from {}", app.branch_name, app.base_branch), Style::default().fg(MUTED))), detail);
    hints(frame, hints_area, &[("enter", "close")]);
}

fn field_line(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool) {
    let label_style = if focused { Style::default().fg(BLUE).add_modifier(Modifier::BOLD) } else { Style::default().fg(MUTED) };
    let mut spans = vec![Span::styled(format!("{label}: "), label_style), Span::styled(value, Style::default().fg(TEXT))];
    if focused { spans.push(Span::styled("▏", Style::default().fg(BLUE))); }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn hints(frame: &mut Frame, area: Rect, entries: &[(&str, &str)]) {
    let mut spans = Vec::new();
    for (i, (key, action)) in entries.iter().enumerate() {
        if i > 0 { spans.push(Span::styled("   ", Style::default())); }
        spans.push(Span::styled(*key, Style::default().fg(BLUE).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {action}"), Style::default().fg(MUTED)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn centered(area: Rect, width_percent: u16, height: u16) -> Rect {
    let [_, content, _] = Layout::horizontal([
        Constraint::Percentage((100 - width_percent) / 2),
        Constraint::Percentage(width_percent),
        Constraint::Percentage((100 - width_percent) / 2),
    ])
    .areas(area);
    let [_, center, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .areas(content);
    center
}

fn creation_step(app: &App, index: usize) -> String {
    match index {
        1 => format!("Creating worktree in {}/…", app.worktree_directory),
        2 => format!("Creating branch {} from {}", app.branch_name, app.base_branch),
        3 => format!("Starting Zellij session {}", app.session_name),
        _ => STEPS[index].to_string(),
    }
}

fn draw(frame: &mut Frame, app: &App) {
    frame.render_widget(Paragraph::new("Session picker · Ctrl-w: new worktree").style(Style::default().fg(MUTED)).alignment(Alignment::Right), Rect { x: frame.area().x, y: frame.area().y, width: frame.area().width.saturating_sub(1), height: 1 });
    match app.screen {
        Screen::Form => draw_form(frame, app),
        Screen::Creating => draw_creating(frame, app),
        Screen::Finished => draw_finished(frame, app),
    }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::new();
    loop {
        terminal.draw(|frame| draw(frame, &app))?;
        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && app.handle_key(key.code, key.modifiers) { break; }
            }
        }
        app.advance();
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
