//! Ollama HTTP client and configuration.
//!
//! Only the endpoints needed by the decision engine are implemented:
//! `GET /api/tags` (health/model discovery) and `POST /api/chat`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::chat::{ChatBackend, ChatMessage, ChatResponse};
use crate::error::Error;

pub const DEFAULT_MODEL: &str = "tev1:4b";
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434";

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Ollama base URL, e.g. `http://127.0.0.1:11434`.
    pub base_url: String,
    /// Model tag, e.g. `tev1:4b`.
    pub model: String,
    /// End-to-end request timeout.
    pub request_timeout: Duration,
    /// TCP connect timeout.
    pub connect_timeout: Duration,
    /// Sampling temperature. Low values keep decisions reproducible.
    pub temperature: f64,
    pub top_p: f64,
    /// Fixed seed for repeatable sampling; `None` lets Ollama choose.
    pub seed: Option<i64>,
    /// Maximum generated tokens. Decisions are small.
    pub num_predict: u32,
    /// Optional context window override.
    pub num_ctx: Option<u32>,
    /// How long Ollama keeps the model loaded between calls.
    pub keep_alive: String,
    /// Use Ollama structured outputs (`format` JSON schema).
    pub structured_output: bool,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            request_timeout: Duration::from_secs(45),
            connect_timeout: Duration::from_secs(3),
            temperature: 0.1,
            top_p: 0.9,
            seed: Some(7),
            num_predict: 512,
            num_ctx: Some(8_192),
            keep_alive: "10m".to_string(),
            structured_output: true,
        }
    }
}

impl OllamaConfig {
    /// Read overrides from the environment.
    ///
    /// Supported variables: `SYSTEM1_OLLAMA_URL` (falls back to `OLLAMA_HOST`),
    /// `SYSTEM1_MODEL`, `SYSTEM1_TIMEOUT_SECS`, `SYSTEM1_TEMPERATURE`, and
    /// `SYSTEM1_SEED`.
    pub fn from_env() -> Result<Self, Error> {
        let mut config = Self::default();

        if let Ok(url) = std::env::var("SYSTEM1_OLLAMA_URL")
            .or_else(|_| std::env::var("OLLAMA_HOST"))
            .map(|value| value.trim().to_string())
            && !url.is_empty()
        {
            config.base_url = normalize_base_url(&url);
        }
        if let Ok(model) = std::env::var("SYSTEM1_MODEL")
            && !model.trim().is_empty()
        {
            config.model = model.trim().to_string();
        }
        if let Ok(value) = std::env::var("SYSTEM1_TIMEOUT_SECS") {
            match value.trim().parse::<u64>() {
                Ok(seconds) => config.request_timeout = Duration::from_secs(seconds.clamp(1, 600)),
                Err(_) => {
                    return Err(Error::Config(format!(
                        "SYSTEM1_TIMEOUT_SECS is not an integer: {value}"
                    )));
                }
            }
        }
        if let Ok(value) = std::env::var("SYSTEM1_TEMPERATURE") {
            match value.trim().parse::<f64>() {
                Ok(temperature) if temperature.is_finite() => {
                    config.temperature = temperature.clamp(0.0, 2.0);
                }
                _ => {
                    return Err(Error::Config(format!(
                        "SYSTEM1_TEMPERATURE is not a number: {value}"
                    )));
                }
            }
        }
        if let Ok(value) = std::env::var("SYSTEM1_SEED") {
            match value.trim().parse::<i64>() {
                Ok(seed) => config.seed = Some(seed),
                Err(_) => {
                    return Err(Error::Config(format!(
                        "SYSTEM1_SEED is not an integer: {value}"
                    )));
                }
            }
        }

        config.validate()?;
        Ok(config)
    }

