use axum::{
    extract::{Path, Query, State},
    extract::ws::{Message, WebSocketUpgrade},
    response::{Json, Response},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppResult;
use crate::state::AppState;
use crate::types::*;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/markets", get(list_markets))
        .route("/markets/:symbol", get(get_market))
        .route("/accounts", get(list_accounts))
        .route("/accounts/:id", get(get_account))
        .route("/accounts/:id/positions", get(get_positions))
        .route("/orders", post(place_order).get(list_orders))
        .route("/orders/:id", get(get_order).delete(cancel_order))
        .route("/ws/prices", get(ws_handler))
        .with_state(state)
}

async fn health(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "trading-bot-logic" }))
}

async fn list_markets(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<MarketData>>> {
    let market = state.market.read().unwrap();
    Ok(Json(market.list()))
}

async fn get_market(
    Path(symbol): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<MarketData>> {
    let market = state.market.read().unwrap();
    let symbol = Symbol::new(symbol);
    let data = market
        .get(&symbol)
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Market {} not found", symbol.as_str())))?;
    Ok(Json(data.clone()))
}

async fn list_accounts(State(state): State<Arc<AppState>>) -> Json<Vec<Account>> {
    Json(state.engine.list_accounts())
}

async fn get_account(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Account>> {
    let id = Uuid::parse_str(&id).map_err(|_| {
        crate::error::AppError::BadRequest(format!("Invalid account id: {id}"))
    })?;
    let account = state.engine.get_account(id)?;
    Ok(Json(account))
}

async fn get_positions(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<Position>>> {
    let id = Uuid::parse_str(&id).map_err(|_| {
        crate::error::AppError::BadRequest(format!("Invalid account id: {id}"))
    })?;
    let account = state.engine.get_account(id)?;
    Ok(Json(account.positions))
}

#[derive(Deserialize)]
pub struct ListOrdersQuery {
    pub symbol: Option<String>,
}

async fn place_order(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PlaceOrderRequest>,
) -> AppResult<Json<Order>> {
    let order = state.engine.place_order(req)?;
    Ok(Json(order))
}

async fn list_orders(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListOrdersQuery>,
) -> Json<Vec<Order>> {
    let orders = state.engine.list_orders();
    let orders = match q.symbol {
        Some(s) => orders
            .into_iter()
            .filter(|o| o.symbol.as_str() == s)
            .collect(),
        None => orders,
    };
    Json(orders)
}

async fn get_order(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Order>> {
    let id = Uuid::parse_str(&id).map_err(|_| {
        crate::error::AppError::BadRequest(format!("Invalid order id: {id}"))
    })?;
    let order = state.engine.get_order(&id)?;
    Ok(Json(order))
}

async fn cancel_order(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Order>> {
    let id = Uuid::parse_str(&id).map_err(|_| {
        crate::error::AppError::BadRequest(format!("Invalid order id: {id}"))
    })?;
    let order = state.engine.cancel_order(&id)?;
    Ok(Json(order))
}

/// WebSocket price feed. Broadcasts the full mock market snapshot once per
/// second until the socket closes.
///
/// axum 0.7 uses `WebSocketUpgrade` as the extractor and `on_upgrade` to spawn
/// the socket handler, so this works with the shared `Arc<AppState>` state.
async fn ws_handler(
    State(state): State<Arc<AppState>>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let market = state.market.clone();
    upgrade.on_upgrade(move |socket| async move {
        let (mut sender, _receiver) = socket.split();
        let _ = sender
            .send(Message::Text(
                serde_json::to_string(&serde_json::json!({
                    "type": "subscribed",
                    "channel": "prices"
                }))
                .unwrap(),
            ))
            .await;

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let payload = serde_json::json!({
                "type": "prices",
                "data": market.read().unwrap().list(),
                "ts": Utc::now()
            });
            match serde_json::to_string(&payload) {
                Ok(txt) => {
                    if sender.send(Message::Text(txt)).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    })
}
