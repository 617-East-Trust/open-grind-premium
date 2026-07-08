# Open Grind Premium — Code Audit

**Date:** 2026-07-08  
**Scope:** Full repository analysis — Rust backend (`src-tauri/src/`), SvelteKit frontend (`src/`), CI, config.  
**Version:** `0.2.0-alpha.1` (Cargo.toml) / `0.1.0-alpha.9` (tauri.conf.json)  
**Commit:** `0e6015f` — fix: add missing version.rs for svelte.config.js build

---

## 1. Architecture Overview

| Layer | Technology | Role |
|:------|:-----------|:-----|
| Frontend | Svelte 5 + SvelteKit 2 + Tailwind CSS 4 + Bits-UI | SPA with real-time UI |
| Desktop Shell | Tauri v2 | Native window, plugins, bridge |
| HTTP Client | `wreq` v5.3 w/ `OkHttp4_12` emulation | TLS/JA4 fingerprint impersonation |
| WS Client | `wreq` WebSocket (HTTP/1.1 only) | Real-time messaging |
| API Bridge | MsgPack → Base64 → `tauri::invoke` | Frontend-to-Rust IPC |
| Auth Storage | Platform-native keyring + file fallback | Multi-account credential persistence |
| State | Svelte 5 runes + Zod schemas + `OnceLock<Arc<GrindrClient>>` | Reactive UI + thread-safe backend |

**Data flow:** SvelteKit page → `fetchRest()` → MsgPack encode → Base64 → `invoke("request", {payload})` → Rust `GrindrClient::request_raw()` → `wreq` HTTP → Grindr API → `maybe_rewrite_response()` → MsgPack encode → Base64 → frontend parse.

---

## 2. Strengths

### 2.1 Premium Injection (`rest.rs:maybe_rewrite_response`)

The response-rewriting pipeline is the core value proposition and it's **well-executed**:

- **7 rewrite targets:** `/v3/bootstrap`, `/v1/entitlements`, `/v3/me/profile`, `/v4/subscriptions`, `/v1/me`, `/v2|v3/inbox`, `/v3/me/settings`
- **Bootstrap injection:** Sets `userRole`/`subscriptionTier` to `"UNLIMITED"` and injects 11 premium feature flags (`readReceipts`, `tapAndGo`, `unlimitedTaps`, `incognitoMode`, etc.) while preserving server-side A/B flags via `unwrap_or_default()`
- **Ban bypass:** Detects ban/suspended/restricted status via JSON field inspection (both `status` string and `code` numeric ranges 40300–40303), rewrites to `{"status":"ok"}` with HTTP 200
- **Precedence:** Ban bypass fires *before* path-specific rewrites, preventing injection on banned accounts
- **Case-insensitive:** `path.to_lowercase()` matching
- **Non-JSON passthrough:** Gracefully returns raw bytes if parsing fails

**Test coverage:** 24 unit tests covering all rewrite paths, ban bypass variants, passthrough, precedence, case-insensitivity, and edge cases. This is exceptional for a reverse-engineering project.

### 2.2 Fingerprint Impersonation (`headers.rs`, `client.rs`)

- **80+ real device profiles** spanning Google Pixel (6–9 Pro XL), Samsung Galaxy S/A/M/Z series, Xiaomi/Redmi/POCO, OnePlus, Motorola, Sony, Nothing, Asus, Realme, Oppo, Vivo, Tecno, Infinix, Huawei, Honor — excellent global market coverage
- **Android version randomization** within each device's supported range via `saturating_sub`
- **Timezone rotation** from 50 real timezones across Europe, Americas, Asia-Pacific
- **Header ordering** matches the documented Grindr security header spec (Authorization → L-Time-Zone → L-Grindr-Roles → L-Device-Info → Accept → User-Agent → L-Locale → Accept-Language → Accept-Encoding)
- **TLS fingerprint:** `OkHttp4_12` wreq emulation profile provides authentic Android OkHttp JA3/JA4 fingerprint verified against Cloudflare expectations
- **Device persistence:** Stored in platform keyring via MsgPack serialization, survives app restarts

### 2.3 Auto-Rotate on Detection (`rest.rs:request_raw_internal`)

When a 401 or 403 hits a non-auth path, the client:
1. Generates a fresh `DeviceInfo` (new device, timezone, advertising ID)
2. Builds new `wreq` HTTP + WS clients
3. Replaces the `Arc<Fingerprint>` atomically
4. Retries the request once

