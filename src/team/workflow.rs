//! Workflow management - goal and task tracking

use super::{Task, TaskProgress, TaskType};

/// A goal that may contain multiple tasks
#[derive(Debug, Clone)]
pub struct Goal {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub tasks: Vec<usize>, // Task IDs
    pub status: GoalStatus,
}

/// Goal status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalStatus {
    Active,
    Completed,
    Failed,
}

/// Workflow manager for tracking goals and tasks
#[derive(Debug, Default)]
pub struct WorkflowManager {
    goals: Vec<Goal>,
    next_goal_id: usize,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            next_goal_id: 1,
        }
    }

    /// Create a new goal
    pub fn create_goal(&mut self, title: String, description: String) -> usize {
        let id = self.next_goal_id;
        self.next_goal_id += 1;

        self.goals.push(Goal {
            id,
            title,
            description,
            tasks: Vec::new(),
            status: GoalStatus::Active,
        });

        id
    }

    /// Add a task to a goal
    pub fn add_task_to_goal(&mut self, goal_id: usize, task_id: usize) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            goal.tasks.push(task_id);
        }
    }

    /// Update goal status based on task completions
    pub fn update_goal_status(&mut self, goal_id: usize, tasks: &[Task]) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            let goal_tasks: Vec<&Task> = tasks
                .iter()
                .filter(|t| goal.tasks.contains(&t.id))
                .collect();

            if goal_tasks.is_empty() {
                return;
            }

            let all_completed = goal_tasks.iter().all(|t| t.status == TaskProgress::Completed);
            let any_failed = goal_tasks.iter().any(|t| t.status == TaskProgress::Failed);

            if all_completed {
                goal.status = GoalStatus::Completed;
            } else if any_failed {
                goal.status = GoalStatus::Failed;
            }
        }
    }

    /// Get active goals
    pub fn get_active_goals(&self) -> Vec<&Goal> {
        self.goals.iter().filter(|g| g.status == GoalStatus::Active).collect()
    }

    /// Get all goals
    pub fn get_goals(&self) -> &[Goal] {
        &self.goals
    }
}

/// Format tasks for display
pub fn format_task_list(tasks: &[Task]) -> Vec<String> {
    tasks
        .iter()
        .map(|task| {
            let status_icon = match task.status {
                TaskProgress::Pending => "[ ]",
                TaskProgress::InProgress => "[~]",
                TaskProgress::Completed => "[x]",
                TaskProgress::Failed => "[!]",
            };

            let assignee = task.assigned_to.as_deref().unwrap_or("unassigned");

            format!(
                "{} {} ({}) - {}",
                status_icon,
                task.title,
                assignee,
                task.task_type.display_name()
            )
        })
        .collect()
}

/// Get progress percentage for a set of tasks
pub fn calculate_progress(tasks: &[Task]) -> f32 {
    if tasks.is_empty() {
        return 0.0;
    }

    let completed = tasks.iter().filter(|t| t.status == TaskProgress::Completed).count();
    (completed as f32 / tasks.len() as f32) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_manager() {
        let mut wm = WorkflowManager::new();
        let goal_id = wm.create_goal("Test Goal".to_string(), "Description".to_string());
        assert_eq!(goal_id, 1);

        wm.add_task_to_goal(goal_id, 1);
        wm.add_task_to_goal(goal_id, 2);

        let goals = wm.get_active_goals();
        assert_eq!(goals.len(), 1);
        assert_eq!(goals[0].tasks.len(), 2);
    }

    #[test]
    fn test_format_task_list() {
        let tasks = vec![
            Task {
                id: 1,
                title: "Write intro".to_string(),
                description: "Write introduction".to_string(),
                task_type: TaskType::Write,
                status: TaskProgress::Completed,
                assigned_to: Some("Alex".to_string()),
                result: None,
            },
            Task {
                id: 2,
                title: "Research".to_string(),
                description: "Research topic".to_string(),
                task_type: TaskType::Research,
                status: TaskProgress::InProgress,
                assigned_to: Some("Gem".to_string()),
                result: None,
            },
        ];

        let formatted = format_task_list(&tasks);
        assert_eq!(formatted.len(), 2);
        assert!(formatted[0].contains("[x]"));
        assert!(formatted[1].contains("[~]"));
    }

    #[test]
    fn test_calculate_progress() {
        let tasks = vec![
            Task {
                id: 1,
                title: "Task 1".to_string(),
                description: "".to_string(),
                task_type: TaskType::General,
                status: TaskProgress::Completed,
                assigned_to: None,
                result: None,
            },
            Task {
                id: 2,
                title: "Task 2".to_string(),
                description: "".to_string(),
                task_type: TaskType::General,
                status: TaskProgress::Pending,
                assigned_to: None,
                result: None,
            },
        ];

        let progress = calculate_progress(&tasks);
        assert!((progress - 50.0).abs() < 0.01);
    }
}
