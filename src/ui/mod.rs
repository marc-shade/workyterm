//! UI module - Claude Code style conversational interface

mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::app::{App, Focus, Message, MessageRole, TaskStatus};

pub use widgets::*;

/// Draw the main UI - Claude Code style
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Min(10),    // Conversation
            Constraint::Length(5),  // Input
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);
    draw_conversation(frame, app, chunks[1]);
    draw_input(frame, app, chunks[2]);
    draw_status_bar(frame, app, chunks[3]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let provider_name = app.provider_name.as_deref().unwrap_or("auto");

    let header = Line::from(vec![
        Span::styled(
            " workyterm ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!(" {} ", provider_name),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            "Ctrl+C to exit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(header);
    frame.render_widget(paragraph, area);
}

fn draw_conversation(frame: &mut Frame, app: &App, area: Rect) {
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height,
    };

    // Build conversation lines
    let mut lines: Vec<Line> = Vec::new();

    if app.messages.is_empty() {
        // Welcome message
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "  Welcome to WorkyTerm!",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "Type a task below and press Enter.",
                Style::default().fg(Color::Gray),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Available providers: ", Style::default().fg(Color::DarkGray)),
        ]));

        for provider in &app.available_providers {
            lines.push(Line::from(vec![
                Span::styled("    • ", Style::default().fg(Color::Green)),
                Span::styled(provider.as_str(), Style::default().fg(Color::White)),
            ]));
        }

        if app.available_providers.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(
                    "(none detected - install claude, codex, gemini, or start ollama)",
                    Style::default().fg(Color::Red),
                ),
            ]));
        }
    } else {
        // Render messages
        for message in &app.messages {
            lines.push(Line::from("")); // Spacing

            match message.role {
                MessageRole::User => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "  You: ",
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    // Wrap user message
                    for line in message.content.lines() {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(line, Style::default().fg(Color::White)),
                        ]));
                    }
                }
                MessageRole::Assistant => {
                    let provider = message.provider.as_deref().unwrap_or("AI");
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {}: ", provider),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    // Wrap assistant message
                    for line in message.content.lines() {
                        lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(line, Style::default().fg(Color::White)),
                        ]));
                    }
                }
                MessageRole::System => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {}", message.content),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }
                MessageRole::Error => {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  Error: {}", message.content),
                            Style::default().fg(Color::Red),
                        ),
                    ]));
                }
            }
        }

        // Show thinking indicator if working
        if app.status == TaskStatus::Working {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "  ● ",
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    "Thinking...",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .scroll((app.output_scroll, 0));

    frame.render_widget(paragraph, inner_area);

    // Scrollbar
    if app.messages.len() > area.height as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(app.messages.len())
            .position(app.output_scroll as usize);
        frame.render_stateful_widget(
            scrollbar,
            area,
            &mut scrollbar_state,
        );
    }
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == Focus::Input {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " Task ",
            Style::default().fg(Color::Cyan),
        ));

    // Show placeholder if empty
    let display_text = if app.input.is_empty() && app.focus == Focus::Input {
        "Type your task here...".to_string()
    } else {
        app.input.clone()
    };

    let text_style = if app.input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(Span::styled(&display_text, text_style))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);

    // Show cursor if focused
    if app.focus == Focus::Input {
        let cursor_x = area.x + 1 + app.cursor as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x.min(area.x + area.width - 2), cursor_y));
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (status_text, status_color) = match app.status {
        TaskStatus::Idle => ("Ready", Color::Green),
        TaskStatus::Working => ("Working...", Color::Yellow),
        TaskStatus::Deliberating => ("Council deliberating...", Color::Magenta),
        TaskStatus::Complete => ("Complete", Color::Green),
        TaskStatus::Error => ("Error", Color::Red),
    };

    let provider_count = app.available_providers.len();

    let status = Line::from(vec![
        Span::raw(" "),
        Span::styled("●", Style::default().fg(status_color)),
        Span::raw(" "),
        Span::styled(status_text, Style::default().fg(status_color)),
        Span::raw("  │  "),
        Span::styled(
            format!("{} provider{}", provider_count, if provider_count == 1 { "" } else { "s" }),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  │  "),
        Span::styled(
            "Enter: submit  ↑↓: scroll  q: quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(status)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, area);
}

// Keep office module for optional animation mode
pub mod office {
    use ratatui::{layout::Rect, Frame};
    use crate::app::App;

    pub fn draw_office(_frame: &mut Frame, _app: &App, _area: Rect) {
        // Office view disabled in conversational mode
        // Could be re-enabled with --office flag
    }
}

pub use office::draw_office;
