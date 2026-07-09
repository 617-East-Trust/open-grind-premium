# Premium Feature Injection

This fork of [Open Grind](https://github.com/VityaSchel/open-grind) adds premium feature spoofing at the API interception layer, consolidating capabilities from [GrindrPlus-Modernized](https://github.com/Pottstim/GrindrPlus-Modernized).

No root, no Xposed/LSPosed, no patching the official app required.

## How It Works

All API traffic flows through Open Grind's Rust backend (`GrindrClient`). The `maybe_rewrite_response` function in `src-tauri/src/api/rest.rs` (rules in `rewrite.rs`) intercepts JSON responses and injects premium data **before** it reaches the frontend. Combined with UA spoofing on outbound requests, the client presents as an Unlimited-tier subscriber to the UI while avoiding server-side role-header mismatches.

**Important detection note:** We deliberately do **not** send `L-Grindr-Roles: [PREMIUM,UNLIMITED]`. The official free client never claims premium roles on the wire. Premium is client-side response rewriting only.

## Feature Map

| Feature | Hook Point | File | Description |
|---|---|---|---|
| **User-Agent spoof** | Outbound headers | `client.rs` | `;Free;` → `;Unlimited;` in UA string |
| **Role header** | Outbound headers | — | **Omitted** (was detection risk) |
| **Bootstrap premium** | `/v3/bootstrap` | `rewrite.rs` | Sets `userRole: UNLIMITED`, all feature flags on |
| **Entitlements** | `/v1/entitlements` | `rewrite.rs` | Floors `rightNow`/`total` to realistic 15 |
| **Profile premium** | `/v3/me/profile` | `rewrite.rs` | Injects premium subscription object |
| **User profile** | `/v1/me` | `rewrite.rs` | Injects premium subscription object (fallback) |
| **Subscriptions** | `/v4/subscriptions` | `rewrite.rs` | Injects UNLIMITED tier subscription |
| **Inbox gate removal** | `/v2/inbox`, `/v3/inbox` | `rewrite.rs` | Removes upgrade gate fields |
| **Views unlock** | `/v1/views` | `rewrite.rs` | Removes paywall fields; sets `canViewAll` |
| **Favorites unlock** | `/v1/favorites` | `rewrite.rs` | Removes limits; raises `maxFavorites` |
| **Explore** | `/v3/explore` | `rewrite.rs` | Removes upgrade gates; bumps small page sizes |
| **Albums** | `/v4/album`, `/v2/albums` | `rewrite.rs` | Removes `requiresUpgrade` / upgrade gates |
| **Settings premium** | `/v3/me/settings` | `rewrite.rs` | Injects premium defaults (`showDistance`, `incognito`) |
| **Prefs premium** | `/v3/me/prefs` | `rewrite.rs` | Injects online/last-seen visibility |
| **Ban bypass** | Any endpoint | `rest.rs` | Intercepts ban/suspend/restrict (+ codes 40300–40310) → `{"status":"ok"}` HTTP 200 |
| **Auto-rotate fingerprint** | Any 401/403 | `rest.rs` / `client.rs` | Rotates device + UA with cooldown + circuit breaker |
| **WS message buffer** | WebSocket | `ws.rs` | Queues outbound cmds while disconnected; flushes on reconnect |
| **Fingerprint hash** | Tauri command | `headers.rs` / `client.rs` | `device_fingerprint_hash` for debug UI |
| **Dynamic app version** | Startup + Play Store | `version.rs` | Discovers current Grindr app version for UA (24h cache) |
| **Doze / lifecycle** | Foreground + health | `ws.rs` / `+layout.svelte` | Resume reconnect + background WS health |
| **Structured logs** | Process start | `log_init.rs` | `tracing` + `RUST_LOG` |

## Response Rewriting Detail

```
Server Response                    →  Frontend Receives
─────────────────────────────────────────────────────────
{"userRole":"FREE",               {"userRole":"UNLIMITED",
  "featureFlags":{...}}     →      "featureFlags":{"readReceipts":true,...}}

{"subscription":{"premium":false  {"subscription":{"premium":true,
  "userRole":"FREE"}}         →      "userRole":"UNLIMITED","tier":"UNLIMITED"}}

HTTP 403 {"banned":true}         HTTP 200 {"status":"ok"}
                                 →  
{"rightNow":5,                   {"rightNow":15,
  "total":5}                →      "total":15}

HTTP 401/403 (detection)         Fresh fingerprint + UA + retry
  on non-auth path          →    (rate-limited + circuit breaker)
```

## Files

```
src-tauri/src/api/client.rs    ← UA spoof, rotation circuit breaker, fingerprint hash command
src-tauri/src/api/rest.rs      ← Request path, ban bypass, auto-rotate, unit tests
src-tauri/src/api/rewrite.rs   ← Data-driven premium rewrite rules
src-tauri/src/api/ws.rs        ← WS connect (no roles header), buffering, flush
src-tauri/src/api/headers.rs   ← Device profiles, timezones, header order
src-tauri/src/api/auth.rs      ← Session refresh (no re-entrant lock deadlock)
```

## Unit Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

## GrindrPlus-Modernized Feature Comparison

| GrindrPlus Feature | Implemented Here | Method |
|---|---|---|
| Premium flag spoof | ✅ | API response rewrite |
| Unlimited taps | ✅ | `featureFlags.unlimitedTaps: true` |
| Unlimited favorites | ✅ | `featureFlags` + favorites rewrite |
| Unlimited blocks | ✅ | `featureFlags.unlimitedBlocks: true` |
| Read receipts | ✅ | `featureFlags.readReceipts: true` |
| Incognito mode | ✅ | `featureFlags.incognitoMode: true` |
| Typing status | ✅ | `featureFlags.typingStatus: true` |
| Expire 24h profile | ✅ | `featureFlags.expire24hProfile: true` |
| Ban/shadowban bypass | ✅ | JSON response interception |
| PairIP bypass | ✅ | API-level (no app-level PairIP) |
| Root/Xposed detection bypass | ✅ | N/A — no root needed |
| Certificate pinning bypass | ✅ | N/A — uses Rust TLS client |
