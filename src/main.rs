//! WorkyTerm - Claude Code style CLI assistant
//!
//! A conversational AI assistant using installed CLI tools
//! (claude, codex, gemini) or Ollama - no API keys required.

mod cache;
mod config;
mod llm;
mod team;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::io::{self, Write};
use std::time::Instant;

use cache::ResponseCache;
use config::Config;
use team::SupportTeam;

#[derive(Parser, Debug)]
#[command(author, version, about = "WorkyTerm - AI coding assistant", long_about = None)]
struct Args {
    /// Initial prompt to process (or use positional)
    #[arg(long)]
    prompt: Option<String>,

    /// Run in print mode (process and exit)
    #[arg(short = 'p', long)]
    print: bool,

    /// Output as JSON (for programmatic use by other tools)
    #[arg(short = 'j', long)]
    json: bool,

    /// Force specific model: gemini, codex, claude, ollama
    #[arg(short = 'm', long)]
    model: Option<String>,

    /// Hint task type: research, code, write, analyze, explain
    #[arg(short = 't', long)]
    task: Option<String>,

    /// Quiet mode - minimal output, just the response
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Enable response caching (default: enabled)
    #[arg(long, default_value = "true")]
    cache: bool,

    /// Disable response caching
    #[arg(long)]
    no_cache: bool,

    /// Cache TTL in seconds (default: 3600)
    #[arg(long, default_value = "3600")]
    cache_ttl: u64,

    /// Clear the cache and exit
    #[arg(long)]
    clear_cache: bool,

    /// Resume a previous session
    #[arg(short, long)]
    resume: Option<String>,

    /// Config file path
    #[arg(short, long)]
    config: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Positional prompt
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
}

/// Session state tracking
struct Session {
    id: String,
    messages: usize,
    tokens_in: usize,
    tokens_out: usize,
    start_time: Instant,
    model: String,
}

impl Session {
    fn new() -> Self {
        Self {
            id: format!("{:08x}", rand::random::<u32>()),
            messages: 0,
            tokens_in: 0,
            tokens_out: 0,
            start_time: Instant::now(),
            model: String::new(),
        }
    }

    fn estimate_tokens(text: &str) -> usize {
        // Rough estimate: ~4 chars per token
        text.len() / 4
    }
}

/// Global verbose flag
static mut VERBOSE: bool = false;

/// Log a debug message if verbose mode is enabled
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[allow(unused_unsafe)]
        unsafe {
            if VERBOSE {
                eprintln!("{} {}", "[DEBUG]".dimmed(), format!($($arg)*).dimmed());
            }
        }
    };
}

fn rand_random() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    nanos
}

mod rand {
    pub fn random<T: From<u32>>() -> T {
        T::from(super::rand_random())
    }
}

/// Normalize model name shortcuts to full provider names
fn normalize_model_name(model: &str) -> String {
    match model.to_lowercase().as_str() {
        "gemini" | "gem" | "g" => "gemini-cli".to_string(),
        "codex" | "code" | "cx" => "codex-cli".to_string(),
        "claude" | "cl" | "c" => "claude-cli".to_string(),
        "ollama" | "local" | "o" | "l" => "ollama".to_string(),
        other => other.to_string(),
    }
}

