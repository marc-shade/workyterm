//! Support Team - Dynamic multi-model task handling
//!
//! The Support Team analyzes user requests, breaks them into tasks,
//! and assigns the best team member (model) for each task.

mod analyzer;
mod members;
mod workflow;

pub use analyzer::*;
pub use members::*;
pub use workflow::*;

use anyhow::Result;
use crate::llm::{
    ClaudeCliProvider, CodexCliProvider, GeminiCliProvider,
    OllamaProvider, LlmProvider, detect_available_providers, detect_available_providers_async, StreamCallback,
};
use crate::config::Config;

/// A task in the workflow
#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub status: TaskProgress,
    pub assigned_to: Option<String>,
    pub result: Option<String>,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskProgress {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// Types of tasks the team can handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    /// Writing content (blog posts, emails, documents)
    Write,
    /// Finding information (research, facts, sources)
    Research,
    /// Analyzing data or code
    Analyze,
    /// Creative work (brainstorming, ideas, designs)
    Create,
    /// Editing and reviewing
    Edit,
    /// Explaining concepts
    Explain,
    /// Problem solving
    Solve,
    /// General assistance
    General,
}

impl TaskType {
    /// Get friendly name for display
    pub fn display_name(&self) -> &str {
        match self {
            TaskType::Write => "Writing",
            TaskType::Research => "Research",
            TaskType::Analyze => "Analysis",
            TaskType::Create => "Creative",
            TaskType::Edit => "Editing",
            TaskType::Explain => "Explaining",
            TaskType::Solve => "Problem Solving",
            TaskType::General => "General Help",
        }
    }
}

/// A team member with their specialty
#[derive(Debug, Clone)]
pub struct TeamMember {
    pub name: String,
    pub role: String,
    pub specialty: TaskType,
    pub provider_type: String,
    pub available: bool,
}

/// The Support Team that handles user requests
pub struct SupportTeam {
    members: Vec<TeamMember>,
    providers: std::collections::HashMap<String, Box<dyn LlmProvider>>,
    tasks: Vec<Task>,
    next_task_id: usize,
}

/// Helper to create team members and providers from available provider list
/// Extracted to avoid code duplication between sync and async constructors
fn create_team_members_and_providers(
    available: &[String],
    config: &Config,
) -> (Vec<TeamMember>, std::collections::HashMap<String, Box<dyn LlmProvider>>) {
    let mut members = Vec::new();
    let mut providers: std::collections::HashMap<String, Box<dyn LlmProvider>> =
        std::collections::HashMap::new();

    // Create team members based on available providers
    // Priority: Gemini (fast) > Codex > Claude > Ollama

    if available.contains(&"gemini-cli".to_string()) {
        members.push(TeamMember {
            name: "Gem".to_string(),
            role: "Researcher".to_string(),
            specialty: TaskType::Research,
            provider_type: "gemini-cli".to_string(),
            available: true,
        });
        members.push(TeamMember {
            name: "Iris".to_string(),
            role: "Writer".to_string(),
            specialty: TaskType::Write,
            provider_type: "gemini-cli".to_string(),
            available: true,
        });
        members.push(TeamMember {
            name: "Nova".to_string(),
            role: "Explainer".to_string(),
            specialty: TaskType::Explain,
            provider_type: "gemini-cli".to_string(),
            available: true,
        });
        providers.insert("gemini-cli".to_string(), Box::new(GeminiCliProvider::new()));
    }

    if available.contains(&"codex-cli".to_string()) {
        members.push(TeamMember {
            name: "Dev".to_string(),
            role: "Analyst".to_string(),
            specialty: TaskType::Analyze,
            provider_type: "codex-cli".to_string(),
            available: true,
        });
        members.push(TeamMember {
            name: "Cody".to_string(),
            role: "Problem Solver".to_string(),
            specialty: TaskType::Solve,
            provider_type: "codex-cli".to_string(),
            available: true,
        });
        providers.insert("codex-cli".to_string(), Box::new(CodexCliProvider::new()));
    }

    if available.contains(&"claude-cli".to_string()) {
        members.push(TeamMember {
            name: "Alex".to_string(),
            role: "Editor".to_string(),
            specialty: TaskType::Edit,
            provider_type: "claude-cli".to_string(),
            available: true,
        });
        members.push(TeamMember {
            name: "Sam".to_string(),
            role: "Creative".to_string(),
            specialty: TaskType::Create,
            provider_type: "claude-cli".to_string(),
            available: true,
        });
        providers.insert("claude-cli".to_string(), Box::new(ClaudeCliProvider::new()));
    }

    if available.contains(&"ollama".to_string()) {
        if let Some(ollama_config) = config.providers.get("ollama") {
            members.push(TeamMember {
                name: "Local".to_string(),
                role: "General Assistant".to_string(),
                specialty: TaskType::General,
                provider_type: "ollama".to_string(),
                available: true,
            });
            providers.insert("ollama".to_string(), Box::new(OllamaProvider::new(ollama_config.clone())));
        }
    }

    // If no providers available, create placeholder member
    if members.is_empty() {
        members.push(TeamMember {
            name: "Team".to_string(),
            role: "Assistant".to_string(),
            specialty: TaskType::General,
            provider_type: "none".to_string(),
            available: false,
        });
    }

    (members, providers)
}

