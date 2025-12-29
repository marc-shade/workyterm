//! Custom Ratatui widgets for WorkyTerm

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

/// Animated progress bar showing work completion
pub struct WorkProgress {
    pub progress: f64,
    pub style: Style,
    pub tick: u64,
}

impl WorkProgress {
    pub fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            style: Style::default().fg(Color::Cyan),
            tick: 0,
        }
    }

    pub fn tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }
}

impl Widget for WorkProgress {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let filled_width = (area.width as f64 * self.progress) as u16;
        let animation_char = match (self.tick / 3) % 4 {
            0 => '▓',
            1 => '▒',
            2 => '░',
            _ => '▒',
        };

        for x in area.left()..area.right() {
            let c = if x < area.left() + filled_width {
                '█'
            } else if x == area.left() + filled_width && self.progress > 0.0 {
                animation_char
            } else {
                '░'
            };
            buf.set_string(x, area.top(), c.to_string(), self.style);
        }
    }
}

/// Sparkline showing activity over time
pub struct ActivitySparkline {
    pub data: Vec<u64>,
    pub max: Option<u64>,
    pub style: Style,
}

impl ActivitySparkline {
    pub fn new(data: Vec<u64>) -> Self {
        Self {
            data,
            max: None,
            style: Style::default().fg(Color::Green),
        }
    }
}

impl Widget for ActivitySparkline {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || self.data.is_empty() {
            return;
        }

        let max = self.max.unwrap_or_else(|| *self.data.iter().max().unwrap_or(&1));
        let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        for (i, &value) in self.data.iter().enumerate() {
            if i as u16 >= area.width {
                break;
            }

            let idx = if max > 0 {
                ((value as f64 / max as f64) * 7.0) as usize
            } else {
                0
            };

            let c = blocks[idx.min(7)];
            buf.set_string(area.left() + i as u16, area.top(), c.to_string(), self.style);
        }
    }
}