/// Map task type hint to TaskType
fn hint_to_task_type(hint: &str) -> team::TaskType {
    match hint.to_lowercase().as_str() {
        "research" | "search" | "find" | "r" => team::TaskType::Research,
        "code" | "analyze" | "debug" | "a" => team::TaskType::Analyze,
        "write" | "draft" | "compose" | "w" => team::TaskType::Write,
        "explain" | "teach" | "e" => team::TaskType::Explain,
        "create" | "creative" | "brainstorm" => team::TaskType::Create,
        "edit" | "improve" | "fix" => team::TaskType::Edit,
        "solve" | "problem" | "s" => team::TaskType::Solve,
        _ => team::TaskType::General,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Set verbose flag
    unsafe { VERBOSE = args.verbose; }

    debug_log!("WorkyTerm starting...");

    // Initialize cache
    let cache_enabled = args.cache && !args.no_cache;
    let cache = ResponseCache::new(cache_enabled, args.cache_ttl);
    cache.init()?;
    debug_log!("Cache initialized (enabled: {})", cache_enabled);

    // Handle cache clear command
    if args.clear_cache {
        match cache.clear() {
            Ok(count) => {
                println!("{} Cleared {} cached entries", "✓".green(), count);
            }
            Err(e) => {
                println!("{} Failed to clear cache: {}", "✗".red(), e);
            }
        }
        return Ok(());
    }

    // Load config
    let config = Config::load(args.config.as_deref())?;
    debug_log!("Config loaded");

    // Use async team initialization for parallel provider detection (faster startup)
    let mut team = SupportTeam::new_async(&config).await;
    let mut session = Session::new();

    // Override model if specified
    if let Some(ref model) = args.model {
        session.model = normalize_model_name(model);
        debug_log!("Forcing model: {}", session.model);
    } else if let Some(member) = team.get_members().iter().find(|m| m.available) {
        session.model = member.provider_type.clone();
    }

    debug_log!("Team: {} members, Model: {}", team.get_members().len(), session.model);

    // Combine prompt sources
    let initial_prompt = args.prompt
        .or_else(|| {
            if !args.query.is_empty() {
                Some(args.query.join(" "))
            } else {
                None
            }
        });

    // Print/JSON/Quiet mode: single query and exit
    if args.print || args.json || args.quiet {
        if let Some(prompt) = initial_prompt {
            let start = Instant::now();

            // Process with optional model override, task hint, and caching
            let (response, from_cache) = process_request_direct(
                &mut team,
                &mut session,
                &prompt,
                args.model.as_deref(),
                args.task.as_deref(),
                &cache,
            ).await?;

            let elapsed = start.elapsed();

            if args.json {
                // JSON output for programmatic consumption
                let json = serde_json::json!({
                    "success": true,
                    "response": response,
                    "model": session.model,
                    "elapsed_ms": elapsed.as_millis(),
                    "tokens_out": Session::estimate_tokens(&response),
                    "cached": from_cache,
                });
                println!("{}", serde_json::to_string(&json)?);
            } else {
                // Plain text output
                println!("{}", response);
            }
        }
        return Ok(());
    }

    // Interactive mode
    print_welcome(&team);

    if let Some(prompt) = initial_prompt {
        process_request(&mut team, &mut session, &prompt, false).await?;
    }

    // Main REPL loop
    loop {
        print!("\n{} ", ">".bright_cyan().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle slash commands
        if input.starts_with('/') {
            if handle_slash_command(input, &team, &mut session).await {
                continue;
            }
            // If command returned false, it means /exit
            break;
        }

        // Handle shell commands with !
        if input.starts_with('!') {
            handle_shell_command(&input[1..]).await;
            continue;
        }

        // Handle file references with @
        let processed_input = process_file_refs(input);

        // Process the request
        process_request(&mut team, &mut session, &processed_input, false).await?;
    }

    Ok(())
}

/// Handle slash commands. Returns true to continue, false to exit.
async fn handle_slash_command(cmd: &str, team: &SupportTeam, session: &mut Session) -> bool {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts[0];
    let _args: Vec<&str> = parts.iter().skip(1).copied().collect();

    match command {
        "/help" | "/h" | "/?" => {
            print_help();
        }
        "/clear" => {
            print!("\x1B[2J\x1B[1;1H");
            print_welcome(team);
        }
        "/status" => {
            print_status(team, session);
        }
        "/team" => {
            print_team(team);
        }
        "/model" => {
            print_models(team);
        }
        "/cost" => {
            print_cost(session);
        }
        "/context" => {
            print_context(session);
        }
        "/compact" => {
            println!("{}", "Context compacted.".dimmed());
            session.tokens_in = session.tokens_in / 2;
            session.tokens_out = session.tokens_out / 2;
        }
        "/exit" | "/quit" | "/q" => {
            println!("{}", "Goodbye!".dimmed());
            return false;
        }
        "/config" => {
            println!("{}", "Config path: ~/.config/workyterm/config.toml".dimmed());
        }
        "/init" => {
            init_project_file();
        }
        "/doctor" => {
            run_doctor(team);
        }
        _ => {
            // Check for custom commands
            if let Some(custom) = find_custom_command(&command[1..]) {
                println!("{} Running custom command: {}", "→".blue(), command);
                println!("{}", custom.dimmed());
            } else {
                println!("{} Unknown command: {}", "?".yellow(), command);
                println!("  Type {} for available commands", "/help".cyan());
            }
        }
    }

    true
}

/// Handle shell commands (! prefix)
async fn handle_shell_command(cmd: &str) {
    println!("{} {}", "$".bright_black(), cmd.dimmed());

    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await;

    match output {
        Ok(out) => {
            if !out.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&out.stdout));
            }
            if !out.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&out.stderr).red());
            }
        }
        Err(e) => {
            println!("{} {}", "Error:".red(), e);
        }
    }
}