    fn validate(&mut self) -> Result<(), Error> {
        if self.model.trim().is_empty() {
            return Err(Error::Config("model must not be empty".to_string()));
        }
        self.base_url = normalize_base_url(&self.base_url);
        let url = Url::parse(&self.base_url)
            .map_err(|error| Error::Config(format!("invalid Ollama URL: {error}")))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(Error::Config(
                "Ollama URL must use http:// or https://".to_string(),
            ));
        }
        if self.request_timeout.is_zero() {
            return Err(Error::Config(
                "request timeout must be positive".to_string(),
            ));
        }
        if !self.temperature.is_finite() || !(0.0..=2.0).contains(&self.temperature) {
            return Err(Error::Config(
                "temperature must be between 0 and 2".to_string(),
            ));
        }
        if !self.top_p.is_finite() || !(0.0..=1.0).contains(&self.top_p) {
            return Err(Error::Config("top_p must be between 0 and 1".to_string()));
        }
        Ok(())
    }
}

fn normalize_base_url(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };
    with_scheme.replace("://0.0.0.0", "://127.0.0.1")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub base_url: String,
    pub model: String,
    pub model_present: bool,
    pub models: Vec<String>,
}

pub struct OllamaClient {
    config: OllamaConfig,
    http: reqwest::Client,
    chat_url: String,
    tags_url: String,
    structured_output: AtomicBool,
}

impl std::fmt::Debug for OllamaClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OllamaClient")
            .field("base_url", &self.config.base_url)
            .field("model", &self.config.model)
            .field(
                "structured_output",
                &self.structured_output.load(Ordering::Relaxed),
            )
            .finish()
    }
}

impl OllamaClient {
    pub fn new(mut config: OllamaConfig) -> Result<Self, Error> {
        config.validate()?;

        let http = reqwest::Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .user_agent(concat!("system1/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| Error::Config(format!("failed to build HTTP client: {error}")))?;

        let base = config.base_url.trim_end_matches('/');
        let chat_url = format!("{base}/api/chat");
        let tags_url = format!("{base}/api/tags");
        let structured = config.structured_output;

        Ok(Self {
            config,
            http,
            chat_url,
            tags_url,
            structured_output: AtomicBool::new(structured),
        })
    }

    pub fn from_env() -> Result<Self, Error> {
        Self::new(OllamaConfig::from_env()?)
    }

    pub fn config(&self) -> &OllamaConfig {
        &self.config
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Check connectivity and whether the configured model is installed.
    pub async fn health(&self) -> Result<Health, Error> {
        let response = self
            .http
            .get(&self.tags_url)
            .send()
            .await
            .map_err(|error| self.transport_error(error))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| self.transport_error(error))?;

        if !status.is_success() {
            return Err(Error::HttpStatus {
                status: status.as_u16(),
                body: truncate(&body, 1_200),
            });
        }

        let parsed: TagsResponse = serde_json::from_str(&body).map_err(|error| {
            Error::InvalidResponse(format!("{error}; response began: {}", truncate(&body, 300)))
        })?;

        let models: Vec<String> = parsed
            .models
            .into_iter()
            .filter_map(|model| model.name.or(model.model))
            .collect();
        let model_present = models
            .iter()
            .any(|name| model_matches(name, &self.config.model));

        Ok(Health {
            base_url: self.config.base_url.clone(),
            model: self.config.model.clone(),
            model_present,
            models,
        })
    }

    /// Return the health report, or a clear error when the model is missing.
    pub async fn ensure_model(&self) -> Result<Health, Error> {
        let health = self.health().await?;
        if !health.model_present {
            return Err(Error::ModelMissing(self.config.model.clone()));
        }
        Ok(health)
    }

    /// Chat with the configured model. Falls back to prompt-only JSON if the
    /// server rejects structured outputs.
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        format: Option<Value>,
    ) -> Result<ChatResponse, Error> {
        let include_format = format.is_some() && self.structured_output.load(Ordering::Relaxed);

        let body = self.build_body(
            &messages,
            include_format.then_some(format.as_ref()).flatten(),
        );
        match self.send_chat(&body).await {
            Ok(response) => Ok(response),
            Err(Error::HttpStatus {
                status,
                body: error_body,
            }) if include_format && status == 400 && looks_like_format_error(&error_body) => {
                self.structured_output.store(false, Ordering::Relaxed);
                tracing::warn!(
                    model = %self.config.model,
                    "Ollama rejected structured output; retrying without the format schema"
                );
                let body = self.build_body(&messages, None);
                self.send_chat(&body).await
            }
            Err(error) => Err(error),
        }
    }

    fn build_body(&self, messages: &[ChatMessage], format: Option<&Value>) -> Value {
        let mut options = json!({
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "num_predict": self.config.num_predict,
        });
        if let Some(seed) = self.config.seed {
            options["seed"] = json!(seed);
        }
        if let Some(num_ctx) = self.config.num_ctx {
            options["num_ctx"] = json!(num_ctx);
        }

        let mut body = json!({
            "model": self.config.model,
            "messages": messages,
            "stream": false,
            "keep_alive": self.config.keep_alive,
            "options": options,
        });
        if let Some(schema) = format {
            body["format"] = schema.clone();
        }
        body
    }

    async fn send_chat(&self, body: &Value) -> Result<ChatResponse, Error> {
        let response = self
            .http
            .post(&self.chat_url)
            .json(body)
            .send()
            .await
            .map_err(|error| self.transport_error(error))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| self.transport_error(error))?;

        if !status.is_success() {
            return Err(Error::HttpStatus {
                status: status.as_u16(),
                body: truncate(&text, 1_200),
            });
        }

        let parsed: OllamaChatResponse = serde_json::from_str(&text).map_err(|error| {
            Error::InvalidResponse(format!("{error}; response began: {}", truncate(&text, 300)))
        })?;

        let content = parsed
            .message
            .map(|message| message.content)
            .unwrap_or_default();
        if content.trim().is_empty() {
            return Err(Error::EmptyResponse);
        }

        Ok(ChatResponse {
            content,
            done_reason: parsed.done_reason,
            eval_count: parsed.eval_count,
            prompt_eval_count: parsed.prompt_eval_count,
            total_duration_ns: parsed.total_duration,
        })
    }