impl SupportTeam {
    /// Create a new support team based on available providers
    pub fn new(config: &Config) -> Self {
        let available = detect_available_providers();
        let (members, providers) = create_team_members_and_providers(&available, config);

        Self {
            members,
            providers,
            tasks: Vec::new(),
            next_task_id: 1,
        }
    }

    /// Create a new support team with parallel provider detection (faster startup)
    pub async fn new_async(config: &Config) -> Self {
        let available = detect_available_providers_async().await;
        let (members, providers) = create_team_members_and_providers(&available, config);

        Self {
            members,
            providers,
            tasks: Vec::new(),
            next_task_id: 1,
        }
    }

    /// Get available team members
    pub fn get_members(&self) -> &[TeamMember] {
        &self.members
    }

    /// Get current tasks
    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Find the best team member for a task type
    pub fn find_member_for_task(&self, task_type: TaskType) -> Option<&TeamMember> {
        // First try to find exact specialty match (prefer CLI providers)
        if let Some(member) = self.members.iter()
            .filter(|m| m.specialty == task_type && m.available)
            .find(|m| m.provider_type.ends_with("-cli"))
        {
            return Some(member);
        }

        // For General tasks, prefer CLI providers first
        if task_type == TaskType::General {
            if let Some(member) = self.members.iter()
                .filter(|m| m.available)
                .find(|m| m.provider_type.ends_with("-cli"))
            {
                return Some(member);
            }
        }

        // Then any exact specialty match
        if let Some(member) = self.members.iter().find(|m| m.specialty == task_type && m.available) {
            return Some(member);
        }

        // Fall back to any available CLI provider
        if let Some(member) = self.members.iter()
            .filter(|m| m.available)
            .find(|m| m.provider_type.ends_with("-cli"))
        {
            return Some(member);
        }

        // Finally, any available member
        self.members.iter().find(|m| m.available)
    }

    /// Analyze request and create tasks
    pub fn plan_request(&mut self, request: &str) -> Vec<Task> {
        let task_type = analyze_request(request);

        // For simple requests, create single task
        // For complex requests, could break into subtasks
        let task = Task {
            id: self.next_task_id,
            title: format!("{} task", task_type.display_name()),
            description: request.to_string(),
            task_type,
            status: TaskProgress::Pending,
            assigned_to: self.find_member_for_task(task_type).map(|m| m.name.clone()),
            result: None,
        };

        self.next_task_id += 1;
        self.tasks.push(task.clone());

        vec![task]
    }

