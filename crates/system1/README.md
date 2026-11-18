# system1

Ollama-backed **System 1** decision layer for the Meridian trading app.

The crate is intentionally independent from `crates/bot`. It accepts a
point-in-time market/account snapshot, asks a locally served small model for a
single action, and returns an order-ready decision after deterministic risk
guardrails. The desktop app (or a future bot loop) can depend on it without
pulling in Axum or the mock engine.

Default model: **`tev1:4b`** at `http://127.0.0.1:11434`.

## Why a separate crate?

- **Fast loop, small context.** The model sees compact indicators plus the last
  N candles, not the whole order history.
- **Deterministic safety.** The model proposes; `validate` disposes. Spread,
  freshness, confidence, sizing, stops, shorting, and exposure limits are
  enforced in Rust.
- **Testable.** `System1<B: ChatBackend>` accepts a fake backend, so retry,
  repair, and guardrail behavior is covered without Ollama running.
- **Reusable.** The same logic can serve the Tauri UI, the Axum engine, or a
  headless paper-trading worker.

## Flow

```text
market/account state
        │
        ▼
MarketFeatures::compute   (SMA/EMA/MACD/RSI/ATR/vol/range/book imbalance)
        │
        ▼
prompt (system policy + JSON snapshot) ──► Ollama /api/chat (structured JSON)
        │
        ▼
parse_decision            (fences, prose, trailing commas, numeric strings)
        │
        ▼
validate                  (risk guardrails, sizing, stops)
        │
        ▼
DecisionOutcome { decision, proposal, features, report, source }
```

If the model returns unusable JSON, the engine makes one repair call. If that
also fails, it returns a **defensive hold** (`DecisionSource::FallbackHold`) so
an automated loop keeps running. Transport errors are returned as `Err`;
`decide_or_hold` converts them to a hold.

## Quick start

```rust
use system1::{Action, System1, TradingContext};

let system = System1::from_env()?;          // model defaults to tev1:4b
system.ensure_model().await?;               // fails clearly if `ollama pull` is needed

let outcome = system.decide(&context).await?;
match outcome.decision.action {
    Action::Buy | Action::Sell | Action::Close => execute(outcome.decision),
    Action::Hold => log(outcome.decision.rationale),
}
```

`TradingContext` owns the snapshot. `RiskPolicy` lives inside it, so the demo
UI can use the same engine with different guardrails per account.

## Configuration

`OllamaConfig::from_env()` reads:

| Variable | Default | Purpose |
|----------|---------|---------|
| `SYSTEM1_OLLAMA_URL` or `OLLAMA_HOST` | `http://127.0.0.1:11434` | Ollama base URL. A bare `host:port` gets `http://`. |
| `SYSTEM1_MODEL` | `tev1:4b` | Model tag. |
| `SYSTEM1_TIMEOUT_SECS` | `45` | Per-request timeout (1–600). |
| `SYSTEM1_TEMPERATURE` | `0.1` | Sampling temperature (0–2). |
| `SYSTEM1_SEED` | `7` | Fixed seed for more repeatable decisions. |

`OllamaConfig` also exposes `top_p`, `num_predict`, `num_ctx`, `keep_alive`, and
`structured_output`. The client uses Ollama's `format` JSON schema; if a model
rejects the schema with HTTP 400, it retries once without it and still parses
the response strictly.

## What the model sees

- Quote: bid/ask/last, mid, spread in bps, 24h high/low/volume/change.
- Indicators: returns (1/5/20), SMA20/50, EMA12/26, MACD/signal/histogram,
  RSI14, ATR14 and ATR%, 20-bar realized volatility, volume ratio, range
  position, support/resistance, price vs SMAs, trend label.
- Compact JSON snapshot: last 40 valid OHLCV bars, top 8 book levels per side
  when supplied, account, position, open orders, recent fills, cooldown, caller
  notes, risk limits, and the all-in cost floor.
- Output schema: `action`, `confidence`, optional `quantity`/`notional`,
  `order_type`, `limit_price`, `stop_loss`, `take_profit`, `time_horizon_minutes`,
  `rationale`, `invalidation`, `risk_flags`.

The system prompt requires a held position when data is stale, the session is
closed, candles are insufficient, spread is too wide, or the daily loss limit is
hit. It also tells the model to prefer hold and to size from the stop distance.

## Guardrails

All percentages in `RiskPolicy` are **fractions of equity** (`0.05` = 5 %), not
percent units.

| Guard | Default | Behavior |
|-------|---------|----------|
| `max_order_notional_pct` | `0.05` | Caps a single order; downsizes. |
| `max_position_notional_pct` | `0.20` | Caps post-fill position; downsizes or blocks increases. |
| `max_gross_exposure_pct` | `1.00` | Caps total account exposure when supplied. |
| `max_risk_per_trade_pct` | `0.01` | Sizes quantity from entry-to-stop risk. |
| `max_daily_loss_pct` | `0.03` | Blocks new risk; `close` still allowed. |
| `min_confidence` | `0.60` | Blocks entries below the threshold. |
| `max_spread_bps` | `20` | No new entries in a wide market. |
| `max_limit_deviation_bps` | `75` | Rejects far-away limit prices. |
| `require_stop_loss` | `true` | Derives a stop from ATR if the model omits one. |
| `min/max_stop_distance_bps` | `15 / 1200` | Rejects stops inside noise or absurdly far. |
| `allow_short` | `false` | Sells are capped to an existing long. |
| `allow_margin` | `false` | Buys are capped by cash. |
| `quantity_step` / `min_quantity` / `max_quantity` | `0` | Exchange sizing rules. |
| `price_tick` | `0` | Rounds limit/stop/target prices. |
| `min_seconds_between_trades` | `30` | Cooldown from `seconds_since_last_trade`. |

`close` bypasses most entry gates because reducing risk should remain possible,
but it still needs a valid quote, respects limit-price deviation, and is capped
to the current position.

Every adjustment is listed in `DecisionOutcome::report.adjustments`; warnings
(e.g. take-profit reward/risk below target) go in `warnings`. A blocked entry
comes back as `Hold` with `source = GuardrailHold` and the reason in
`blocked_reason`.

## Testing

```sh
cargo test -p system1          # 41 unit tests, no network
cargo clippy -p system1 --all-targets
```

With Ollama running:

```sh
ollama pull tev1:4b
cargo run -p system1 --example decide
cargo test -p system1 --test live_ollama -- --ignored
```

The example prints the feature snapshot result, latency, attempts, and the full
`DecisionOutcome` as JSON. It exits with a readable message if Ollama is down or
the model is missing.

## Integrating with the app

1. Translate the engine/UI state into `TradingContext` (prices in quote
   currency, signed position quantity, policy fractions).
2. Run `System1::decide_or_hold` on a blocking task or a dedicated worker.
3. If the action is `Buy`/`Sell`, submit the returned `quantity` (and limit price
   if `order_type = Limit`). If `Close`, flatten/reduce.
4. Persist the whole outcome for audit; it includes the proposal, features,
   adjustments, model, latency, and attempt count.

The crate does not place orders and does not know about `crates/bot` order
types. That translation belongs in the caller, which keeps the model layer
independent and prevents accidental coupling to the mock engine.

## Limitations

- Designed for a local, trusted Ollama instance. No authentication or TLS is
  configured by default (HTTPS URLs are accepted if the build has a TLS
  backend; the default features deliberately do not include one).
- Small models can be confidently wrong. The guardrails cap damage; they do not
  guarantee profit. Paper-trade and review outcomes before risking capital.
- Prompt/context length assumes a model with at least an 8k context window. The
  example caps raw candles at 40 to stay well inside it.
