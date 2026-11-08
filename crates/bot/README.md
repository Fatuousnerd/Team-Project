# trading-bot-logic

Rust logic layer for the team trading-bot project. Exposes a REST + WebSocket
API via [Axum](https://docs.rs/axum). The GUIs (GPUI desktop, React-Native /
GPUI-mobile, Vite-React web) consume this layer to render charts, accounts,
positions, and to place/cancel orders.

## Run

```sh
cargo run
# server listens on http://0.0.0.0:3001
```

## Mock markets

On startup the engine seeds mock symbols:

- Stocks: AAPL, TSLA, MSFT, GOOGL, AMZN, NVDA, META
- Forex: EURUSD, GBPUSD, USDJPY, AUDUSD, USDCAD, USDCHF
- Crypto: BTCUSDT, ETHUSDT, SOLUSDT, ADAUSDT, DOGEUSDT, XRPUSDT

A background task ticks prices every 500 ms (random walk), so chart endpoints
stream live(ish) data immediately.

## API

### Markets

| Method | Path | Description |
|--------|------|-------------|
| GET | `/markets` | Full market snapshot |
| GET | `/markets/:symbol` | Market data for one symbol |
| GET | `/ws/prices` | WebSocket price feed (snapshot / sec) |

### Accounts

| Method | Path | Description |
|--------|------|-------------|
| GET | `/accounts` | All accounts |
| GET | `/accounts/:id` | Single account |
| GET | `/accounts/:id/positions` | Positions for an account |

### Orders

| Method | Path | Description |
|--------|------|-------------|
| POST | `/orders` | Place an order |
| GET | `/orders` | All orders (query `?symbol=...`) |
| GET | `/orders/:id` | Single order |
| DELETE | `/orders/:id` | Cancel an order |

### Place order body

```json
{
  "account_id": "<uuid>",
  "symbol": "BTCUSDT",
  "side": "Buy",
  "type": "Limit",
  "price": 60000,
  "quantity": 0.1
}
```

`type` can be `Market`, `Limit`, `Stop`, or `StopLimit` (stop variants rejected
until implemented). `price` is required for `Limit`.

## Next steps for the team

1. Replace `market::MockMarket` with a real market-data feed (WebSocket/REST).
2. Wire `TradingEngine` to the real Rust execution layer / broker adapter.
3. Add auth/sessions on top of Axum.
4. Point the GPUI / React-Native / Vite-React frontends at `http://localhost:3001`.