    /// Process a task with the assigned team member
    pub async fn process_task(&mut self, task_id: usize) -> Result<String> {
        let task = self.tasks.iter_mut().find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.status = TaskProgress::InProgress;

        // Find the provider for this task
        let member = self.members.iter()
            .find(|m| Some(m.name.clone()) == task.assigned_to)
            .ok_or_else(|| anyhow::anyhow!("No team member assigned"))?;

        let provider = self.providers.get(&member.provider_type)
            .ok_or_else(|| anyhow::anyhow!("Provider not available"))?;

        // Create a prompt based on task type
        let prompt = create_task_prompt(&task.description, task.task_type, &member.role);

        match provider.generate(&prompt).await {
            Ok(response) => {
                // Update task in tasks list
                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    t.status = TaskProgress::Completed;
                    t.result = Some(response.clone());
                }
                Ok(response)
            }
            Err(e) => {
                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    t.status = TaskProgress::Failed;
                }
                Err(e)
            }
        }
    }

    /// Process a user request end-to-end
    pub async fn handle_request(&mut self, request: &str) -> Result<(String, Vec<Task>)> {
        // Plan the request into tasks
        let tasks = self.plan_request(request);

        if tasks.is_empty() {
            return Err(anyhow::anyhow!("Could not create tasks for this request"));
        }

        // Process each task
        let mut results = Vec::new();
        for task in &tasks {
            match self.process_task(task.id).await {
                Ok(result) => results.push(result),
                Err(e) => results.push(format!("Error: {}", e)),
            }
        }

        // Combine results
        let final_result = results.join("\n\n");
        let completed_tasks = self.tasks.clone();

        Ok((final_result, completed_tasks))
    }

    /// Process a task with streaming output
    pub async fn process_task_streaming(
        &mut self,
        task_id: usize,
        callback: StreamCallback,
    ) -> Result<String> {
        let task = self.tasks.iter_mut().find(|t| t.id == task_id)
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.status = TaskProgress::InProgress;

        // Find the provider for this task
        let member = self.members.iter()
            .find(|m| Some(m.name.clone()) == task.assigned_to)
            .ok_or_else(|| anyhow::anyhow!("No team member assigned"))?;

        let provider = self.providers.get(&member.provider_type)
            .ok_or_else(|| anyhow::anyhow!("Provider not available"))?;

        // Create a prompt based on task type
        let prompt = create_task_prompt(&task.description, task.task_type, &member.role);

        match provider.generate_streaming(&prompt, callback).await {
            Ok(response) => {
                // Update task in tasks list
                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    t.status = TaskProgress::Completed;
                    t.result = Some(response.clone());
                }
                Ok(response)
            }
            Err(e) => {
                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    t.status = TaskProgress::Failed;
                }
                Err(e)
            }
        }
    }

    /// Process a user request with streaming output
    pub async fn handle_request_streaming(
        &mut self,
        request: &str,
        callback: StreamCallback,
    ) -> Result<(String, Vec<Task>)> {
        // Plan the request into tasks
        let tasks = self.plan_request(request);

        if tasks.is_empty() {
            return Err(anyhow::anyhow!("Could not create tasks for this request"));
        }

        // Process each task with streaming
        let mut results = Vec::new();
        for task in &tasks {
            match self.process_task_streaming(task.id, Box::new(|_| {})).await {
                Ok(result) => {
                    // Stream the actual result
                    callback(&result);
                    results.push(result);
                }
                Err(e) => results.push(format!("Error: {}", e)),
            }
        }

        // Combine results
        let final_result = results.join("\n\n");
        let completed_tasks = self.tasks.clone();

        Ok((final_result, completed_tasks))
    }

    /// Check if team has any available members
    pub fn is_available(&self) -> bool {
        self.members.iter().any(|m| m.available)
    }

    /// Get team status summary
    pub fn status_summary(&self) -> String {
        let available_count = self.members.iter().filter(|m| m.available).count();
        let total_tasks = self.tasks.len();
        let completed = self.tasks.iter().filter(|t| t.status == TaskProgress::Completed).count();

        format!(
            "{} team members ready | {} tasks ({} completed)",
            available_count, total_tasks, completed
        )
    }
}

