//! Virtual office rendering with animated pixel art workers

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Focus};
use crate::workers::{Worker, WorkerState};

/// Draw the virtual office with animated workers
pub fn draw_office(frame: &mut Frame, app: &App, area: Rect) {
    let style = if app.focus == Focus::Workers {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(" The Office ")
        .borders(Borders::ALL)
        .border_style(style);

    // Create office scene
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Draw office background
    draw_office_background(frame, inner_area, app.tick);

    // Draw each worker
    let worker_width = inner_area.width / (app.office.workers.len() as u16 + 1);
    for (i, worker) in app.office.workers.iter().enumerate() {
        let x = inner_area.x + (i as u16 + 1) * worker_width - worker_width / 2;
        let y = inner_area.y + inner_area.height / 2;
        draw_worker(frame, worker, x, y, app.tick);
    }

    // Draw thought bubbles
    for msg in &app.worker_messages {
        if msg.worker_id < app.office.workers.len() {
            let x = inner_area.x + (msg.worker_id as u16 + 1) * worker_width - worker_width / 2;
            let y = inner_area.y + 1;
            draw_thought_bubble(frame, &msg.message, x, y);
        }
    }
}

fn draw_office_background(frame: &mut Frame, area: Rect, tick: u64) {
    // Simple office decorations
    let decorations = vec![
        // Desk line
        Line::from("━".repeat(area.width as usize)),
        // Floor pattern (animated)
        Line::from(if tick % 20 < 10 { "░" } else { "▒" }.repeat(area.width as usize)),
    ];

    let floor_y = area.y + area.height.saturating_sub(2);
    if floor_y < area.y + area.height {
        let floor = Paragraph::new(decorations);
        let floor_area = Rect::new(area.x, floor_y, area.width, 2.min(area.height));
        frame.render_widget(floor, floor_area);
    }
}

fn draw_worker(frame: &mut Frame, worker: &Worker, x: u16, y: u16, tick: u64) {
    let sprite = get_worker_sprite(worker, tick);
    let color = get_worker_color(worker);

    for (i, line) in sprite.iter().enumerate() {
        let span = Span::styled(*line, Style::default().fg(color));
        let para = Paragraph::new(Line::from(span));
        let sprite_y = y.saturating_sub(sprite.len() as u16 / 2) + i as u16;
        let sprite_x = x.saturating_sub(line.len() as u16 / 2);

        if sprite_y < frame.area().height && sprite_x < frame.area().width {
            let sprite_area = Rect::new(sprite_x, sprite_y, line.len() as u16, 1);
            frame.render_widget(para, sprite_area);
        }
    }

    // Draw name under worker
    let name = Paragraph::new(Line::from(Span::styled(
        &worker.name,
        Style::default()
            .fg(color)
            .add_modifier(Modifier::BOLD),
    )));
    let name_x = x.saturating_sub(worker.name.len() as u16 / 2);
    let name_y = y + 3;
    if name_y < frame.area().height {
        let name_area = Rect::new(name_x, name_y, worker.name.len() as u16, 1);
        frame.render_widget(name, name_area);
    }
}

fn get_worker_sprite(worker: &Worker, tick: u64) -> Vec<&'static str> {
    let frame = (tick / 5) % 4;

    match worker.state {
        WorkerState::Idle => match frame {
            0 | 2 => vec!["  O  ", " /|\\ ", " / \\ "],
            1 => vec!["  o  ", " /|\\ ", " / \\ "],
            _ => vec!["  O  ", " /|\\", "  / \\"],
        },
        WorkerState::Thinking => match frame {
            0 => vec!["  O? ", " /|\\ ", " / \\ "],
            1 => vec!["  O  ", "\\|/  ", " / \\ "],
            2 => vec!["  O! ", " /|\\ ", " / \\ "],
            _ => vec!["  O  ", " \\|/ ", " / \\ "],
        },
        WorkerState::Typing => match frame {
            0 => vec!["  O  ", " /|_ ", " / \\ "],
            1 => vec!["  O  ", " /|\\~", " / \\ "],
            2 => vec!["  O  ", " _|/ ", " / \\ "],
            _ => vec!["  O  ", "~/|\\ ", " / \\ "],
        },
        WorkerState::Celebrating => match frame {
            0 => vec!["\\O/  ", " |   ", "/ \\  "],
            1 => vec![" \\O/ ", "  |  ", " / \\ "],
            2 => vec!["  \\O/", "   | ", "  / \\"],
            _ => vec![" \\O/ ", "  |  ", " / \\ "],
        },
        WorkerState::Confused => match frame {
            0 => vec!["  O? ", " /|\\ ", " / \\ "],
            1 => vec!["  ?O ", " /|\\ ", " / \\ "],
            2 => vec!["  O?!", " /|\\ ", " / \\ "],
            _ => vec!["  O  ", " /|?\\ ", " / \\ "],
        },
        WorkerState::Collaborating => match frame {
            0 => vec!["  O> ", " /|\\ ", " / \\ "],
            1 => vec!["  O>>", " /|\\ ", " / \\ "],
            2 => vec!["  O> ", " /|\\>", " / \\ "],
            _ => vec!["  O  ", " /|\\>", " / \\>"],
        },
    }
}

fn get_worker_color(worker: &Worker) -> Color {
    match worker.state {
        WorkerState::Idle => Color::Gray,
        WorkerState::Thinking => Color::Yellow,
        WorkerState::Typing => Color::Cyan,
        WorkerState::Celebrating => Color::Green,
        WorkerState::Confused => Color::Red,
        WorkerState::Collaborating => Color::Magenta,
    }
}

fn draw_thought_bubble(frame: &mut Frame, message: &str, x: u16, y: u16) {
    let max_width = 20;
    let truncated: String = if message.chars().count() > max_width {
        // UTF-8 safe truncation using char boundaries
        let truncate_at = max_width - 3;
        let safe_slice: String = message.chars().take(truncate_at).collect();
        format!("{}...", safe_slice)
    } else {
        message.to_string()
    };

    let bubble = vec![
        format!("╭{}╮", "─".repeat(truncated.len() + 2)),
        format!("│ {} │", truncated),
        format!("╰{}╯", "─".repeat(truncated.len() + 2)),
        "    ○".to_string(),
        "   o".to_string(),
    ];

    for (i, line) in bubble.iter().enumerate() {
        let span = Span::styled(line.as_str(), Style::default().fg(Color::White));
        let para = Paragraph::new(Line::from(span));
        let bubble_y = y + i as u16;
        if bubble_y < frame.area().height && x < frame.area().width {
            let bubble_area = Rect::new(x, bubble_y, line.len() as u16, 1);
            frame.render_widget(para, bubble_area);
        }
    }
}
