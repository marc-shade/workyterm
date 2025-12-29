//! LLM Provider implementations

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::ProviderConfig;

/// Generic LLM provider trait
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String>;
    fn name(&self) -> &str;
}

/// Ollama provider (local)
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
}

/// OpenAI-compatible provider
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
}

/// Anthropic Claude provider
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
}

/// Factory function to create provider from config
pub fn create_provider(
    provider_name: &str,
    config: ProviderConfig,
    api_key: Option<String>,
) -> Result<Box<dyn LlmProvider>> {
    match provider_name {
        "ollama" => Ok(Box::new(OllamaProvider::new(config))),
        "openai" => {
            let key = api_key.ok_or_else(|| anyhow::anyhow!("OpenAI API key required"))?;
            Ok(Box::new(OpenAiProvider::new(config, key)))
        }
        "anthropic" => {
            let key = api_key.ok_or_else(|| anyhow::anyhow!("Anthropic API key required"))?;
            Ok(Box::new(AnthropicProvider::new(config, key)))
        }
        _ => Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
    }
}
