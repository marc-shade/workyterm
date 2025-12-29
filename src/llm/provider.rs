//! LLM Provider implementations - CLI-first, API optional

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::config::ProviderConfig;

/// Callback type for streaming responses
pub type StreamCallback = Box<dyn Fn(&str) + Send + Sync>;

/// Generic LLM provider trait
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;

    /// Generate with streaming output - calls callback for each chunk
    async fn generate_streaming(
        &self,
        prompt: &str,
        callback: StreamCallback,
    ) -> Result<String> {
        // Default: fall back to non-streaming
        let response = self.generate(prompt).await?;
        callback(&response);
        Ok(response)
    }

    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn supports_streaming(&self) -> bool {
        false
    }
}

// ============================================================================
// CLI-BASED PROVIDERS (No API Key Required)
// ============================================================================

/// Claude Code CLI provider
pub struct ClaudeCliProvider {
    command: String,
}

impl ClaudeCliProvider {
    pub fn new() -> Self {
        Self {
            command: "claude".to_string(),
        }
    }

    pub fn is_installed() -> bool {
        Command::new("which")
            .arg("claude")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for ClaudeCliProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LlmProvider for ClaudeCliProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        // claude -p "prompt" - the prompt is positional, -p means print mode
        let output = tokio::process::Command::new(&self.command)
            .args(["-p", "--output-format", "text", prompt])
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Claude CLI error: {}", stderr))
        }
    }

    async fn generate_streaming(
        &self,
        prompt: &str,
        callback: StreamCallback,
    ) -> Result<String> {
        let mut child = tokio::process::Command::new(&self.command)
            .args(["-p", "--output-format", "text", prompt])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let mut reader = BufReader::new(stdout).lines();
        let mut full_response = String::new();

        while let Some(line) = reader.next_line().await? {
            callback(&line);
            callback("\n");
            if !full_response.is_empty() {
                full_response.push('\n');
            }
            full_response.push_str(&line);
        }

        let status = child.wait().await?;
        if !status.success() {
            return Err(anyhow::anyhow!("Claude CLI exited with error"));
        }

        Ok(full_response)
    }

    fn name(&self) -> &str {
        "Claude"
    }

    fn is_available(&self) -> bool {
        Self::is_installed()
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// OpenAI Codex CLI provider
pub struct CodexCliProvider {
    command: String,
}

impl CodexCliProvider {
    pub fn new() -> Self {
        Self {
            command: "codex".to_string(),
        }
    }

    pub fn is_installed() -> bool {
        Command::new("which")
            .arg("codex")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for CodexCliProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LlmProvider for CodexCliProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        // codex exec "prompt" for non-interactive mode
        let output = tokio::process::Command::new(&self.command)
            .args(["exec", prompt])
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Codex CLI error: {}", stderr))
        }
    }

    async fn generate_streaming(
        &self,
        prompt: &str,
        callback: StreamCallback,
    ) -> Result<String> {
        let mut child = tokio::process::Command::new(&self.command)
            .args(["exec", prompt])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let mut reader = BufReader::new(stdout).lines();
        let mut full_response = String::new();

        while let Some(line) = reader.next_line().await? {
            callback(&line);
            callback("\n");
            if !full_response.is_empty() {
                full_response.push('\n');
            }
            full_response.push_str(&line);
        }

        let status = child.wait().await?;
        if !status.success() {
            return Err(anyhow::anyhow!("Codex CLI exited with error"));
        }

        Ok(full_response)
    }

    fn name(&self) -> &str {
        "Codex"
    }

    fn is_available(&self) -> bool {
        Self::is_installed()
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Gemini CLI provider
pub struct GeminiCliProvider {
    command: String,
}

impl GeminiCliProvider {
    pub fn new() -> Self {
        Self {
            command: "gemini".to_string(),
        }
    }

    pub fn is_installed() -> bool {
        Command::new("which")
            .arg("gemini")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for GeminiCliProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LlmProvider for GeminiCliProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        // gemini "prompt" - uses positional prompt in non-interactive mode
        let output = tokio::process::Command::new(&self.command)
            .arg(prompt)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Gemini CLI error: {}", stderr))
        }
    }

    async fn generate_streaming(
        &self,
        prompt: &str,
        callback: StreamCallback,
    ) -> Result<String> {
        let mut child = tokio::process::Command::new(&self.command)
            .arg(prompt)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let mut reader = BufReader::new(stdout).lines();
        let mut full_response = String::new();

        while let Some(line) = reader.next_line().await? {
            callback(&line);
            callback("\n");
            if !full_response.is_empty() {
                full_response.push('\n');
            }
            full_response.push_str(&line);
        }

        let status = child.wait().await?;
        if !status.success() {
            return Err(anyhow::anyhow!("Gemini CLI exited with error"));
        }

        Ok(full_response)
    }

    fn name(&self) -> &str {
        "Gemini"
    }

    fn is_available(&self) -> bool {
        Self::is_installed()
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

// ============================================================================
// API-BASED PROVIDERS (Fallback when CLI not available)
// ============================================================================

/// Ollama provider (local API)
pub struct OllamaProvider {
    client: Client,
    config: ProviderConfig,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn is_running() -> bool {
        std::net::TcpStream::connect("127.0.0.1:11434").is_ok()
    }
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: &self.config.model,
            prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.config.endpoint);
        let response: OllamaResponse = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.response)
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn is_available(&self) -> bool {
        Self::is_running()
    }
}

/// OpenAI API provider
pub struct OpenAiProvider {
    client: Client,
    config: ProviderConfig,
    api_key: String,
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageResponse,
}

#[derive(Deserialize)]
struct OpenAiMessageResponse {
    content: String,
}

impl OpenAiProvider {
    pub fn new(config: ProviderConfig, api_key: String) -> Self {
        Self {
            client: Client::new(),
            config,
            api_key,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let request = OpenAiRequest {
            model: &self.config.model,
            messages: vec![OpenAiMessage {
                role: "user",
                content: prompt,
            }],
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        let url = format!("{}/chat/completions", self.config.endpoint);
        let response: OpenAiResponse = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Anthropic API provider
pub struct AnthropicProvider {
    client: Client,
    config: ProviderConfig,
    api_key: String,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<AnthropicMessage<'a>>,
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig, api_key: String) -> Self {
        Self {
            client: Client::new(),
            config,
            api_key,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let request = AnthropicRequest {
            model: &self.config.model,
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            messages: vec![AnthropicMessage {
                role: "user",
                content: prompt,
            }],
        };

        let url = format!("{}/messages", self.config.endpoint);
        let response: AnthropicResponse = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from Anthropic"))
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

// ============================================================================
// PROVIDER DETECTION AND FACTORY
// ============================================================================

/// Detect all available providers (CLI first, then API) - sequential version
pub fn detect_available_providers() -> Vec<String> {
    let mut available = Vec::new();

    // CLI providers (preferred)
    if ClaudeCliProvider::is_installed() {
        available.push("claude-cli".to_string());
    }
    if CodexCliProvider::is_installed() {
        available.push("codex-cli".to_string());
    }
    if GeminiCliProvider::is_installed() {
        available.push("gemini-cli".to_string());
    }

    // Local API
    if OllamaProvider::is_running() {
        available.push("ollama".to_string());
    }

    available
}

/// Detect all available providers in parallel for faster startup
pub async fn detect_available_providers_async() -> Vec<String> {
    use tokio::task::JoinSet;

    let mut set = JoinSet::new();

    // Check CLI providers in parallel
    set.spawn(async {
        if tokio::process::Command::new("which")
            .arg("claude")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            Some("claude-cli".to_string())
        } else {
            None
        }
    });

    set.spawn(async {
        if tokio::process::Command::new("which")
            .arg("codex")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            Some("codex-cli".to_string())
        } else {
            None
        }
    });

    set.spawn(async {
        if tokio::process::Command::new("which")
            .arg("gemini")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            Some("gemini-cli".to_string())
        } else {
            None
        }
    });

    set.spawn(async {
        // Check Ollama with a short timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(500))
            .build()
            .ok();

        if let Some(client) = client {
            if client.get("http://localhost:11434/api/tags")
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false)
            {
                return Some("ollama".to_string());
            }
        }
        None
    });

    // Collect results
    let mut available = Vec::new();
    while let Some(result) = set.join_next().await {
        if let Ok(Some(provider)) = result {
            available.push(provider);
        }
    }

    available
}

/// Factory function to create provider - CLI first, API fallback
pub fn create_provider(
    provider_name: &str,
    config: ProviderConfig,
    api_key: Option<String>,
) -> Result<Box<dyn LlmProvider>> {
    match provider_name {
        // CLI providers (no API key needed)
        "claude-cli" | "claude" => Ok(Box::new(ClaudeCliProvider::new())),
        "codex-cli" | "codex" => Ok(Box::new(CodexCliProvider::new())),
        "gemini-cli" | "gemini" => Ok(Box::new(GeminiCliProvider::new())),

        // Local API (no API key needed)
        "ollama" => Ok(Box::new(OllamaProvider::new(config))),

        // Cloud API (API key required)
        "openai" | "openai-api" => {
            let key = api_key.ok_or_else(|| anyhow::anyhow!("OpenAI API key required"))?;
            Ok(Box::new(OpenAiProvider::new(config, key)))
        }
        "anthropic" | "anthropic-api" => {
            let key = api_key.ok_or_else(|| anyhow::anyhow!("Anthropic API key required"))?;
            Ok(Box::new(AnthropicProvider::new(config, key)))
        }

        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
    }
}

/// Auto-select best available provider
pub fn auto_select_provider(
    config: &crate::config::Config,
) -> Result<Box<dyn LlmProvider>> {
    // Priority: CLI tools > Ollama > API providers

    // 1. Try Claude CLI
    if ClaudeCliProvider::is_installed() {
        return Ok(Box::new(ClaudeCliProvider::new()));
    }

    // 2. Try Codex CLI
    if CodexCliProvider::is_installed() {
        return Ok(Box::new(CodexCliProvider::new()));
    }

    // 3. Try Gemini CLI
    if GeminiCliProvider::is_installed() {
        return Ok(Box::new(GeminiCliProvider::new()));
    }

    // 4. Try Ollama (local)
    if OllamaProvider::is_running() {
        if let Some(ollama_config) = config.providers.get("ollama") {
            return Ok(Box::new(OllamaProvider::new(ollama_config.clone())));
        }
    }

    // 5. Try API providers with keys
    if let Some(api_key) = config.resolve_api_key("anthropic") {
        if let Some(anthropic_config) = config.providers.get("anthropic") {
            return Ok(Box::new(AnthropicProvider::new(
                anthropic_config.clone(),
                api_key,
            )));
        }
    }

    if let Some(api_key) = config.resolve_api_key("openai") {
        if let Some(openai_config) = config.providers.get("openai") {
            return Ok(Box::new(OpenAiProvider::new(openai_config.clone(), api_key)));
        }
    }

    Err(anyhow::anyhow!(
        "No LLM providers available.\n\n\
        Install one of these CLI tools:\n\
        • claude (Claude Code CLI)\n\
        • codex (OpenAI Codex CLI)\n\
        • gemini (Gemini CLI)\n\n\
        Or start Ollama:\n\
        • ollama serve"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_providers() {
        let available = detect_available_providers();
        // Should return a list (may be empty depending on system)
        assert!(available.len() >= 0);
    }

    #[test]
    fn test_create_claude_cli() {
        let config = ProviderConfig {
            endpoint: String::new(),
            api_key: String::new(),
            model: String::new(),
            enabled: true,
            max_tokens: None,
            temperature: None,
        };
        let provider = create_provider("claude-cli", config, None);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "Claude");
    }

    #[test]
    fn test_create_codex_cli() {
        let config = ProviderConfig {
            endpoint: String::new(),
            api_key: String::new(),
            model: String::new(),
            enabled: true,
            max_tokens: None,
            temperature: None,
        };
        let provider = create_provider("codex-cli", config, None);
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "Codex");
    }
}
