use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::AppState;
use crate::handlers;

pub async fn run(state: Arc<AppState>, addr: SocketAddr) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = handlers::router(state).layer(cors);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    tracing::info!("trading-bot-logic server listening on {}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .expect("server error");
}
