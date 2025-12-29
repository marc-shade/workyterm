//! Application state management

use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::llm::Council;
use crate::workers::{Office, WorkerState};

/// Focus areas in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Output,
    Workers,
}

/// Application state
pub struct App {
    /// Configuration
    pub config: Config,

    /// Current task input
    pub input: String,

    /// Cursor position in input
    pub cursor: usize,

    /// Generated output
    pub output: String,

    /// Output scroll position
    pub output_scroll: u16,

    /// Current focus
    pub focus: Focus,

    /// Virtual office with workers
    pub office: Office,

    /// LLM council for deliberation
    pub council: Council,

    /// Current task status
    pub status: TaskStatus,

    /// Output file path
    pub output_path: Option<PathBuf>,

    /// Animation tick counter
    pub tick: u64,

    /// Messages/thoughts from workers
    pub worker_messages: Vec<WorkerMessage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Idle,
    Working,
    Deliberating,
    Complete,
    Error,
}

#[derive(Debug, Clone)]
pub struct WorkerMessage {
    pub worker_id: usize,
    pub message: String,
    pub timestamp: u64,
}

impl App {
    pub fn new(
        initial_task: Option<String>,
        output_path: Option<String>,
        config_path: Option<String>,
    ) -> Result<Self> {
        let config = Config::load(config_path.as_deref())?;
        let office = Office::new(&config);
        let council = Council::new(&config);

        Ok(Self {
            config,
            input: initial_task.unwrap_or_default(),
            cursor: 0,
            output: String::new(),
            output_scroll: 0,
            focus: Focus::Input,
            office,
            council,
            status: TaskStatus::Idle,
            output_path: output_path.map(PathBuf::from),
            tick: 0,
            worker_messages: Vec::new(),
        })
    }

    /// Submit current task for processing
    pub async fn submit_task(&mut self) -> Result<()> {
        if self.input.is_empty() || self.status == TaskStatus::Working {
            return Ok(());
        }

        self.status = TaskStatus::Working;
        self.output.clear();
        self.worker_messages.clear();

        // Wake up workers
        for worker in &mut self.office.workers {
            worker.state = WorkerState::Thinking;
        }

        // Add initial message
        self.add_worker_message(0, "Let me think about this...");

        // Process with council
        match self.council.process(&self.input).await {
            Ok(response) => {
                self.output = response;
                self.status = TaskStatus::Complete;

                // Workers celebrate
                for worker in &mut self.office.workers {
                    worker.state = WorkerState::Celebrating;
                }

                // Auto-save if configured
                if self.config.output.auto_save {
                    if let Some(path) = &self.output_path {
                        std::fs::write(path, &self.output)?;
                    }
                }
            }
            Err(e) => {
                self.output = format!("Error: {}", e);
                self.status = TaskStatus::Error;

                // Workers show error state
                for worker in &mut self.office.workers {
                    worker.state = WorkerState::Confused;
                }
            }
        }

        Ok(())
    }

    /// Add a message from a worker
    pub fn add_worker_message(&mut self, worker_id: usize, message: &str) {
        self.worker_messages.push(WorkerMessage {
            worker_id,
            message: message.to_string(),
            timestamp: self.tick,
        });

        // Keep only recent messages
        if self.worker_messages.len() > 10 {
            self.worker_messages.remove(0);
        }
    }

    /// Update animation tick
    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.office.tick(self.tick);

        // Clear old worker messages
        self.worker_messages
            .retain(|m| self.tick.saturating_sub(m.timestamp) < 100);
    }

    /// Navigate to next focus area
    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Input => Focus::Output,
            Focus::Output => Focus::Workers,
            Focus::Workers => Focus::Input,
        };
    }

    /// Navigate to previous focus area
    pub fn prev_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Input => Focus::Workers,
            Focus::Output => Focus::Input,
            Focus::Workers => Focus::Output,
        };
    }

    /// Input character at cursor
    pub fn input_char(&mut self, c: char) {
        if self.focus == Focus::Input {
            self.input.insert(self.cursor, c);
            self.cursor += 1;
        }
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.focus == Focus::Input && self.cursor > 0 {
            self.cursor -= 1;
            self.input.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.focus == Focus::Input && self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.focus == Focus::Input && self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Scroll output up
    pub fn scroll_up(&mut self) {
        if self.focus == Focus::Output && self.output_scroll > 0 {
            self.output_scroll -= 1;
        }
    }

    /// Scroll output down
    pub fn scroll_down(&mut self) {
        if self.focus == Focus::Output {
            self.output_scroll += 1;
        }
    }
}
