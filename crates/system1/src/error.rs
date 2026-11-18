//! Error types for configuration, transport, and model output failures.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("could not reach Ollama at {url}: {source}")]
    Unreachable {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Ollama returned HTTP {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("model `{0}` is not installed in Ollama (run `ollama pull {0}`)")]
    ModelMissing(String),

    #[error("Ollama returned an empty response")]
    EmptyResponse,

    #[error("Ollama returned an invalid response: {0}")]
    InvalidResponse(String),

    #[error("could not parse decision JSON: {0}")]
    Parse(String),

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
}

impl Error {
    /// Transport hiccups and transient server statuses are worth retrying.
    /// Validation/parse errors are handled separately by the engine.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Unreachable { .. } => true,
            Self::HttpStatus { status, .. } => *status == 429 || *status >= 500,
            Self::Request(error) => error.is_timeout() || error.is_connect(),
            _ => false,
        }
    }
}