/// Process @file references in input
fn process_file_refs(input: &str) -> String {
    let mut result = input.to_string();

    // Find @path patterns
    let re = regex::Regex::new(r"@([\w./\-]+)").ok();
    if let Some(re) = re {
        for cap in re.captures_iter(input) {
            let path = &cap[1];
            if let Ok(content) = std::fs::read_to_string(path) {
                let replacement = format!("\n--- {} ---\n{}\n---", path, content);
                result = result.replace(&format!("@{}", path), &replacement);
            }
        }
    }

    result
}

/// Find custom command in ~/.workyterm/commands/
fn find_custom_command(name: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = format!("{}/.workyterm/commands/{}.md", home, name);
    std::fs::read_to_string(path).ok()
}

/// Initialize project CLAUDE.md file
fn init_project_file() {
    let content = r#"# CLAUDE.md

## Project Overview

[Describe your project here]

## Architecture

[Key architectural decisions]

## Commands

```bash
# Build
# Test
# Run
```

## Conventions

[Code style and conventions]
"#;

    match std::fs::write("CLAUDE.md", content) {
        Ok(_) => println!("{} Created CLAUDE.md", "✓".green()),
        Err(e) => println!("{} Failed to create CLAUDE.md: {}", "✗".red(), e),
    }
}