This is aggressive but effective for evading soft rate-limit blocks.

### 2.4 Multi-Account System (`auth.rs`)

- Full CRUD: login, add_account, switch_account, remove_account, list_accounts
- Keyring persistence via `AuthStorage` with MsgPack serialization
- Accounts index stored as separate keyring entry for enumeration
- Active session tracking with `active-profile-id` keyring entry
- **Auto-switch on logout:** `logout_current()` activates the next available account
- **Double-checked locking** on `refresh_token()` prevents concurrent refresh storms
- **Pre-emptive refresh:** `authorization_header()` refreshes 60s before expiry

### 2.5 WebSocket (`ws.rs`)

- **Connect timeout:** 15s cap prevents Android Doze wedge on half-open connections
- **Heartbeat:** 45s ping/pong with timeout detection (missed pong → disconnect)
- **Reconnect backoff:** Exponential (1s → 30s max) with circuit breaker at 50 consecutive failures
- **Async signaling:** Separate `auth_notify` and `ws_reconnect_notify` prevent race conditions — `notify_waiters()` ensures both the outer loop and per-connection watcher receive account-switch signals
- **Background notifications:** Push notifications fire only when `is_foreground` is false and sender is not self
- **Rich message previews:** Text truncation, media-type-aware notification bodies (photo, video, voice, album, Gaymoji, GIF, location)

### 2.6 Frontend API Layer (`api/index.ts`)

- **Zod schemas** for all Tauri command request/response shapes — runtime validation catches backend mismatches
- **Typed `callMethod<T>()`** with conditional args (undefined → no arg required)
- **Cloudflare block detection** in `fetchRest().json()` with user-facing alert
- **Auth error handling:** Auto-redirects to sign-in page on `"Not logged in"` errors
- **`parseApiResponse()`** with schema validation and detailed error logging

### 2.7 CI Pipeline (`.github/workflows/build-apk.yml`)

- `cargo check` → `cargo test` → `cargo clippy -D warnings` on push
- Android APK build on release publish + manual `workflow_dispatch`
- Proper NDK/SDK/Java version pinning, artifact uploads

---

## 3. Issues & Recommendations

### 3.1 🔴 HIGH: Silently Dropped WS Messages (`ws.rs:ws_send`)

```rust
// Current:
let _ = state.ws_tx.try_send(command);
// Buffering code is entirely commented out.
```

**Problem:** If the WS channel is full (64 message cap) or the WS is disconnected, outbound messages are silently lost. The user sends a chat message, sees no error, but it never reaches Grindr. There's no user feedback, no retry, no buffer.

**Fix:** Either:
- Restore the buffer (commented out at lines ~203 and ~385) with capacity enforcement
- Return an error to the frontend so it can show "Message not sent" with a retry button
- Use an unbounded channel or a ring buffer with oldest-message eviction

The `WsCommand::BUFFER_CAPACITY` constant (64) still exists but is unused. The commented-out buffer code had the right shape — it just needed the send-after-reconnect path to flush.

### 3.2 🟡 MEDIUM: Version Number Inconsistency

| Source | Version |
|:-------|:--------|
| `Cargo.toml` | `0.2.0-alpha.1` |
| `tauri.conf.json` | `0.1.0-alpha.9` |
| `version.rs` (fallback) | `3.0.0` |

The `svelte.config.js` reads from `version.rs`, but these don't agree. The `tauri.conf.json` version is what shows in app metadata. Unify to a single canonical source — either read from `Cargo.toml` in the build script, or generate `version.rs` from `package.json` version.

### 3.3 🟡 MEDIUM: No Circuit Breaker on Fingerprint Rotation

```rust
// GrindrClient has these fields:
pub(super) last_rotation: AtomicI64,
pub(super) consecutive_rotations: AtomicU32,
// ...but they are NEVER incremented or checked.
```

**Problem:** If Grindr's anti-abuse escalates from soft blocks to device-level bans, the auto-rotate-retry loop will keep rotating indefinitely every time a request fails. Each rotation creates new `wreq` clients (expensive) and new device profiles. There's no cooldown, no user warning, no cap on rotation frequency.

**Fix:** Increment `consecutive_rotations` in `rotate_fingerprint()`, check against a threshold (e.g., 5 rotations in 10 minutes), and if exceeded, surface a "You may be device-banned" warning to the user via a Tauri event.

### 3.4 🟡 MEDIUM: `edition = "2024"` Requires Rust ≥ 1.85

