# Meridian — Desktop UI

Tauri v2 desktop shell for the team trading bot. React 19, TypeScript, Tailwind
CSS v4, and the shadcn `base-nova` component set.

This package is currently **UI-only**: it runs on seeded mock data so every
screen is clickable without the Rust service. The Axum API in
[`../crates/bot`](../crates/bot) is the intended data source later.

## Run

```sh
bun install
bun dev        # browser at http://localhost:1420
bun start      # Tauri window (compiles src-tauri)
bun typescript # tsc + production build
```

## Design system

| Token      | Value                                                     |
| ---------- | --------------------------------------------------------- |
| Base       | Ink — near-black neutrals with a faint green undertone     |
| Accent     | Emerald — primary actions, gains, live state               |
| Negative   | Rose — losses, sells, destructive intent                   |
| Contrast   | Flat surfaces, hairline borders, **no gradients anywhere** |
| Type       | Inter Variable, tabular numerals for every price/P&L value |
| Themes     | Dark first, matching light palette, persisted locally      |

All colors live as CSS custom properties in `src/App.css` and are exposed to
Tailwind through `@theme inline`. Edit tokens there, not in components.

## Routes

| Path                | Screen                                                                   |
| ------------------- | ------------------------------------------------------------------------ |
| `/`                 | Dashboard — equity curve, allocation, positions, watchlist, activity     |
| `/markets`          | Sortable/filterable instrument table with sparklines                     |
| `/markets/:symbol`  | Chart + order ticket + order book + time & sales                         |
| `/portfolio`        | Positions, allocation, account switching                                 |
| `/orders`           | Order history with inline cancel                                         |
| `/strategies`       | Bot cards, pause/resume, create-draft dialog                             |
| `/settings`         | Appearance, trading defaults, feed info, notifications                   |

`⌘K` / `Ctrl+K` opens the command palette (pages + all symbols). The sidebar
collapses to icons (`⌘B`). Toasts confirm every paper action.

## Source layout

```text
src/
├─ components/app/     # App-specific composed UI (tables, ticket, charts…)
├─ components/ui/      # shadcn primitives (generated, do not hand-edit)
├─ hooks/              # useMarketStream, useTheme
├─ layouts/AppShell.tsx
├─ lib/mock/           # Types, seeded fixtures, Zustand store, formatting
├─ lib/router.tsx      # Hash router (safer for packaged Tauri builds)
└─ pages/              # One file per route
```

## Mock data layer

- `lib/mock/data.ts` — seeded instruments, quotes, candles, equity curves,
  positions, orders, strategies. Deterministic per symbol/timeframe.
- `lib/mock/market-store.ts` — Zustand store. Ticks prices every 1.5s, places
  and cancels paper orders, toggles strategies, records activity.
- `lib/mock/format.ts` — currency/percent/price formatting, market-aware
  decimals.

To move to the real API:

1. Replace the tick loop with the `/ws/prices` feed and `GET /markets`.
2. Swap `placeOrder` / `cancelOrder` for `POST /orders` and `DELETE /orders/:id`.
3. Keep the selectors in `derive.ts`; they only depend on the `Quote`/`Position`
   shapes, which mirror `crates/bot/src/types.rs`.

## Notes

- Supabase auth from the original starter was removed — the app opens straight
  to the dashboard. Re-add a login gate (and `@supabase/supabase-js`) only when
  the auth flow is actually needed.
- Settings switches are presentation-only placeholders; wire them to a config
  store or Tauri store plugin when needed.
- The starter's `components/ai-elements` folder, unused auth helpers, and their
  AI, Supabase, and node-flow packages have been removed. Some unused shadcn
  primitives remain in `components/ui`, so their dependencies (carousel, drawer,
  calendar, OTP, resizable panels) stay until those components are dropped.