/// Run diagnostic checks
fn run_doctor(team: &SupportTeam) {
    println!();
    println!("{}", "WorkyTerm Doctor".bold());
    println!("{}", "────────────────".dimmed());

    // Check providers
    println!("\n{}", "Providers:".bold());
    let providers = [
        ("claude", "Claude CLI"),
        ("codex", "Codex CLI"),
        ("gemini", "Gemini CLI"),
    ];

    for (cmd, name) in providers {
        let status = std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if status {
            println!("  {} {} installed", "✓".green(), name);
        } else {
            println!("  {} {} not found", "○".dimmed(), name);
        }
    }

    // Check Ollama
    let ollama = std::process::Command::new("curl")
        .args(["-s", "http://localhost:11434/api/tags"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if ollama {
        println!("  {} Ollama running", "✓".green());
    } else {
        println!("  {} Ollama not running", "○".dimmed());
    }

    // Team status
    println!("\n{}", "Team Status:".bold());
    let available = team.get_members().iter().filter(|m| m.available).count();
    let total = team.get_members().len();
    println!("  {} {}/{} members available",
        if available > 0 { "✓".green() } else { "✗".red() },
        available, total
    );
}

fn print_welcome(team: &SupportTeam) {
    println!();
    println!("{} {}", "WorkyTerm".bold().bright_cyan(), env!("CARGO_PKG_VERSION").dimmed());

    let available = team.get_members().iter().filter(|m| m.available).count();
    if available > 0 {
        let models: Vec<_> = team.get_members()
            .iter()
            .filter(|m| m.available)
            .map(|m| m.provider_type.replace("-cli", ""))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        println!("{} {} via {}",
            "✓".green(),
            format!("{} providers ready", available).dimmed(),
            models.join(", ").cyan()
        );
    } else {
        println!("{}", "⚠ No providers available. Run /doctor".yellow());
    }

    println!();
    println!("{}", "Type your message, or /help for commands.".dimmed());
}

fn print_help() {
    println!();
    println!("{}", "Slash Commands".bold());
    println!("{}", "──────────────".dimmed());

    let commands = [
        ("/help", "Show this help message"),
        ("/clear", "Clear conversation history"),
        ("/status", "Show session status"),
        ("/team", "Show available team members"),
        ("/model", "Show available models"),
        ("/cost", "Show token usage and estimated cost"),
        ("/context", "Show context usage"),
        ("/compact", "Compress conversation context"),
        ("/config", "Show configuration path"),
        ("/init", "Create CLAUDE.md in current directory"),
        ("/doctor", "Run diagnostic checks"),
        ("/exit", "Exit WorkyTerm"),
    ];

    for (cmd, desc) in commands {
        println!("  {:12} {}", cmd.cyan(), desc.dimmed());
    }

    println!();
    println!("{}", "Special Syntax".bold());
    println!("{}", "──────────────".dimmed());
    println!("  {:12} {}", "@file".cyan(), "Include file contents in prompt".dimmed());
    println!("  {:12} {}", "!command".cyan(), "Execute shell command".dimmed());

    println!();
    println!("{}", "Keyboard".bold());
    println!("{}", "────────".dimmed());
    println!("  {:12} {}", "Ctrl+C".cyan(), "Cancel / Exit".dimmed());
    println!("  {:12} {}", "Ctrl+D".cyan(), "Exit".dimmed());
}

fn print_status(team: &SupportTeam, session: &Session) {
    println!();
    println!("{}", "Session Status".bold());
    println!("{}", "──────────────".dimmed());

    let elapsed = session.start_time.elapsed();
    let minutes = elapsed.as_secs() / 60;
    let seconds = elapsed.as_secs() % 60;

    println!("  Session:  {}", session.id.cyan());
    println!("  Duration: {}m {}s", minutes, seconds);
    println!("  Messages: {}", session.messages);
    println!("  Model:    {}", session.model.cyan());

    let available = team.get_members().iter().filter(|m| m.available).count();
    println!("  Team:     {}/{} available", available, team.get_members().len());
}

fn print_team(team: &SupportTeam) {
    println!();
    println!("{}", "Team Members".bold());
    println!("{}", "────────────".dimmed());

    for member in team.get_members() {
        let status = if member.available {
            "●".green()
        } else {
            "○".bright_black()
        };

        println!("  {} {:8} {:15} {}",
            status,
            member.name.bold(),
            member.role,
            member.provider_type.dimmed()
        );
    }
}

fn print_models(team: &SupportTeam) {
    println!();
    println!("{}", "Available Models".bold());
    println!("{}", "────────────────".dimmed());

    let mut seen = std::collections::HashSet::new();
    for member in team.get_members() {
        if member.available && seen.insert(&member.provider_type) {
            println!("  {} {}", "●".green(), member.provider_type.cyan());
        }
    }
}

fn print_cost(session: &Session) {
    println!();
    println!("{}", "Token Usage".bold());
    println!("{}", "───────────".dimmed());

    println!("  Input:    {:>8} tokens", session.tokens_in);
    println!("  Output:   {:>8} tokens", session.tokens_out);
    println!("  Total:    {:>8} tokens", session.tokens_in + session.tokens_out);

    // Rough cost estimate (varies by model)
    let cost = (session.tokens_in as f64 * 0.000003) + (session.tokens_out as f64 * 0.000015);
    println!("  Est Cost: ${:.4}", cost);

    println!();
    println!("{}", "(Note: Using CLI tools - actual cost via provider accounts)".dimmed());
}

fn print_context(session: &Session) {
    println!();
    println!("{}", "Context Usage".bold());
    println!("{}", "─────────────".dimmed());

    // Estimate context as percentage of 200k
    let total = session.tokens_in + session.tokens_out;
    let max_context = 200_000;
    let pct = (total as f64 / max_context as f64 * 100.0).min(100.0);

    // Visual bar
    let filled = (pct / 5.0) as usize;
    let empty = 20 - filled;
    let bar = format!("{}{}",
        "█".repeat(filled).green(),
        "░".repeat(empty).bright_black()
    );

    println!("  [{}] {:.1}%", bar, pct);
    println!("  {} / {} tokens", total, max_context);
}

/// Process a request directly for programmatic use (JSON/quiet modes)
/// Supports model override, task type hints, and response caching
async fn process_request_direct(
    team: &mut SupportTeam,
    session: &mut Session,
    request: &str,
    model_override: Option<&str>,
    task_hint: Option<&str>,
    cache: &ResponseCache,
) -> Result<(String, bool)> {
    debug_log!("Direct processing: \"{}\"", request);

    session.messages += 1;
    session.tokens_in += Session::estimate_tokens(request);

    // Get the task type (from hint or auto-detect)
    let task_type = task_hint
        .map(hint_to_task_type)
        .unwrap_or_else(|| team::analyze_request_detailed(request).primary_type);

    // Find the appropriate member/provider
    let provider_type = if let Some(model) = model_override {
        normalize_model_name(model)
    } else {
        // Use task-based routing
        team.find_member_for_task(task_type)
            .map(|m| m.provider_type.clone())
            .unwrap_or_else(|| session.model.clone())
    };

    debug_log!("Using provider: {} for task: {:?}", provider_type, task_type);

    // Check cache first
    if let Some(cached) = cache.get(request, &provider_type) {
        debug_log!("Cache hit!");
        session.tokens_out += Session::estimate_tokens(&cached);
        session.model = provider_type;
        return Ok((cached, true)); // true = from cache
    }

    // Plan and process the request
    let tasks = team.plan_request(request);

    if tasks.is_empty() {
        return Err(anyhow::anyhow!("No provider available"));
    }

    // Process without streaming for direct mode
    match team.handle_request(request).await {
        Ok((response, _completed_tasks)) => {
            session.tokens_out += Session::estimate_tokens(&response);
            session.model = provider_type.clone();

            // Store in cache
            if let Err(e) = cache.set(request, &provider_type, &response) {
                debug_log!("Failed to cache response: {}", e);
            }

            Ok((response, false)) // false = not from cache
        }
        Err(e) => {
            debug_log!("Error: {}", e);
            Err(e)
        }
    }
}

async fn process_request(
    team: &mut SupportTeam,
    session: &mut Session,
    request: &str,
    quiet: bool
) -> Result<String> {
    debug_log!("Processing: \"{}\"", request);
    let start = Instant::now();

    session.messages += 1;
    session.tokens_in += Session::estimate_tokens(request);

    if !quiet {
        // Show tool indicator
        println!();
        print!("{}", "● ".bright_yellow());
        io::stdout().flush()?;
    }

    // Plan the request
    let tasks = team.plan_request(request);
    debug_log!("Tasks: {}", tasks.len());

    if !quiet {
        // Show what's happening
        for task in &tasks {
            if let Some(ref assignee) = task.assigned_to {
                println!("{} {} → {}",
                    "".bright_black(),
                    task.task_type.display_name().dimmed(),
                    assignee.cyan()
                );
            }
        }
        println!();
    }

    // Process with streaming output
    debug_log!("Calling provider (streaming)...");

    // Create streaming callback that prints each chunk
    let callback: Box<dyn Fn(&str) + Send + Sync> = Box::new(|chunk: &str| {
        print!("{}", chunk);
        let _ = io::stdout().flush();
    });

    match team.handle_request_streaming(request, callback).await {
        Ok((response, _completed_tasks)) => {
            let elapsed = start.elapsed();
            debug_log!("Response in {:.2}s", elapsed.as_secs_f64());

            session.tokens_out += Session::estimate_tokens(&response);

            if !quiet {
                // Show timing
                println!();
                println!("{}", format!("({:.1}s)", elapsed.as_secs_f64()).bright_black());
            }

            Ok(response)
        }
        Err(e) => {
            debug_log!("Error: {}", e);
            if !quiet {
                println!();
                println!("{} {}", "Error:".red().bold(), e);
            }
            Err(e)
        }
    }
}

/// Render response with basic markdown formatting
fn render_response(text: &str) {
    let mut in_code_block = false;
    let mut code_lang = String::new();

    for line in text.lines() {
        // Code block handling
        if line.starts_with("```") {
            if in_code_block {
                println!("{}", "```".bright_black());
                in_code_block = false;
                code_lang.clear();
            } else {
                code_lang = line[3..].trim().to_string();
                if code_lang.is_empty() {
                    println!("{}", "```".bright_black());
                } else {
                    println!("{}", format!("```{}", code_lang).bright_black());
                }
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            // Code content - show with slight indent, could add syntax highlighting
            println!("  {}", line.bright_white());
        } else {
            // Regular text - handle inline formatting
            let formatted = format_inline(line);
            println!("{}", formatted);
        }
    }
}

/// Format inline markdown elements
fn format_inline(text: &str) -> String {
    let mut result = text.to_string();

    // Bold **text**
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let end = start + 2 + end;
            let bold_text = &result[start + 2..end];
            result = format!(
                "{}{}{}",
                &result[..start],
                bold_text.bold(),
                &result[end + 2..]
            );
        } else {
            break;
        }
    }

    // Inline code `text`
    while let Some(start) = result.find('`') {
        if let Some(end) = result[start + 1..].find('`') {
            let end = start + 1 + end;
            let code_text = &result[start + 1..end];
            result = format!(
                "{}{}{}",
                &result[..start],
                code_text.on_bright_black(),
                &result[end + 1..]
            );
        } else {
            break;
        }
    }

    // Headers
    if result.starts_with("### ") {
        result = format!("{}", result[4..].bold());
    } else if result.starts_with("## ") {
        result = format!("{}", result[3..].bold().underline());
    } else if result.starts_with("# ") {
        result = format!("\n{}", result[2..].bold().underline());
    }

    // Bullet points
    if result.starts_with("- ") || result.starts_with("* ") {
        result = format!("  {} {}", "•".dimmed(), &result[2..]);
    }

    // Numbered lists
    if let Some(num_end) = result.find(". ") {
        if num_end <= 3 && result[..num_end].chars().all(|c| c.is_ascii_digit()) {
            result = format!("  {} {}", result[..num_end + 1].dimmed(), &result[num_end + 2..]);
        }
    }

    result
}
