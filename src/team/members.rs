//! Team member definitions and provider mappings

use super::TaskType;

/// Model preference for a task type
#[derive(Debug, Clone)]
pub struct ModelPreference {
    pub task_type: TaskType,
    pub preferred_providers: Vec<&'static str>,
    pub fallback_providers: Vec<&'static str>,
}

/// Get the best provider order for a task type
pub fn get_provider_preference(task_type: TaskType) -> ModelPreference {
    match task_type {
        TaskType::Write => ModelPreference {
            task_type,
            preferred_providers: vec!["claude-cli", "gemini-cli"],
            fallback_providers: vec!["ollama", "codex-cli"],
        },
        TaskType::Research => ModelPreference {
            task_type,
            preferred_providers: vec!["gemini-cli", "claude-cli"],
            fallback_providers: vec!["ollama", "codex-cli"],
        },
        TaskType::Analyze => ModelPreference {
            task_type,
            preferred_providers: vec!["codex-cli", "claude-cli"],
            fallback_providers: vec!["gemini-cli", "ollama"],
        },
        TaskType::Create => ModelPreference {
            task_type,
            preferred_providers: vec!["claude-cli", "gemini-cli"],
            fallback_providers: vec!["ollama", "codex-cli"],
        },
        TaskType::Edit => ModelPreference {
            task_type,
            preferred_providers: vec!["claude-cli", "codex-cli"],
            fallback_providers: vec!["gemini-cli", "ollama"],
        },
        TaskType::Explain => ModelPreference {
            task_type,
            preferred_providers: vec!["gemini-cli", "claude-cli"],
            fallback_providers: vec!["ollama", "codex-cli"],
        },
        TaskType::Solve => ModelPreference {
            task_type,
            preferred_providers: vec!["codex-cli", "claude-cli"],
            fallback_providers: vec!["gemini-cli", "ollama"],
        },
        TaskType::General => ModelPreference {
            task_type,
            preferred_providers: vec!["claude-cli", "gemini-cli", "codex-cli"],
            fallback_providers: vec!["ollama"],
        },
    }
}

/// Team member personality traits
#[derive(Debug, Clone)]
pub struct MemberPersonality {
    pub greeting: &'static str,
    pub working_message: &'static str,
    pub success_message: &'static str,
    pub style_hint: &'static str,
}

/// Get personality for a team member role
pub fn get_member_personality(role: &str) -> MemberPersonality {
    match role {
        "Writer" => MemberPersonality {
            greeting: "I'll craft that for you!",
            working_message: "Writing...",
            success_message: "Here's what I've written:",
            style_hint: "Be creative but clear",
        },
        "Researcher" => MemberPersonality {
            greeting: "Let me look into that!",
            working_message: "Researching...",
            success_message: "Here's what I found:",
            style_hint: "Be thorough and cite sources",
        },
        "Analyst" => MemberPersonality {
            greeting: "I'll analyze this carefully.",
            working_message: "Analyzing...",
            success_message: "Here's my analysis:",
            style_hint: "Be logical and detailed",
        },
        "Creative" => MemberPersonality {
            greeting: "Let's get creative!",
            working_message: "Brainstorming...",
            success_message: "Here are my ideas:",
            style_hint: "Think outside the box",
        },
        "Editor" => MemberPersonality {
            greeting: "I'll polish that up!",
            working_message: "Editing...",
            success_message: "Here's the improved version:",
            style_hint: "Focus on clarity and flow",
        },
        "Explainer" => MemberPersonality {
            greeting: "Let me break that down for you.",
            working_message: "Explaining...",
            success_message: "Here's the explanation:",
            style_hint: "Be simple and patient",
        },
        "Problem Solver" => MemberPersonality {
            greeting: "I'll help you solve this!",
            working_message: "Problem solving...",
            success_message: "Here's the solution:",
            style_hint: "Be practical and step-by-step",
        },
        _ => MemberPersonality {
            greeting: "I'm here to help!",
            working_message: "Working on it...",
            success_message: "Here you go:",
            style_hint: "Be helpful and friendly",
        },
    }
}

/// Team member icons for UI display
pub fn get_member_icon(role: &str) -> &'static str {
    match role {
        "Writer" => "[W]",
        "Researcher" => "[R]",
        "Analyst" => "[A]",
        "Creative" => "[C]",
        "Editor" => "[E]",
        "Explainer" => "[X]",
        "Problem Solver" => "[S]",
        "General Assistant" => "[G]",
        _ => "[?]",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_preference() {
        let write_pref = get_provider_preference(TaskType::Write);
        assert!(write_pref.preferred_providers.contains(&"claude-cli"));

        let analyze_pref = get_provider_preference(TaskType::Analyze);
        assert!(analyze_pref.preferred_providers.contains(&"codex-cli"));
    }

    #[test]
    fn test_personality() {
        let writer = get_member_personality("Writer");
        assert!(writer.greeting.contains("craft"));

        let researcher = get_member_personality("Researcher");
        assert!(researcher.greeting.contains("look"));
    }
}
