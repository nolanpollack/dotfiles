use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CtEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::{Frame, Terminal};

const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const TEXT: Color = Color::Rgb(205, 214, 244);
const SUBTEXT: Color = Color::Rgb(147, 153, 178);
const OVERLAY: Color = Color::Rgb(88, 91, 112);
const SURFACE0: Color = Color::Rgb(49, 50, 68);
const GREEN: Color = Color::Rgb(166, 227, 161);
const RED: Color = Color::Rgb(243, 139, 168);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const TEAL: Color = Color::Rgb(148, 226, 213);
const BLUE: Color = Color::Rgb(137, 180, 250);

#[derive(Clone, Copy, PartialEq)]
enum AgentState {
    Blocked,
    Working,
    Done,
    Idle,
    Unknown,
}

fn state_glyph(state: AgentState, tick: usize) -> (&'static str, Color) {
    match state {
        AgentState::Blocked => ("●", RED),
        AgentState::Working => (SPINNER[tick % SPINNER.len()], YELLOW),
        AgentState::Done => ("✓", GREEN),
        AgentState::Idle => ("○", TEAL),
        AgentState::Unknown => ("·", OVERLAY),
    }
}

fn state_label(state: AgentState) -> &'static str {
    match state {
        AgentState::Blocked => "blocked",
        AgentState::Working => "working",
        AgentState::Done => "done",
        AgentState::Idle => "idle",
        AgentState::Unknown => "unknown",
    }
}

struct Agent {
    tool: &'static str,
    state: AgentState,
    doing: &'static str,
    tab_position: usize,
}

fn a(tool: &'static str, state: AgentState, doing: &'static str, tab_position: usize) -> Agent {
    Agent { tool, state, doing, tab_position }
}

struct SessionRow {
    name: &'static str,
    branch: Option<&'static str>,
    is_current: bool,
    active: bool,
    agents: Vec<Agent>,
}

fn s(name: &'static str, branch: Option<&'static str>, is_current: bool, active: bool, agents: Vec<Agent>) -> SessionRow {
    SessionRow { name, branch, is_current, active, agents }
}

struct Repo {
    name: &'static str,
    root: Option<SessionRow>,
    worktrees: Vec<SessionRow>,
}

fn fake_tree() -> (Vec<Repo>, Vec<SessionRow>) {
    let repos = vec![
        Repo {
            name: "pay-server",
            root: Some(s(
                "pay-server",
                Some("main"),
                true,
                true,
                vec![
                    a("claude", AgentState::Working, "editing src/payments.rs", 0),
                    a("codex", AgentState::Idle, "waiting for input", 1),
                ],
            )),
            worktrees: vec![
                s("feature-foo", Some("feature/foo-auth"), false, true, vec![a("claude", AgentState::Blocked, "needs approval to run migration", 0)]),
                s("bugfix-bar", Some("fix/bar-nil-deref"), false, false, vec![]),
            ],
        },
        Repo {
            name: "zoolander",
            root: None,
            worktrees: vec![
                s("refactor-ui", Some("refactor/ui-cleanup"), false, true, vec![a("claude", AgentState::Done, "finished refactor, ready for review", 0)]),
                s("add-metrics", Some("feat/metrics"), false, false, vec![]),
            ],
        },
        Repo {
            name: "android",
            root: Some(s("android", Some("main"), false, true, vec![])),
            worktrees: vec![],
        },
    ];
    let standalone = vec![
        s("notes", None, false, true, vec![]),
        s("scratch", None, false, false, vec![]),
    ];
    (repos, standalone)
}

fn branch_span(branch: Option<&'static str>) -> Vec<Span<'static>> {
    match branch {
        Some(b) => vec![Span::raw(" "), Span::styled(b, Style::default().fg(OVERLAY))],
        None => vec![],
    }
}

fn agent_cluster(agents: &[Agent], tick: usize) -> Vec<Span<'static>> {
    if agents.is_empty() {
        return vec![];
    }
    let mut spans = vec![Span::raw(" ")];
    for (i, ag) in agents.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        let (glyph, color) = state_glyph(ag.state, tick);
        spans.push(Span::styled(glyph, Style::default().fg(color)));
    }
    spans
}

