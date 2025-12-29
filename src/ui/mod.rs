//! UI module - Ratatui-based terminal interface

mod office;
mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Focus, TaskStatus};

pub use office::draw_office;
pub use widgets::*;

/// Draw the main UI
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Input
            Constraint::Min(10),    // Office + Output
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    draw_title(frame, chunks[0]);
    draw_input(frame, app, chunks[1]);

    // Split middle area: office on left, output on right
    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[2]);

    draw_office(frame, app, middle[0]);
    draw_output(frame, app, middle[1]);
    draw_status_bar(frame, app, chunks[3]);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let title = vec![
        Line::from(vec![
            Span::styled(
                "  WorkyTerm  ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Your AI Office Team"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(title).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let style = if app.focus == Focus::Input {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(" What would you like me to work on? ")
        .borders(Borders::ALL)
        .border_style(style);

    let input = Paragraph::new(app.input.as_str())
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);

    // Show cursor if focused
    if app.focus == Focus::Input {
        let cursor_x = area.x + 1 + app.cursor as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn draw_output(frame: &mut Frame, app: &App, area: Rect) {
    let style = if app.focus == Focus::Output {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(" Output ")
        .borders(Borders::ALL)
        .border_style(style);

    let output = Paragraph::new(app.output.as_str())
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.output_scroll, 0));

    frame.render_widget(output, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (status_text, status_color) = match app.status {
        TaskStatus::Idle => ("Ready - Press Enter to start", Color::Gray),
        TaskStatus::Working => ("Working...", Color::Yellow),
        TaskStatus::Deliberating => ("Workers deliberating...", Color::Magenta),
        TaskStatus::Complete => ("Done! Press 'q' to quit or type a new task", Color::Green),
        TaskStatus::Error => ("Error occurred - check output", Color::Red),
    };

    let status = Line::from(vec![
        Span::raw(" Status: "),
        Span::styled(status_text, Style::default().fg(status_color)),
        Span::raw(" | "),
        Span::raw("[Tab] Switch focus | [Enter] Submit | [q] Quit"),
    ]);

    let paragraph = Paragraph::new(status);
    frame.render_widget(paragraph, area);
}
