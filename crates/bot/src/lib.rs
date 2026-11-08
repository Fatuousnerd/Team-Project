//! Trading bot logic layer (Rust).
//!
//! Exposes a REST + WebSocket API via Axum. GUIs (GPUI, React-Native, Vite-React)
//! consume this layer to render and interact with market data.

pub mod error;
pub mod handlers;
pub mod market;
pub mod model;
pub mod server;
pub mod state;
pub mod types;

pub use state::AppState;