The `rust-toolchain.toml` pins `1.95.0` which does support edition 2024. However, the Zo sandbox runs Rust 1.63. This isn't a production issue but blocks local verification in this environment. No action needed beyond awareness that `cargo check` will fail on older toolchains.

### 3.5 🟡 MEDIUM: Cloudflare Block Detection is Fragile

```typescript
// In fetchRest().json():
if (
  status === 403 &&
  text.includes("<title>Attention Required! | Cloudflare</title>") &&
  text.includes("Sorry, you have been blocked")
) {
```

**Problem:** Relies on exact Cloudflare challenge page strings. If Cloudflare changes their template, detection silently breaks. Also only triggers on HTTP 403 — other Cloudflare responses (e.g., 503 with JS challenge) are missed.

**Fix:** Also check for Cloudflare-specific headers (`cf-ray`, `cf-chl-bypass`, `server: cloudflare`). Consider a broader heuristic: if the response isn't valid JSON and contains `<html`, it's likely a block page.

### 3.6 🟡 MEDIUM: Dead Commented Code

`ws.rs` has ~15 lines of commented-out buffer flush logic in `connect_and_run()` and `ws_send()`. Either restore it (preferred) or remove it. Dead code rots and confuses future readers.

### 3.7 🟢 LOW: `rotate_api_params` Creates New Clients Unnecessarily

```rust
let http = build_api_client()?;
let ws_http = build_ws_client()?;
```

Building new `wreq::Client` instances is expensive — it tears down and rebuilds connection pools and TLS sessions. The fingerprint only needs new headers, not new clients. Consider rotating the `DeviceInfo` and header values only, reusing the existing clients.

### 3.8 🟢 LOW: Large File — `rest.rs`

At ~650 lines, `rest.rs` mixes request dispatch, fingerprint rotation, response rewriting, image upload, authed fetch, and 24 tests. Consider extracting `maybe_rewrite_response` and its tests into `api/rewrite.rs`.

### 3.9 🟢 LOW: No Integration Tests

The unit test coverage for response rewriting is excellent, but there are no end-to-end tests that exercise the full pipeline (login → refresh → authenticated request → response rewrite). Adding a mock HTTP server test would catch regressions in the request/retry/rewrite flow.

### 3.10 🟢 LOW: `L-Grindr-Roles` Missing from Image Upload

```rust
// .header("L-Grindr-Roles", grindr_roles_header_value())  <-- commented out
```

The `upload_image` endpoint omits the `L-Grindr-Roles` header that all other authenticated requests include. The `grindr_roles_header_value()` function was removed in commit `50ec32c`. If Grindr validates this header on upload, images may fail. Either verify it's not required or restore it.

---

## 4. Code Quality Summary

| Dimension | Rating | Notes |
|:----------|:-------|:------|
| **Architecture** | ⭐⭐⭐⭐ | Clean separation of concerns, good use of Tauri + SvelteKit |
| **Response Rewriting** | ⭐⭐⭐⭐⭐ | Sophisticated, well-tested, well-documented injection pipeline |
| **Error Handling** | ⭐⭐⭐⭐ | Typed errors (`AppError`), Zod validation, but WS message loss is a gap |
| **Security** | ⭐⭐⭐⭐ | Platform keyring, Bearer auth, input validation — solid for a reverse-engineering project |
| **Testing** | ⭐⭐⭐ | Excellent unit tests for rewriting, no integration tests, frontend coverage thin |
| **Code Hygiene** | ⭐⭐⭐ | Dead comments, version inconsistency, large files — functional but needs cleanup |
| **Anti-Detection** | ⭐⭐⭐⭐ | OkHttp fingerprint + 80 device profiles + header ordering + auto-rotate are strong |

---

## 5. Build Status

**On this environment:** `cargo check` fails due to Rust 1.63 not supporting edition 2024.  
**On target (Rust 1.95 via `rust-toolchain.toml`):** Should compile cleanly — last CI run passed `cargo check`, `cargo test`, and `cargo clippy -D warnings` as of commit `0e6015f`.

---

## 6. Bottom Line

Open Grind Premium is a **technically sophisticated reverse-engineered client**. The premium injection, fingerprint rotation, multi-account system, and WebSocket handling are production-quality. The frontend is modern and well-structured.

The main gaps are operational reliability issues: silently dropped messages during WS disconnect (P1), the lack of a fingerprint rotation circuit breaker (P2), and version number inconsistency (P3). None are showstoppers, but fixing them would move this from alpha to beta readiness.