    fn transport_error(&self, error: reqwest::Error) -> Error {
        if error.is_connect() || error.is_timeout() {
            Error::Unreachable {
                url: self.config.base_url.clone(),
                source: error,
            }
        } else {
            Error::Request(error)
        }
    }
}

#[async_trait]
impl ChatBackend for OllamaClient {
    fn model(&self) -> &str {
        &self.config.model
    }

    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        format: Option<Value>,
    ) -> Result<ChatResponse, Error> {
        self.chat_completion(messages, format).await
    }
}

fn model_matches(candidate: &str, wanted: &str) -> bool {
    fn normalize(name: &str) -> &str {
        name.trim().strip_suffix(":latest").unwrap_or(name.trim())
    }
    normalize(candidate).eq_ignore_ascii_case(normalize(wanted))
}

fn looks_like_format_error(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    ["format", "schema", "grammar", "json"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn truncate(text: &str, limit: usize) -> String {
    let mut output: String = text.chars().take(limit).collect();
    if text.chars().count() > limit {
        output.push('…');
    }
    output
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
struct TagModel {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    #[serde(default)]
    message: Option<OllamaMessage>,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    eval_count: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<u64>,
    #[serde(default)]
    total_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    #[serde(default)]
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_hosts() {
        assert_eq!(
            normalize_base_url("127.0.0.1:11434"),
            "http://127.0.0.1:11434"
        );
        assert_eq!(
            normalize_base_url("http://0.0.0.0:11434/"),
            "http://127.0.0.1:11434"
        );
    }

    #[test]
    fn matches_latest_suffix() {
        assert!(model_matches("tev1:4b", "tev1:4b"));
        assert!(model_matches("tev1:4b:latest", "tev1:4b"));
        assert!(!model_matches("llama3:8b", "tev1:4b"));
    }

    #[test]
    fn body_includes_schema_and_options() {
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let schema = json!({"type": "object"});
        let body = client.build_body(&[ChatMessage::user("hello")], Some(&schema));
        assert_eq!(body["model"], DEFAULT_MODEL);
        assert_eq!(body["stream"], false);
        assert_eq!(body["format"]["type"], "object");
        assert_eq!(body["options"]["temperature"], 0.1);
    }
}
