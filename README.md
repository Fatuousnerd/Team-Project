# Team-Project

Trading-bot team project.

- `crates/bot` — Rust trading engine and Axum API (markets, accounts, orders, WebSocket prices).
- `crates/system1` — Ollama-backed System 1 decision layer (local `tev1:4b` by default). See [crates/system1/README.md](crates/system1/README.md).
- `desktop` — Tauri v2 desktop UI (React, Tailwind, shadcn). See [desktop/README.md](desktop/README.md).
- `mobile` — mobile client workspace (placeholder).

## Desktop UI

```sh
cd desktop
bun install
bun dev    # browser preview
bun start  # Tauri window
```
