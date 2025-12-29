//! Configuration management for WorkyTerm

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM provider configurations
    pub providers: HashMap<String, ProviderConfig>,

    /// Default provider to use
    pub default_provider: String,

    /// Council mode settings
    pub council: CouncilConfig,

    /// UI preferences
    pub ui: UiConfig,

    /// Output settings
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API endpoint URL
    pub endpoint: String,

    /// API key (can be env var reference like $OPENAI_API_KEY)
    pub api_key: String,

    /// Model to use
    pub model: String,

    /// Whether this provider is enabled
    pub enabled: bool,

    /// Max tokens for responses
    pub max_tokens: Option<u32>,

    /// Temperature setting
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilConfig {
    /// Enable multi-LLM deliberation
    pub enabled: bool,

    /// Providers to include in council
    pub members: Vec<String>,

    /// Number of deliberation rounds
    pub rounds: u32,

    /// Consensus threshold (0.0 - 1.0)
    pub consensus_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Animation speed (frames per second)
    pub animation_fps: u32,

    /// Show worker thought bubbles
    pub show_thoughts: bool,

    /// Color theme
    pub theme: String,

    /// Worker personality names
    pub worker_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output directory
    pub directory: PathBuf,

    /// Auto-save outputs
    pub auto_save: bool,

    /// Output format preference
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();

        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                endpoint: "http://localhost:11434".to_string(),
                api_key: String::new(),
                model: "llama3.2".to_string(),
                enabled: true,
                max_tokens: Some(4096),
                temperature: Some(0.7),
            },
        );

        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: "$OPENAI_API_KEY".to_string(),
                model: "gpt-4o-mini".to_string(),
                enabled: false,
                max_tokens: Some(4096),
                temperature: Some(0.7),
            },
        );

        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                endpoint: "https://api.anthropic.com/v1".to_string(),
                api_key: "$ANTHROPIC_API_KEY".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                enabled: false,
                max_tokens: Some(4096),
                temperature: Some(0.7),
            },
        );

        Self {
            providers,
            default_provider: "ollama".to_string(),
            council: CouncilConfig {
                enabled: false,
                members: vec!["ollama".to_string()],
                rounds: 2,
                consensus_threshold: 0.7,
            },
            ui: UiConfig {
                animation_fps: 10,
                show_thoughts: true,
                theme: "default".to_string(),
                worker_names: vec![
                    "Pixel".to_string(),
                    "Byte".to_string(),
                    "Nova".to_string(),
                    "Chip".to_string(),
                    "Luna".to_string(),
                ],
            },
            output: OutputConfig {
                directory: dirs::document_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("WorkyTerm"),
                auto_save: true,
                format: "markdown".to_string(),
            },
        }
    }
}

impl Config {
    /// Load config from file or create default
    pub fn load(path: Option<&str>) -> Result<Self> {
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => Self::default_path(),
        };

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save(&config_path)?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get default config path
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("workyterm")
            .join("config.toml")
    }

    /// Resolve API key from config (handles env var references)
    pub fn resolve_api_key(&self, provider: &str) -> Option<String> {
        self.providers.get(provider).and_then(|p| {
            if p.api_key.starts_with('$') {
                std::env::var(&p.api_key[1..]).ok()
            } else if p.api_key.is_empty() {
                None
            } else {
                Some(p.api_key.clone())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.default_provider, "ollama");
        assert!(config.providers.contains_key("ollama"));
        assert!(config.providers.contains_key("openai"));
        assert!(config.providers.contains_key("anthropic"));

        // Ollama should be enabled by default
        assert!(config.providers.get("ollama").unwrap().enabled);

        // OpenAI and Anthropic should be disabled by default
        assert!(!config.providers.get("openai").unwrap().enabled);
        assert!(!config.providers.get("anthropic").unwrap().enabled);
    }

    #[test]
    fn test_worker_names() {
        let config = Config::default();

        assert_eq!(config.ui.worker_names.len(), 5);
        assert!(config.ui.worker_names.contains(&"Pixel".to_string()));
        assert!(config.ui.worker_names.contains(&"Byte".to_string()));
        assert!(config.ui.worker_names.contains(&"Nova".to_string()));
        assert!(config.ui.worker_names.contains(&"Chip".to_string()));
        assert!(config.ui.worker_names.contains(&"Luna".to_string()));
    }

    #[test]
    fn test_council_config() {
        let config = Config::default();

        assert!(!config.council.enabled);
        assert_eq!(config.council.rounds, 2);
        assert!((config.council.consensus_threshold - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_resolve_api_key_empty() {
        let config = Config::default();

        // Ollama has empty API key
        assert!(config.resolve_api_key("ollama").is_none());
    }

    #[test]
    fn test_resolve_api_key_env_var() {
        let config = Config::default();

        // Set test env var
        std::env::set_var("TEST_API_KEY", "test_value");

        let mut test_config = config.clone();
        test_config.providers.get_mut("ollama").unwrap().api_key = "$TEST_API_KEY".to_string();

        assert_eq!(
            test_config.resolve_api_key("ollama"),
            Some("test_value".to_string())
        );

        std::env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_resolve_api_key_direct() {
        let mut config = Config::default();

        config.providers.get_mut("ollama").unwrap().api_key = "direct_key".to_string();

        assert_eq!(
            config.resolve_api_key("ollama"),
            Some("direct_key".to_string())
        );
    }

    #[test]
    fn test_output_config() {
        let config = Config::default();

        assert!(config.output.auto_save);
        assert_eq!(config.output.format, "markdown");
    }

    #[test]
    fn test_provider_config() {
        let config = Config::default();
        let ollama = config.providers.get("ollama").unwrap();

        assert_eq!(ollama.endpoint, "http://localhost:11434");
        assert_eq!(ollama.model, "llama3.2");
        assert_eq!(ollama.max_tokens, Some(4096));
        assert!((ollama.temperature.unwrap() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();

        // Should contain key sections
        assert!(toml_str.contains("[providers"));
        assert!(toml_str.contains("default_provider"));
        assert!(toml_str.contains("[council]"));
        assert!(toml_str.contains("[ui]"));
        assert!(toml_str.contains("[output]"));
    }
}
