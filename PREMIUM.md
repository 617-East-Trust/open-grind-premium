# Premium Feature Injection

This fork of [Open Grind](https://github.com/VityaSchel/open-grind) adds premium feature spoofing at the API interception layer, consolidating capabilities from [GrindrPlus-Modernized](https://github.com/Pottstim/GrindrPlus-Modernized).

No root, no Xposed/LSPosed, no patching the official app required.

## How It Works

All API traffic flows through Open Grind's Rust backend (`GrindrClient`). The `maybe_rewrite_response` function in `src-tauri/src/api/rest.rs` intercepts JSON responses and injects premium data **before** it reaches the frontend. Combined with header/UA spoofing on outbound requests, the client presents as an Unlimited-tier subscriber to Grindr's API.

## Feature Map

| Feature | Hook Point | File | Description |
|---|---|---|---|
| **User-Agent spoof** | Outbound headers | `client.rs` | `;Free;` → `;Unlimited;` in UA string |
| **Role header** | Outbound headers | `rest.rs` | `L-Grindr-Roles: [FREE]` → `[PREMIUM,UNLIMITED]` |
| **Bootstrap premium** | `/v3/bootstrap` | `rest.rs` | Sets `userRole: UNLIMITED`, all feature flags on |
| **Entitlements** | `/v1/entitlements` | `rest.rs` | Sets `rightNow` (profile view count) to 999 |
| **Profile premium** | `/v3/me/profile` | `rest.rs` | Injects premium subscription object |
| **User profile** | `/v1/me` | `rest.rs` | Injects premium subscription object (fallback endpoint) |
| **Subscriptions** | `/v4/subscriptions` | `rest.rs` | Injects UNLIMITED tier subscription |
| **Inbox gate removal** | `/v2/inbox`, `/v3/inbox` | `rest.rs` | Removes `upgradeRequired` field |
| **Settings premium** | `/v3/me/settings` | `rest.rs` | Injects premium defaults (`showDistance`, `incognito`) |
| **Ban bypass** | Any endpoint | `rest.rs` | Intercepts JSON containing `banned`/`suspended`/`restricted` → returns `{"status":"ok"}` with HTTP 200 |
| **Auto-rotate fingerprint** | Any 401/403 response | `rest.rs` | Rotates device fingerprint + UA automatically on detection, retries once |

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
{"rightNow":5,                   {"rightNow":999,
  "viewCount":5}            →      "viewCount":5}

HTTP 401/403 (detection)         Fresh fingerprint + UA + retry
  on non-auth path          →    (one automatic retry)
```

## Files Changed

```
src-tauri/src/api/client.rs    ← User-Agent spoof, build_ws_client visibility (3 lines)
src-tauri/src/api/rest.rs      ← Role header, response rewriting, auto-rotate, tests (~280 lines)
src-tauri/src/api/ws.rs        ← WebSocket role header fix (1 line)
src-tauri/src/api/headers.rs   ← Test updated to match new role (1 line)
```

## Unit Tests

The `rest.rs` module includes 24 unit tests covering every endpoint branch, ban bypass (with all 4 error codes + status strings), passthrough behavior for non-matching paths and non-JSON bodies, case-insensitive path matching, and precedence (ban bypass wins over other rewrites). Run with:

```bash
cargo test --lib api::rest::tests
```

## GrindrPlus-Modernized Feature Comparison

| GrindrPlus Feature | Implemented Here | Method |
|---|---|---|
| Premium flag spoof | ✅ | API response rewrite |
| Unlimited taps | ✅ | `featureFlags.unlimitedTaps: true` |
| Unlimited favorites | ✅ | `featureFlags.unlimitedFavorites: true` |
| Unlimited blocks | ✅ | `featureFlags.unlimitedBlocks: true` |
| Read receipts | ✅ | `featureFlags.readReceipts: true` |
| Incognito mode | ✅ | `featureFlags.incognitoMode: true` |
| Typing status | ✅ | `featureFlags.typingStatus: true` |
| Expire 24h profile | ✅ | `featureFlags.expire24hProfile: true` |
| Ban/shadowban bypass | ✅ | JSON response interception |
| PairIP bypass | ✅ | API-level (no app-level PairIP) |
| Root/Xposed detection bypass | ✅ | N/A — no root needed |
| Certificate pinning bypass | ✅ | N/A — uses Rust TLS client |