// ---- Scene 1: agent state legend ----

fn scene_legend(tick: usize) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled("Agent status states", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::raw(""),
    ];
    for state in [AgentState::Blocked, AgentState::Working, AgentState::Done, AgentState::Idle, AgentState::Unknown] {
        let (glyph, color) = state_glyph(state, tick);
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(glyph, Style::default().fg(color)),
            Span::raw("  "),
            Span::styled(state_label(state), Style::default().fg(TEXT)),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "\"working\" is the only animated state (braille spinner, ~90ms/frame).",
        Style::default().fg(SUBTEXT),
    )));
    lines
}

// ---- Scene 2: glyph placement — before vs after the session name ----

fn scene_glyph_placement(tick: usize) -> Vec<Line<'static>> {
    let (glyph, color) = state_glyph(AgentState::Working, tick);
    let (glyph2, color2) = state_glyph(AgentState::Idle, tick);
    vec![
        Line::from(Span::styled("Agent glyph placement", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        Line::from(Span::styled("after the name (current mockups)", Style::default().fg(SUBTEXT))),
        Line::from(vec![
            Span::raw("  pay-server"),
            Span::styled(" main", Style::default().fg(OVERLAY)),
            Span::raw(" "),
            Span::styled(glyph, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(glyph2, Style::default().fg(color2)),
        ]),
        Line::raw(""),
        Line::from(Span::styled("before the name, in a fixed-width gutter column", Style::default().fg(SUBTEXT))),
        Line::from(vec![
            Span::styled(glyph, Style::default().fg(color)),
            Span::styled(glyph2, Style::default().fg(color2)),
            Span::raw("  pay-server"),
            Span::styled(" main", Style::default().fg(OVERLAY)),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "recommendation: before, in a fixed-width column — glyphs stay vertically aligned across",
            Style::default().fg(SUBTEXT),
        )),
        Line::from(Span::styled(
            "rows regardless of session-name length, so a scan down the list reads as a status column.",
            Style::default().fg(SUBTEXT),
        )),
        Line::from(Span::styled(
            "after-the-name means the glyph's x-position drifts with every name length, harder to scan.",
            Style::default().fg(SUBTEXT),
        )),
    ]
}

// ---- Scenes 3-4: split panel — agents column always shows every session's agents, grouped by session ----

/// Flattened (session_name, agent) pairs across every repo/worktree, in tree order.
fn agents_by_session(repos: &[Repo]) -> Vec<(&'static str, &Agent)> {
    let mut pairs = Vec::new();
    for repo in repos {
        if let Some(root) = &repo.root {
            pairs.extend(root.agents.iter().map(|ag| (root.name, ag)));
        }
        for wt in &repo.worktrees {
            pairs.extend(wt.agents.iter().map(|ag| (wt.name, ag)));
        }
    }
    pairs
}

/// Builds the agents panel's list items: a bold, non-selectable session-name header
/// whenever the session changes, followed by that session's agent rows (2 lines each).
/// Returns the items plus the item-index of `highlight_pair_idx` (skipping headers),
/// so callers can point `ListState` at the right row.
fn agent_panel_items(pairs: &[(&'static str, &Agent)], tick: usize, highlight_pair_idx: usize) -> (Vec<ListItem<'static>>, usize) {
    let mut items = Vec::new();
    let mut highlighted_item_idx = 0;
    let mut last_session: Option<&str> = None;
    for (i, (session, ag)) in pairs.iter().enumerate() {
        if last_session != Some(*session) {
            items.push(ListItem::new(Line::styled(*session, Style::default().fg(TEXT).add_modifier(Modifier::BOLD))));
            last_session = Some(*session);
        }
        if i == highlight_pair_idx {
            highlighted_item_idx = items.len();
        }
        let (glyph, color) = state_glyph(ag.state, tick);
        items.push(ListItem::new(vec![
            Line::from(vec![
                Span::raw("  "),
                Span::styled(glyph, Style::default().fg(color)),
                Span::raw(" "),
                Span::styled(ag.tool, Style::default().fg(TEXT)),
                Span::styled(format!("  [tab {}]", ag.tab_position), Style::default().fg(OVERLAY)),
            ]),
            Line::from(vec![Span::raw("    "), Span::styled(ag.doing, Style::default().fg(SUBTEXT))]),
        ]));
    }
    (items, highlighted_item_idx)
}

fn scene_split_panel(frame: &mut Frame, area: Rect, tick: usize, agents_focused: bool) {
    let title = if agents_focused {
        "Split panel — agents column focused (Tab / Ctrl-h back to sessions)"
    } else {
        "Split panel — sessions column focused, default (Tab / Ctrl-l to agents)"
    };
    let [title_area, body] = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
    frame.render_widget(Paragraph::new(Line::styled(title, Style::default().fg(TEXT).add_modifier(Modifier::BOLD))), title_area);

    let [list_area, panel_area] = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(body);
    let (repos, standalone) = fake_tree();

    let list_border_color = if agents_focused { OVERLAY } else { BLUE };
    let panel_border_color = if agents_focused { BLUE } else { OVERLAY };

    let list_block = Block::default().borders(Borders::ALL).title(" sessions ").border_style(Style::default().fg(list_border_color));
    let list_inner = list_block.inner(list_area);
    frame.render_widget(list_block, list_area);
    let lines = tree_lines(&repos, &standalone, tick, CurrentStyle::GutterBar);
    let items: Vec<ListItem> = lines.into_iter().map(ListItem::new).collect();
    let mut list_state = ListState::default().with_selected(Some(0));
    frame.render_stateful_widget(
        List::new(items).highlight_style(if agents_focused { Style::default() } else { Style::default().bg(SURFACE0) }),
        list_inner,
        &mut list_state,
    );

    let panel_block = Block::default().borders(Borders::ALL).title(" agents — all sessions ").border_style(Style::default().fg(panel_border_color));
    let panel_inner = panel_block.inner(panel_area);
    frame.render_widget(panel_block, panel_area);
    let pairs = agents_by_session(&repos);
    // highlight the 2nd agent overall (feature-foo's) to make the cross-session grouping obvious
    let (items, highlighted_item_idx) = agent_panel_items(&pairs, tick, 2);
    let mut agent_state = ListState::default().with_selected(if agents_focused { Some(highlighted_item_idx) } else { None });
    frame.render_stateful_widget(
        List::new(items).highlight_style(if agents_focused { Style::default().bg(SURFACE0) } else { Style::default() }),
        panel_inner,
        &mut agent_state,
    );

    let hint = if agents_focused {
        "enter: jump to that agent's tab   tab / ctrl-h: back to sessions   esc: cancel"
    } else {
        "enter: open session   tab / ctrl-l: focus agents panel   esc: cancel"
    };
    let hint_area = Rect { x: area.x, y: area.y + area.height.saturating_sub(1), width: area.width, height: 1 };
    frame.render_widget(Paragraph::new(Span::styled(hint, Style::default().fg(SUBTEXT))), hint_area);
}

// ---- Scene 5: Concept B — inline expandable rows, one flat navigable list ----

fn scene_inline_expand(tick: usize) -> Vec<Line<'static>> {
    let (repos, _) = fake_tree();
    let session = repos[0].root.as_ref().unwrap();
    let mut lines = vec![
        Line::from(Span::styled("Concept B — inline expandable rows", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        Line::from(Span::styled("Sessions with agents get a ▸/▾ expand marker. Collapsed (default):", Style::default().fg(SUBTEXT))),
    ];
    let mut collapsed = vec![Span::raw(" ▸ "), Span::styled("pay-server", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))];
    collapsed.extend(branch_span(session.branch));
    collapsed.extend(agent_cluster(&session.agents, tick));
    lines.push(Line::from(collapsed));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled("Pressing → / l on that row expands it in place, as real selectable rows:", Style::default().fg(SUBTEXT))));
    let mut expanded = vec![Span::raw(" ▾ "), Span::styled("pay-server", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))];
    expanded.extend(branch_span(session.branch));
    lines.push(Line::from(expanded));
    for (i, ag) in session.agents.iter().enumerate() {
        let (glyph, color) = state_glyph(ag.state, tick);
        let selected = i == 0;
        let style = if selected { Style::default().fg(TEXT).bg(SURFACE0) } else { Style::default().fg(SUBTEXT) };
        lines.push(Line::from(vec![
            Span::styled(if selected { "   > " } else { "     " }, style),
            Span::styled(glyph, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(ag.tool, style),
            Span::raw("  "),
            Span::styled(ag.doing, style),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "Up/Down moves through session rows AND agent rows in one flat list — no focus toggle needed.",
        Style::default().fg(SUBTEXT),
    )));
    lines.push(Line::from(Span::styled(
        "<enter> on the session row = open session (default). <enter> on an agent row = jump to that tab/pane.",
        Style::default().fg(SUBTEXT),
    )));
    lines
}

// ---- Scene 6: Concept C — popup overlay, agents hidden until asked for ----

fn scene_agent_popup(frame: &mut Frame, area: Rect, tick: usize) {
    let (repos, standalone) = fake_tree();
    frame.render_widget(
        Paragraph::new(Line::styled(
            "Concept C — popup overlay, on demand",
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        )),
        Rect { x: area.x, y: area.y, width: area.width, height: 1 },
    );
    let list_area = Rect { x: area.x, y: area.y + 2, width: area.width, height: area.height.saturating_sub(3) };
    let lines = tree_lines(&repos, &standalone, tick, CurrentStyle::GutterBar);
    let items: Vec<ListItem> = lines.into_iter().map(ListItem::new).collect();
    let mut list_state = ListState::default().with_selected(Some(0));
    frame.render_stateful_widget(List::new(items).highlight_style(Style::default().bg(SURFACE0)), list_area, &mut list_state);

    let session = repos[0].root.as_ref().unwrap();
    let popup_width = 34u16.min(area.width);
    let popup_height = (session.agents.len() as u16 * 2 + 2).min(area.height);
    let popup_area = Rect { x: area.x + 8, y: area.y + 3, width: popup_width, height: popup_height };
    let popup_block = Block::default().borders(Borders::ALL).title(" agents (a) ").border_style(Style::default().fg(BLUE));
    let popup_inner = popup_block.inner(popup_area);
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(popup_block, popup_area);
    let pairs: Vec<(&'static str, &Agent)> = session.agents.iter().map(|ag| (session.name, ag)).collect();
    let (items, _) = agent_panel_items(&pairs, tick, usize::MAX);
    frame.render_widget(List::new(items), popup_inner);

    let hint_area = Rect { x: area.x, y: area.y + area.height.saturating_sub(1), width: area.width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(
            "enter: open session   a: pick a specific agent   esc: close popup / cancel",
            Style::default().fg(SUBTEXT),
        )),
        hint_area,
    );
}

fn tree_lines(repos: &[Repo], standalone: &[SessionRow], tick: usize, current_style: CurrentStyle) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for repo in repos {
        match &repo.root {
            Some(row) => lines.push(session_line(row, "", tick, current_style)),
            None => lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(repo.name, Style::default().fg(OVERLAY)),
                Span::styled("  no session — <enter> to create", Style::default().fg(OVERLAY).add_modifier(Modifier::ITALIC)),
            ])),
        }
        for (i, wt) in repo.worktrees.iter().enumerate() {
            let prefix = if i + 1 == repo.worktrees.len() { "  └─ " } else { "  ├─ " };
            lines.push(session_line(wt, prefix, tick, current_style));
        }
    }
    for row in standalone {
        lines.push(session_line(row, "", tick, current_style));
    }
    lines
}

