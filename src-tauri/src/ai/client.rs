use reqwest::Client;
use serde::{Deserialize, Serialize};

// ── Shared message type ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiMessage {
    pub role: String,
    pub content: String,
}

// ── OpenAI chat/completions format ──────────────────────────────────────────

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<ApiMessage>,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

// ── Anthropic v1/messages format ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ApiMessage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

// ── Model list (same shape for both providers) ───────────────────────────────

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

// ── Format detection ─────────────────────────────────────────────────────────

enum ApiFormat { OpenAI, Anthropic }

impl ApiFormat {
    fn resolve(format_str: &str, endpoint: &str) -> Self {
        match format_str {
            "anthropic" => ApiFormat::Anthropic,
            "openai"    => ApiFormat::OpenAI,
            _           => {
                // auto-detect from URL
                if endpoint.contains("anthropic.com") {
                    ApiFormat::Anthropic
                } else {
                    ApiFormat::OpenAI
                }
            }
        }
    }
}

// ── Client ───────────────────────────────────────────────────────────────────

pub struct AiClient {
    client:   Client,
    api_key:  String,
    model:    String,
    endpoint: String, // base URL, e.g. "https://api.anthropic.com"
    format:   String, // "auto" | "openai" | "anthropic"
}

impl AiClient {
    pub fn new(api_key: String, endpoint: String, model: String, format: String) -> Self {
        Self { client: Client::new(), api_key, model, endpoint, format }
    }

    fn base(&self) -> String {
        self.endpoint.trim_end_matches('/').to_string()
    }

    pub async fn send(
        &self,
        system_prompt: &str,
        history: &[ApiMessage],
        user_message: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        match ApiFormat::resolve(&self.format, &self.endpoint) {
            ApiFormat::Anthropic => self.send_anthropic(system_prompt, history, user_message, max_tokens).await,
            ApiFormat::OpenAI    => self.send_openai(system_prompt, history, user_message, max_tokens).await,
        }
    }

    async fn send_anthropic(
        &self,
        system_prompt: &str,
        history: &[ApiMessage],
        user_message: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        let mut messages = history.to_vec();
        messages.push(ApiMessage { role: "user".to_string(), content: user_message.to_string() });

        let body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens,
            system: system_prompt.to_string(),
            messages,
        };

        let resp = self.client
            .post(format!("{}/v1/messages", self.base()))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, text));
        }

        let parsed: AnthropicResponse = resp.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        parsed.content.into_iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text)
            .ok_or_else(|| "Empty response from API".to_string())
    }

    async fn send_openai(
        &self,
        system_prompt: &str,
        history: &[ApiMessage],
        user_message: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        let mut messages = vec![
            ApiMessage { role: "system".to_string(), content: system_prompt.to_string() },
        ];
        messages.extend_from_slice(history);
        messages.push(ApiMessage { role: "user".to_string(), content: user_message.to_string() });

        let body = OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens,
        };

        let resp = self.client
            .post(format!("{}/v1/chat/completions", self.base()))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, text));
        }

        let parsed: OpenAIResponse = resp.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        parsed.choices.into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| "Empty response from API".to_string())
    }

    pub async fn fetch_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/v1/models", self.base());

        let req = match ApiFormat::resolve(&self.format, &self.endpoint) {
            ApiFormat::Anthropic => self.client.get(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01"),
            ApiFormat::OpenAI => self.client.get(&url)
                .header("Authorization", format!("Bearer {}", self.api_key)),
        };

        let resp = req.send().await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Models API error {}: {}", status, text));
        }

        let parsed: ModelsResponse = resp.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        let mut ids: Vec<String> = parsed.data.into_iter().map(|m| m.id).collect();
        ids.sort();
        Ok(ids)
    }
}
