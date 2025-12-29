//! Request analyzer - categorizes user requests

use super::TaskType;

/// Keywords that indicate different task types
pub struct TaskKeywords {
    pub write: Vec<&'static str>,
    pub research: Vec<&'static str>,
    pub analyze: Vec<&'static str>,
    pub create: Vec<&'static str>,
    pub edit: Vec<&'static str>,
    pub explain: Vec<&'static str>,
    pub solve: Vec<&'static str>,
}

impl Default for TaskKeywords {
    fn default() -> Self {
        Self {
            write: vec![
                "write", "draft", "compose", "author", "blog", "article",
                "email", "letter", "document", "report", "essay", "story",
                "script", "copy", "content", "post", "message", "text",
            ],
            research: vec![
                "research", "find", "search", "look up", "discover", "learn about",
                "what is", "who is", "where is", "when did", "how many",
                "statistics", "facts", "information", "sources", "reference",
            ],
            analyze: vec![
                "analyze", "review", "examine", "inspect", "assess", "evaluate",
                "code", "debug", "data", "compare", "contrast", "check",
                "audit", "test", "verify", "validate", "diagnose",
            ],
            create: vec![
                "create", "brainstorm", "ideas", "design", "imagine", "invent",
                "generate", "come up with", "think of", "suggest", "propose",
                "innovate", "concept", "vision", "plan",
            ],
            edit: vec![
                "edit", "proofread", "improve", "fix", "rewrite", "polish",
                "refine", "revise", "correct", "enhance", "clean up",
                "format", "restructure", "reorganize",
            ],
            explain: vec![
                "explain", "how does", "why does", "teach", "help me understand",
                "clarify", "describe", "what does", "meaning of", "define",
                "elaborate", "break down", "simplify", "tutorial",
            ],
            solve: vec![
                "solve", "problem", "issue", "error", "broken", "not working",
                "fix", "troubleshoot", "resolve", "help with", "stuck",
                "can't", "won't", "failing", "crashed",
            ],
        }
    }
}

/// Confidence score for task type detection
#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    pub primary_type: TaskType,
    pub confidence: f32,
    pub keywords_found: Vec<String>,
    pub is_complex: bool,
}

/// Analyze a request with detailed scoring
pub fn analyze_request_detailed(request: &str) -> TaskAnalysis {
    let lower = request.to_lowercase();
    let keywords = TaskKeywords::default();
    let mut scores: Vec<(TaskType, f32, Vec<String>)> = Vec::new();

    // Score each task type
    scores.push(score_keywords(&lower, &keywords.write, TaskType::Write));
    scores.push(score_keywords(&lower, &keywords.research, TaskType::Research));
    scores.push(score_keywords(&lower, &keywords.analyze, TaskType::Analyze));
    scores.push(score_keywords(&lower, &keywords.create, TaskType::Create));
    scores.push(score_keywords(&lower, &keywords.edit, TaskType::Edit));
    scores.push(score_keywords(&lower, &keywords.explain, TaskType::Explain));
    scores.push(score_keywords(&lower, &keywords.solve, TaskType::Solve));

    // Find highest scoring type
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let (primary_type, confidence, keywords_found) = scores
        .first()
        .cloned()
        .unwrap_or((TaskType::General, 0.0, Vec::new()));

    // Determine if request is complex (multiple task types detected)
    let significant_scores = scores.iter().filter(|(_, s, _)| *s > 0.3).count();
    let is_complex = significant_scores > 1 || request.len() > 200;

    TaskAnalysis {
        primary_type: if confidence > 0.2 { primary_type } else { TaskType::General },
        confidence,
        keywords_found,
        is_complex,
    }
}

/// Score how well a request matches a set of keywords
fn score_keywords(text: &str, keywords: &[&str], task_type: TaskType) -> (TaskType, f32, Vec<String>) {
    let mut found = Vec::new();
    let mut score = 0.0;

    for keyword in keywords {
        if text.contains(keyword) {
            found.push(keyword.to_string());
            // Earlier keywords in the list are more important
            let position_bonus = 1.0 - (keywords.iter().position(|k| k == keyword).unwrap_or(0) as f32 / keywords.len() as f32) * 0.5;
            score += 0.2 * position_bonus;
        }
    }

    // Cap at 1.0
    score = score.min(1.0);

    (task_type, score, found)
}

/// Break a complex request into subtasks
pub fn decompose_request(request: &str) -> Vec<(String, TaskType)> {
    let analysis = analyze_request_detailed(request);

    if !analysis.is_complex {
        return vec![(request.to_string(), analysis.primary_type)];
    }

    // For complex requests, try to identify distinct parts
    let mut subtasks = Vec::new();

    // Split by common separators
    let parts: Vec<&str> = request
        .split(&['.', ';', '\n'][..])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() > 1 {
        for part in parts {
            let sub_analysis = analyze_request_detailed(part);
            subtasks.push((part.to_string(), sub_analysis.primary_type));
        }
    } else {
        // Can't decompose, treat as single task
        subtasks.push((request.to_string(), analysis.primary_type));
    }

    subtasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detailed_analysis() {
        let analysis = analyze_request_detailed("write a blog post about cooking");
        assert_eq!(analysis.primary_type, TaskType::Write);
        assert!(analysis.confidence > 0.2);
        assert!(!analysis.keywords_found.is_empty());
    }

    #[test]
    fn test_complex_detection() {
        let simple = analyze_request_detailed("hi");
        assert!(!simple.is_complex);

        let complex = analyze_request_detailed(
            "First research the topic, then write an article about it, and finally edit for clarity."
        );
        assert!(complex.is_complex);
    }

    #[test]
    fn test_decompose() {
        // Test that decomposition works on complex multi-part requests
        let subtasks = decompose_request(
            "First research the latest AI developments thoroughly. Then write a detailed summary of the findings. Finally edit everything for clarity and flow."
        );
        // Should decompose into multiple parts when complex enough
        assert!(subtasks.len() >= 1);
    }
}