/// Analyze a request to determine task type
fn analyze_request(request: &str) -> TaskType {
    let lower = request.to_lowercase();

    // Check for writing indicators
    if lower.contains("write") || lower.contains("draft") || lower.contains("compose")
        || lower.contains("blog") || lower.contains("email") || lower.contains("letter")
        || lower.contains("article") || lower.contains("story") {
        return TaskType::Write;
    }

    // Check for research indicators
    if lower.contains("research") || lower.contains("find") || lower.contains("search")
        || lower.contains("look up") || lower.contains("what is") || lower.contains("who is")
        || lower.contains("where is") || lower.contains("when did") {
        return TaskType::Research;
    }

    // Check for analysis indicators
    if lower.contains("analyze") || lower.contains("review") || lower.contains("code")
        || lower.contains("debug") || lower.contains("data") || lower.contains("compare") {
        return TaskType::Analyze;
    }

    // Check for creative indicators
    if lower.contains("create") || lower.contains("brainstorm") || lower.contains("ideas")
        || lower.contains("design") || lower.contains("imagine") || lower.contains("invent") {
        return TaskType::Create;
    }

    // Check for editing indicators
    if lower.contains("edit") || lower.contains("proofread") || lower.contains("improve")
        || lower.contains("fix") || lower.contains("rewrite") || lower.contains("polish") {
        return TaskType::Edit;
    }

    // Check for explanation indicators
    if lower.contains("explain") || lower.contains("how does") || lower.contains("why does")
        || lower.contains("teach") || lower.contains("help me understand") {
        return TaskType::Explain;
    }

    // Check for problem solving indicators
    if lower.contains("solve") || lower.contains("problem") || lower.contains("issue")
        || lower.contains("error") || lower.contains("broken") || lower.contains("not working") {
        return TaskType::Solve;
    }

    // Default to general
    TaskType::General
}

/// Create a prompt tailored to the task type and role
fn create_task_prompt(request: &str, task_type: TaskType, role: &str) -> String {
    let context = match task_type {
        TaskType::Write => "You are a skilled writer. Create clear, engaging content.",
        TaskType::Research => "You are a thorough researcher. Find accurate, relevant information.",
        TaskType::Analyze => "You are an analytical expert. Provide detailed, logical analysis.",
        TaskType::Create => "You are a creative thinker. Generate innovative, original ideas.",
        TaskType::Edit => "You are a meticulous editor. Improve clarity and quality.",
        TaskType::Explain => "You are a patient teacher. Explain concepts simply and clearly.",
        TaskType::Solve => "You are a problem solver. Find practical, effective solutions.",
        TaskType::General => "You are a helpful assistant. Provide useful, friendly assistance.",
    };

    format!(
        "{}\n\nAs the team's {}, please help with this request:\n\n{}",
        context, role, request
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_request_write() {
        assert_eq!(analyze_request("write a blog post"), TaskType::Write);
        assert_eq!(analyze_request("draft an email"), TaskType::Write);
        assert_eq!(analyze_request("compose a letter"), TaskType::Write);
    }

    #[test]
    fn test_analyze_request_research() {
        assert_eq!(analyze_request("research AI trends"), TaskType::Research);
        assert_eq!(analyze_request("find information about"), TaskType::Research);
        assert_eq!(analyze_request("what is machine learning"), TaskType::Research);
    }

    #[test]
    fn test_analyze_request_analyze() {
        assert_eq!(analyze_request("analyze this code"), TaskType::Analyze);
        assert_eq!(analyze_request("review the data"), TaskType::Analyze);
        assert_eq!(analyze_request("debug this function"), TaskType::Analyze);
    }

    #[test]
    fn test_analyze_request_general() {
        assert_eq!(analyze_request("hello"), TaskType::General);
        assert_eq!(analyze_request("thanks"), TaskType::General);
    }
}
