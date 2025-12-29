//! Application state management

use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::llm::{auto_select_provider, detect_available_providers, LlmProvider};
use crate::workers::{Office, WorkerState};

/// Focus areas in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Output,
    Workers,
}

/// Message role in conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

/// A message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub provider: Option<String>,
}

/// Application state
pub struct App {
    /// Configuration
    pub config: Config,

    /// Current task input
    pub input: String,

    /// Cursor position in input
    pub cursor: usize,

    /// Generated output (legacy - kept for compatibility)
    pub output: String,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// Output scroll position
    pub output_scroll: u16,

    /// Current focus
    pub focus: Focus,

    /// Virtual office with workers (optional/legacy)
    pub office: Office,

    /// Current LLM provider
    provider: Option<Box<dyn LlmProvider>>,

    /// Provider name for display
    pub provider_name: Option<String>,

    /// Available providers detected
    pub available_providers: Vec<String>,

    /// Current task status
    pub status: TaskStatus,

    /// Output file path
    pub output_path: Option<PathBuf>,

    /// Animation tick counter
    pub tick: u64,

    /// Messages/thoughts from workers (legacy)
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

        // Detect available providers
        let available_providers = detect_available_providers();

        // Auto-select best provider
        let (provider, provider_name) = match auto_select_provider(&config) {
            Ok(p) => {
                let name = p.name().to_string();
                (Some(p), Some(name))
            }
            Err(_) => (None, None),
        };

        let mut app = Self {
            config,
            input: String::new(),
            cursor: 0,
            output: String::new(),
            messages: Vec::new(),
            output_scroll: 0,
            focus: Focus::Input,
            office,
            provider,
            provider_name,
            available_providers,
            status: TaskStatus::Idle,
            output_path: output_path.map(PathBuf::from),
            tick: 0,
            worker_messages: Vec::new(),
        };

        // If task provided via CLI, set it
        if let Some(task) = initial_task {
            app.input = task;
            app.cursor = app.input.len();
        }

        Ok(app)
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, role: MessageRole, content: String, provider: Option<String>) {
        self.messages.push(Message {
            role,
            content,
            provider,
        });
    }

    /// Submit current task for processing
    pub async fn submit_task(&mut self) -> Result<()> {
        if self.input.is_empty() || self.status == TaskStatus::Working {
            return Ok(());
        }

        let task = self.input.clone();
        self.input.clear();
        self.cursor = 0;

        // Add user message
        self.add_message(MessageRole::User, task.clone(), None);

        self.status = TaskStatus::Working;
        self.output.clear();
        self.worker_messages.clear();

        // Wake up workers (legacy animation)
        for worker in &mut self.office.workers {
            worker.state = WorkerState::Thinking;
        }

        // Process with provider
        if let Some(provider) = &self.provider {
            match provider.generate(&task).await {
                Ok(response) => {
                    self.output = response.clone();
                    self.add_message(
                        MessageRole::Assistant,
                        response,
                        Some(provider.name().to_string()),
                    );
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
                    let error_msg = format!("{}", e);
                    self.output = error_msg.clone();
                    self.add_message(MessageRole::Error, error_msg, None);
                    self.status = TaskStatus::Error;

                    // Workers show error state
                    for worker in &mut self.office.workers {
                        worker.state = WorkerState::Confused;
                    }
                }
            }
        } else {
            let error_msg = "No LLM provider available. Install claude, codex, gemini CLI, or start ollama.".to_string();
            self.add_message(MessageRole::Error, error_msg, None);
            self.status = TaskStatus::Error;
        }

        Ok(())
    }

    /// Add a message from a worker (legacy)
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
        if self.output_scroll > 0 {
            self.output_scroll -= 1;
        }
    }

    /// Scroll output down
    pub fn scroll_down(&mut self) {
        self.output_scroll += 1;
    }
}
