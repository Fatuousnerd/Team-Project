//! Minimal chat abstraction shared by the Ollama client and the engine.
//!
//! The trait exists so retry/repair/guardrail behavior can be tested without a
//! running Ollama instance. Production code uses [`crate::OllamaClient`].

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    #[serde(default)]
    pub done_reason: Option<String>,
    #[serde(default)]
    pub eval_count: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
    #[serde(default)]
    pub total_duration_ns: Option<u64>,
}

#[async_trait]
pub trait ChatBackend: Send + Sync {
    /// Model identifier used for logs and [`crate::DecisionOutcome::model`].
    fn model(&self) -> &str;

    /// Send a chat completion request. `format` is a JSON schema when the
    /// backend supports structured output.
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        format: Option<Value>,
    ) -> Result<ChatResponse, Error>;
}
