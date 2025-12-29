//! Application state - Claude Code style assistant

use anyhow::Result;
use std::path::PathBuf;

use crate::config::Config;
use crate::team::{SupportTeam, Task, TaskProgress};

/// Focus areas in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Input,
    Output,
}

/// Message role in conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
    Tool,      // For showing tool/action usage
    Thinking,  // For showing reasoning
}

/// A message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub provider: Option<String>,
}

/// Todo item for task tracking
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
    pub active_form: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

/// Application state - Claude Code style
pub struct App {
    /// Configuration
    pub config: Config,

    /// Current input
    pub input: String,

    /// Cursor position in input
    pub cursor: usize,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// Todo list (visible to user)
    pub todos: Vec<TodoItem>,

    /// Output scroll position
    pub output_scroll: u16,

    /// Current focus
    pub focus: Focus,

    /// Support team for handling requests
    pub team: SupportTeam,

    /// Current status
    pub status: AppStatus,

    /// Working directory
    pub cwd: PathBuf,

    /// Session stats
    pub turns: usize,
    pub tokens_used: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    Idle,
    Working,
    Complete,
    Error,
}

impl App {
    pub fn new(
        initial_task: Option<String>,
        _output_path: Option<String>,
        config_path: Option<String>,
    ) -> Result<Self> {
        let config = Config::load(config_path.as_deref())?;
        let team = SupportTeam::new(&config);
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let mut app = Self {
            config,
            input: String::new(),
            cursor: 0,
            messages: Vec::new(),
            todos: Vec::new(),
            output_scroll: 0,
            focus: Focus::Input,
            team,
            status: AppStatus::Idle,
            cwd,
            turns: 0,
            tokens_used: 0,
        };

        // Welcome message
        if !app.team.is_available() {
            app.add_message(MessageRole::Error,
                "No AI providers available. Please install claude, codex, or gemini CLI, or start ollama.".to_string(),
                None);
        }

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

    /// Add a todo item
    pub fn add_todo(&mut self, content: &str, active_form: &str) {
        self.todos.push(TodoItem {
            content: content.to_string(),
            status: TodoStatus::Pending,
            active_form: active_form.to_string(),
        });
    }

    /// Update todo status
    pub fn update_todo(&mut self, index: usize, status: TodoStatus) {
        if let Some(todo) = self.todos.get_mut(index) {
            todo.status = status;
        }
    }

    /// Clear completed todos
    pub fn clear_completed_todos(&mut self) {
        self.todos.retain(|t| t.status != TodoStatus::Completed);
    }

    /// Submit current input for processing
    pub async fn submit_task(&mut self) -> Result<()> {
        if self.input.is_empty() || self.status == AppStatus::Working {
            return Ok(());
        }

        let request = self.input.clone();
        self.input.clear();
        self.cursor = 0;
        self.turns += 1;

        // Add user message
        self.add_message(MessageRole::User, request.clone(), None);

        self.status = AppStatus::Working;

        // Show thinking
        self.add_message(MessageRole::Thinking, "Analyzing your request...".to_string(), None);

        // Process with support team
        match self.team.handle_request(&request).await {
            Ok((response, tasks)) => {
                // Remove thinking message
                self.messages.retain(|m| m.role != MessageRole::Thinking);

                // Update todos from tasks
                self.update_todos_from_tasks(&tasks);

                // Add response
                let provider_name = self.team.get_members()
                    .first()
                    .map(|m| m.name.clone());
                self.add_message(MessageRole::Assistant, response, provider_name);

                self.status = AppStatus::Complete;
                self.tokens_used += 100; // Estimate
            }
            Err(e) => {
                // Remove thinking message
                self.messages.retain(|m| m.role != MessageRole::Thinking);

                self.add_message(MessageRole::Error, format!("{}", e), None);
                self.status = AppStatus::Error;
            }
        }

        Ok(())
    }

    /// Update todos from task list
    fn update_todos_from_tasks(&mut self, tasks: &[Task]) {
        for task in tasks {
            // Find or create todo
            let existing = self.todos.iter_mut()
                .find(|t| t.content == task.title);

            if let Some(todo) = existing {
                todo.status = match task.status {
                    TaskProgress::Pending => TodoStatus::Pending,
                    TaskProgress::InProgress => TodoStatus::InProgress,
                    TaskProgress::Completed => TodoStatus::Completed,
                    TaskProgress::Failed => TodoStatus::Pending, // Keep visible
                };
            } else {
                self.todos.push(TodoItem {
                    content: task.title.clone(),
                    status: match task.status {
                        TaskProgress::Pending => TodoStatus::Pending,
                        TaskProgress::InProgress => TodoStatus::InProgress,
                        TaskProgress::Completed => TodoStatus::Completed,
                        TaskProgress::Failed => TodoStatus::Pending,
                    },
                    active_form: task.task_type.display_name().to_string(),
                });
            }
        }
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
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor += 1;
        }
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.output_scroll > 0 {
            self.output_scroll -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        self.output_scroll += 1;
    }

    /// Get status line text
    pub fn status_line(&self) -> String {
        let provider = self.team.get_members()
            .iter()
            .find(|m| m.available)
            .map(|m| m.provider_type.as_str())
            .unwrap_or("none");

        let dir = self.cwd.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".");

        format!(
            "{} | {} | {} turn{} | Press Enter to send, Esc to quit",
            dir,
            provider,
            self.turns,
            if self.turns == 1 { "" } else { "s" }
        )
    }

    /// Get active todo count
    pub fn active_todo_count(&self) -> usize {
        self.todos.iter().filter(|t| t.status != TodoStatus::Completed).count()
    }

    // Legacy methods for compatibility
    pub fn tick(&mut self) {}
    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Input => Focus::Output,
            Focus::Output => Focus::Input,
        };
    }
    pub fn prev_focus(&mut self) {
        self.next_focus();
    }
}

// Re-export types for UI compatibility
pub use crate::team::{Task, TaskProgress};

// Legacy exports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Idle,
    Working,
    Deliberating,
    Complete,
    Error,
}

pub struct WorkerMessage {
    pub worker_id: usize,
    pub message: String,
    pub timestamp: u64,
}