#[derive(Clone, Copy)]
enum CurrentStyle {
    GutterBar,
    Bold,
}

fn session_line(row: &SessionRow, prefix: &'static str, tick: usize, current_style: CurrentStyle) -> Line<'static> {
    let gutter = match current_style {
        CurrentStyle::GutterBar if row.is_current => Span::styled("▐", Style::default().fg(BLUE)),
        _ => Span::raw(" "),
    };
    let name_style = if !row.active {
        Style::default().fg(OVERLAY)
    } else if row.is_current {
        Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(TEXT)
    };
    let mut spans = vec![gutter, Span::raw(prefix), Span::styled(row.name, name_style)];
    spans.extend(branch_span(row.branch));
    if !row.active {
        spans.push(Span::styled("  ↺ inactive", Style::default().fg(OVERLAY).add_modifier(Modifier::ITALIC)));
    }
    spans.extend(agent_cluster(&row.agents, tick));
    Line::from(spans)
}

// ---- Scene 4/5: combined mockup with each current-session style ----

fn scene_combined_gutter(tick: usize) -> Vec<Line<'static>> {
    let (repos, standalone) = fake_tree();
    let mut lines = vec![
        Line::from(Span::styled("Current session = leading gutter bar (▐)", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::raw(""),
    ];
    lines.extend(tree_lines(&repos, &standalone, tick, CurrentStyle::GutterBar));
    lines
}

fn scene_combined_bold(tick: usize) -> Vec<Line<'static>> {
    let (repos, standalone) = fake_tree();
    let mut lines = vec![
        Line::from(Span::styled("Current session = bold text only, no glyph", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::raw(""),
    ];
    lines.extend(tree_lines(&repos, &standalone, tick, CurrentStyle::Bold));
    lines
}

// ---- Scene 6: no-root-session edge case ----

fn scene_no_root(_tick: usize) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled("Repo with worktrees but no root session", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("zoolander", Style::default().fg(OVERLAY)),
            Span::styled("  no session — <enter> to create", Style::default().fg(OVERLAY).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(vec![
            Span::raw("  ├─ "),
            Span::styled("refactor-ui", Style::default().fg(TEXT)),
            Span::styled(" refactor/ui-cleanup", Style::default().fg(OVERLAY)),
            Span::raw(" "),
            Span::styled("✓", Style::default().fg(GREEN)),
        ]),
        Line::from(vec![
            Span::raw("  └─ "),
            Span::styled("add-metrics", Style::default().fg(OVERLAY)),
            Span::styled(" feat/metrics", Style::default().fg(OVERLAY)),
            Span::styled("  ↺ inactive", Style::default().fg(OVERLAY).add_modifier(Modifier::ITALIC)),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Root row is grayed + italic hint text; selecting it and hitting <enter> runs `zellij -s zoolander` in the repo root instead of switch_session.",
            Style::default().fg(SUBTEXT),
        )),
    ]
}

// ---- Scene 7: inactive / resurrectable sessions ----

fn scene_resurrect(_tick: usize) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled("Inactive (resurrectable) sessions", Style::default().fg(TEXT).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        Line::from(vec![Span::raw("  notes"), Span::raw("                     "), Span::styled("active", Style::default().fg(GREEN))]),
        Line::from(vec![
            Span::styled("  scratch", Style::default().fg(OVERLAY)),
            Span::raw("                   "),
            Span::styled("↺ inactive — <enter> resurrects", Style::default().fg(OVERLAY).add_modifier(Modifier::ITALIC)),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Inactive rows: dimmed name, no agent cluster (nothing running), '↺ inactive' trailing hint.",
            Style::default().fg(SUBTEXT),
        )),
    ]
}

// ---- Scene 8: full reference layout — everything together ----

fn scene_full_layout(frame: &mut Frame, area: Rect, tick: usize) {
    let (repos, standalone) = fake_tree();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BLUE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [search_area, sep_area, columns_area, status_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    frame.render_widget(Paragraph::new("> pay").style(Style::default().fg(TEXT)), search_area);
    frame.render_widget(
        Paragraph::new(Line::styled("─".repeat(sep_area.width as usize), Style::default().fg(OVERLAY))),
        sep_area,
    );

    let [list_area, sep_col, agents_area] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Length(1), Constraint::Fill(1)]).areas(columns_area);

    let lines = tree_lines(&repos, &standalone, tick, CurrentStyle::GutterBar);
    let items: Vec<ListItem> = lines.into_iter().map(ListItem::new).collect();
    let mut list_state = ListState::default().with_selected(Some(1));
    frame.render_stateful_widget(
        List::new(items).highlight_style(Style::default().bg(SURFACE0)),
        list_area,
        &mut list_state,
    );

    for y in sep_col.y..sep_col.y + sep_col.height {
        frame.render_widget(Paragraph::new(Span::styled("│", Style::default().fg(OVERLAY))), Rect { x: sep_col.x, y, width: 1, height: 1 });
    }

    let pairs = agents_by_session(&repos);
    let (items, _) = agent_panel_items(&pairs, tick, usize::MAX);
    frame.render_widget(List::new(items), agents_area);

    let hints = Line::from(vec![
        Span::styled("enter", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
        Span::styled(" switch  ", Style::default().fg(SUBTEXT)),
        Span::styled("tab/ctrl-l", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
        Span::styled(" agents  ", Style::default().fg(SUBTEXT)),
        Span::styled("ctrl-d", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
        Span::styled(" kill  ", Style::default().fg(SUBTEXT)),
        Span::styled("esc", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
        Span::styled(" cancel", Style::default().fg(SUBTEXT)),
    ]);
    let [hints_area, count_area] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(4)]).areas(status_area);
    frame.render_widget(Paragraph::new(hints), hints_area);
    frame.render_widget(Paragraph::new("6/7").style(Style::default().fg(SUBTEXT)), count_area);
}

fn scenes() -> Vec<&'static str> {
    vec![
        "1/11 legend",
        "2/11 glyph placement",
        "3/11 concept A — split panel (sessions focused)",
        "4/11 concept A — split panel (agents focused)",
        "5/11 concept B — inline expand",
        "6/11 concept C — popup on demand",
        "7/11 combined — gutter bar",
        "8/11 combined — bold",
        "9/11 no-root-session edge case",
        "10/11 inactive / resurrect",
        "11/11 full reference layout",
    ]
}

fn draw(frame: &mut Frame, scene_idx: usize, tick: usize) {
    let titles = scenes();
    let [body, footer] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

    match scene_idx {
        2 => scene_split_panel(frame, body, tick, false),
        3 => scene_split_panel(frame, body, tick, true),
        5 => scene_agent_popup(frame, body, tick),
        10 => scene_full_layout(frame, body, tick),
        _ => {
            let lines = match scene_idx {
                0 => scene_legend(tick),
                1 => scene_glyph_placement(tick),
                4 => scene_inline_expand(tick),
                6 => scene_combined_gutter(tick),
                7 => scene_combined_bold(tick),
                8 => scene_no_root(tick),
                9 => scene_resurrect(tick),
                _ => unreachable!(),
            };
            frame.render_widget(Paragraph::new(lines), pad(body));
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(titles[scene_idx], Style::default().fg(SUBTEXT)),
            Span::raw("   "),
            Span::styled("←/→ switch scene   q quit", Style::default().fg(OVERLAY)),
        ])),
        footer,
    );
}

fn pad(area: Rect) -> Rect {
    Rect { x: area.x + 1, y: area.y + 1, width: area.width.saturating_sub(2), height: area.height.saturating_sub(1) }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let scenes_len = scenes().len();
    let mut scene_idx = 0usize;
    let mut tick = 0usize;
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw(f, scene_idx, tick))?;

        let timeout = Duration::from_millis(90).saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let CtEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Right | KeyCode::Char('l') => scene_idx = (scene_idx + 1) % scenes_len,
                        KeyCode::Left | KeyCode::Char('h') => scene_idx = (scene_idx + scenes_len - 1) % scenes_len,
                        _ => {}
                    }
                }
            }
        }
        if last_tick.elapsed() >= Duration::from_millis(90) {
            tick = tick.wrapping_add(1);
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
