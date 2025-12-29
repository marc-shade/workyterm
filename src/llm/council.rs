//! LLM Council - Multi-model deliberation system

use anyhow::Result;
use std::collections::HashMap;

use crate::config::Config;
use crate::llm::provider::{create_provider, LlmProvider};

/// Council of LLM providers that deliberate on tasks
pub struct Council {
    providers: Vec<Box<dyn LlmProvider>>,
    rounds: u32,
    consensus_threshold: f32,
    enabled: bool,
}

impl Council {
    pub fn new(config: &Config) -> Self {
        let mut providers: Vec<Box<dyn LlmProvider>> = Vec::new();

        if config.council.enabled {
            for member in &config.council.members {
                if let Some(provider_config) = config.providers.get(member) {
                    if provider_config.enabled {
                        let api_key = config.resolve_api_key(member);

                        match create_provider(member, provider_config.clone(), api_key) {
                            Ok(provider) => providers.push(provider),
                            Err(e) => {
                                eprintln!("Warning: Failed to create provider {}: {}", member, e);
                            }
                        }
                    }
                }
            }
        }

        // Fallback to default provider if no council members
        if providers.is_empty() {
            if let Some(provider_config) = config.providers.get(&config.default_provider) {
                let api_key = config.resolve_api_key(&config.default_provider);

                if let Ok(provider) = create_provider(
                    &config.default_provider,
                    provider_config.clone(),
                    api_key,
                ) {
                    providers.push(provider);
                }
            }
        }

        let enabled = config.council.enabled && providers.len() > 1;

        Self {
            providers,
            rounds: config.council.rounds,
            consensus_threshold: config.council.consensus_threshold,
            enabled,
        }
    }

    /// Process a task through the council
    pub async fn process(&self, task: &str) -> Result<String> {
        if self.providers.is_empty() {
            return Err(anyhow::anyhow!(
                "No LLM providers available. Check your configuration."
            ));
        }

        if !self.enabled || self.providers.len() == 1 {
            // Single provider mode
            return self.providers[0].generate(task).await;
        }

        // Multi-provider deliberation
        self.deliberate(task).await
    }

    /// Run multi-round deliberation
    async fn deliberate(&self, task: &str) -> Result<String> {
        let mut responses: HashMap<String, Vec<String>> = HashMap::new();
        let mut context = String::new();

        for round in 0..self.rounds {
            // Deliberation round {round+1}/{rounds}

            let prompt = if round == 0 {
                format!(
                    "Task: {}\n\nPlease provide your response to this task.",
                    task
                )
            } else {
                format!(
                    "Task: {}\n\n\
                    Previous responses from other council members:\n{}\n\n\
                    Please review the previous responses and provide your updated response. \
                    Consider the strengths of each approach and aim for consensus.",
                    task, context
                )
            };

            // Gather responses from all providers
            let mut round_responses = Vec::new();
            for provider in &self.providers {
                match provider.generate(&prompt).await {
                    Ok(response) => {
                        round_responses.push((provider.name().to_string(), response.clone()));
                        responses
                            .entry(provider.name().to_string())
                            .or_default()
                            .push(response);
                    }
                    Err(e) => {
                        eprintln!("Warning: Provider {} failed: {}", provider.name(), e);
                    }
                }
            }

            // Build context for next round
            context = round_responses
                .iter()
                .map(|(name, response)| {
                    format!(
                        "=== {} ===\n{}\n",
                        name,
                        truncate_response(response, 500)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Final synthesis
        self.synthesize(task, &responses).await
    }

    /// Synthesize final response from council deliberation
    async fn synthesize(
        &self,
        task: &str,
        responses: &HashMap<String, Vec<String>>,
    ) -> Result<String> {
        // Get the most recent response from each provider
        let final_responses: Vec<String> = responses
            .values()
            .filter_map(|v| v.last().cloned())
            .collect();

        if final_responses.is_empty() {
            return Err(anyhow::anyhow!("No responses from council members"));
        }

        if final_responses.len() == 1 {
            return Ok(final_responses[0].clone());
        }

        // Ask a provider to synthesize
        let synthesis_prompt = format!(
            "You are synthesizing responses from multiple AI council members.\n\n\
            Original task: {}\n\n\
            Council responses:\n{}\n\n\
            Please synthesize these responses into a single, cohesive answer that:\n\
            1. Incorporates the best ideas from each response\n\
            2. Resolves any contradictions\n\
            3. Maintains clarity and usefulness\n\n\
            Provide only the synthesized response, without meta-commentary.",
            task,
            final_responses
                .iter()
                .enumerate()
                .map(|(i, r)| format!("Response {}:\n{}\n", i + 1, truncate_response(r, 800)))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Use first available provider for synthesis
        self.providers[0].generate(&synthesis_prompt).await
    }
}

/// Truncate a response to a maximum length
fn truncate_response(response: &str, max_len: usize) -> &str {
    if response.len() <= max_len {
        response
    } else {
        &response[..max_len]
    }
}
