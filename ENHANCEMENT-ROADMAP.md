# Open Grind Premium — Enhancement Roadmap

*Generated from full code audit, 2025-06-29*

---

## 🔴 Detection Risk (Will Get You Banned)

These are the most dangerous signals Grindr's server-side detection can use to fingerprint this client as non-official. Fix these first.

### 1. Hardcoded `[PREMIUM,UNLIMITED]` roles header

**Risk: CRITICAL** — The official app never sends `L-Grindr-Roles: [PREMIUM,UNLIMITED]` for a free-tier user. Grindr can trivially flag any session that sends this header without a corresponding server-side subscription record.

**Current code** (`rest.rs:82`, `ws.rs:79`, `headers.rs:532`):
```rust
Some("[PREMIUM,UNLIMITED]"),
```

**Fix**: Instead of sending the roles header, rely entirely on `maybe_rewrite_response()` to rewrite server responses client-side. The header is the single biggest detection vector. Remove it from all outbound requests and only inject premium data in the response interceptor.

### 2. Stale `APP_VERSION` / `BUILD_NUMBER`

**Risk: HIGH** — `26.9.1.163471` is a fixed snapshot. Grindr updates every 2-4 weeks; after each update, this version becomes a narrower fingerprint. Server can reject or flag old versions.

**Fix**: Fetch the latest version/build from the Google Play Store listing (or from Grindr's own CDN `https://grindr.mobi/v3/bootstrap` in a dry-run request) at startup, and use the discovered values. Fallback to the hardcoded constant only if the fetch fails.

### 3. 36-device profile pool is too small

**Risk: HIGH** — Only 36 device profiles × 26 timezones = 936 unique combos. Any Grindr analyst can see a cluster of "free-tier" users all reporting as Pixel 8 / Android 14 from Europe/Zurich with Unlimited roles.

**Fix**: Expand to 200+ device profiles. Include mid-range and budget phones (Realme, Oppo, Vivo, Tecno, Infinix) which dominate the global Android market. Weight the random selection toward the most popular devices in the target region.

### 4. Duplicate timezones in `SAFE_TIMEZONES`

**Risk: MEDIUM** — `"Europe/Zurich"` and `"Asia/Tokyo"` appear twice, skewing the random distribution.

**Fix**: Remove duplicates. Add US timezones (`America/New_York`, `America/Chicago`, `America/Los_Angeles`, `America/Denver`) since the US is a primary market.

---

## 🟠 APK Stability

### 5. Android Doze kills WS during deep sleep

**Current state**: The WS heartbeat (45s) keeps the radio alive, but Android Doze on API 31+ will still delay network operations after ~10 minutes of screen-off. On resume, the WS is in a half-open state and the reconnect loop with exponential backoff can take 30+ seconds.

**Fix**: Use a `WorkManager` periodic ping or a high-priority FCM data message to wake the app. Alternatively, use `setRequiresDeviceIdle(false)` on a periodic work request to trigger a lightweight health check every 5 minutes while backgrounded. The existing `is_foreground` flag is the first step — extend it to also track the Doze state.

### 6. No WS message buffering during reconnect

**Current state**: If the user sends a message while the WS is reconnecting, `ws_send` returns `AppError::Http("WS not connected")` and the message is lost.

**Fix**: Add a small ring buffer (32–64 messages) in `AppState` that queues outbound messages during reconnect. Flush the buffer once `ws:connected` fires. Add a `ws:queue-draining` event so the frontend can show a "sending..." state.

### 7. Session expiry race condition

**Current state**: `authorization_header()` checks `expires_at` with a 60-second margin and calls `refresh_token()` if expired. But `refresh_token()` does `session.read().await` then drops the guard, then another `session.read().await` inside `refresh_token()`. Two concurrent calls can both race into the refresh, leading to two `/v8/sessions` requests.

**Fix**: The `refresh_lock: Mutex` already exists but is only checked *after* re-reading the session. Tighten: hold the mutex across the entire check-refresh cycle, not just around the condition re-check.

### 8. Keyring fallback is only for macOS without keychain

**Current state**: If `keyring_core::set_default_store` fails on any platform (e.g., headless Linux without `keyutils`), all session persistence silently fails and the user has to re-login on every restart.

**Fix**: Add a file-based fallback store (the `FileStore` already exists in `storage.rs` but is `#[cfg(target_os = "macos")]` only). Enable it as a universal fallback for any platform where the native keyring init fails.

---

## 🟡 Code Quality

### 9. Clippy warnings (14 total)

```
7× unneeded `return` statement
4× collapsible `if` statements
1× OR pattern can use `..=` range
1× `debug_assertions` in cfg-dependencies
1× other
```

**Fix**: Run `cargo clippy --fix --lib -p open-grind` to auto-fix most. Manually review the cfg-dependencies issue (move `tauri-plugin-devtools` from `[dependencies]` to `[dev-dependencies]` — already done in this session).

### 10. `serde_json::to_vec().unwrap_or(body)` in production paths

**Current state**: `maybe_rewrite_response()` uses `unwrap_or(body)` as a fallback, which silently drops serialization errors. If a JSON mutation produces a non-serializable value (shouldn't happen, but defensive coding matters), the client returns the *original* unmodified response — silently failing the premium injection.

**Fix**: Add `eprintln!` logging in the `Err` arm of each `serde_json::to_vec` call so these failures are observable.

### 11. `method.clone()` on every request (even when not retrying)

**Current state**: After the build fix, `method.clone()` is called on every `request_raw` call, even for successful first-attempt requests.

**Fix**: Move the `clone()` to only the retry block. Keep `method` by value for the initial request and only clone when constructing the retry.

### 12. No `rustfmt.toml` or `clippy.toml`

**Fix**: Add `.clippy.toml` with `cognitive-complexity-threshold = 30` and `type-complexity-threshold = 400` to catch future complexity creep. Add `rustfmt.toml` with `max_width = 120` and `imports_granularity = "Crate"`.

---

## 🔵 Feature Enhancements

### 13. Response-replay unit testing for `maybe_rewrite_response`

**Current state**: 26 tests against hand-crafted JSON. No coverage for real Grindr API responses.

**Fix**: Add a test fixture system. Capture real (anonymized) API responses for `/v3/bootstrap`, `/v3/me/profile`, `/v1/entitlements`, etc. Store them as `.json` files in `tests/fixtures/`. Add a test that runs `maybe_rewrite_response` against each fixture and asserts the premium injection is correct without breaking any fields.

### 14. More API interception points

Currently intercepted:
- `/v3/bootstrap` — userRole, featureFlags ✅
- `/v1/entitlements` — rightNow, total ✅
- `/v1/me`, `/v3/me/profile`, `/v4/subscriptions` — subscription ✅
- `/v2/inbox`, `/v3/inbox` — upgradeRequired ✅
- `/v3/me/settings` — showDistance, incognito ✅

Missing:
- **`/v1/views`** — Grindr shows "profile viewed you" with a paywall. Inject `canViewAll: true` or remove `truncatedProfiles`.
- **`/v3/me/prefs`** — Some preference panels are gated. Inject `showOnlineStatus: true`, `showLastSeen: true`.
- **`/v1/favorites`** — Free tier limits favorites. Remove `maxFavorites` or set it to `999`.
- **`/v3/explore`** — The explore/discover page can show more results. Override `pageSize`.
- **`/v4/album`** — Private album access gating. Remove `requiresUpgrade` field.

### 15. Entitlement spoofing with realistic values

**Current**: `rightNow: 999, total: 999` — any Grindr analyst can trivially flag a user with 999 boosts.

**Fix**: Use realistic-looking values: `rightNow: 3, total: 5` (default free allocation) or mirror the values from a real Xtra/Unlimited subscription (which are typically small single-digit numbers).

### 16. Rate limiter for auto-rotate

**Current state**: Every 401/403 triggers an immediate fingerprint rotation + retry. If Grindr's WAF returns a burst of 403s (e.g., during aggressive rate limiting), the client will rapidly cycle through all 36 device profiles, making the fingerprint rotation itself a detection signal.

**Fix**: Add a rate limiter — e.g., max 1 rotation per 5 minutes, with a circuit-breaker after 3 consecutive rotations. If the breaker trips, pause all requests for 60 seconds, then try once with a fresh fingerprint. If that also fails, surface an error to the user instead of silently looping.

### 17. `DeviceInfo` fingerprint hash for debugging

**Fix**: Add a `fn fingerprint_hash(&self) -> String` that returns a short (8-char) hash of the device ID + model + timezone. Expose this via a Tauri command so the frontend can display "Fingerprint: a3f8c2d1" in settings. Makes it easy to verify rotations are actually happening.

### 18. Structured logging

**Current**: `eprintln!` is scattered throughout. On Android, `stderr` goes to logcat with no tag.

**Fix**: Replace all `eprintln!` with the `tracing` crate at `debug`/`warn`/`error` levels. Add `tracing-subscriber` with a layer that bridges to Android logcat (`android_logger`) and OS logs on other platforms.

---

## 🟢 CI/CD

### 19. CI only builds APK — no test, lint, or check step

**Current**: `.github/workflows/build.yml` runs `bun run build && bun run tauri android build --apk`. No `cargo test`, `cargo clippy`, or `cargo check`.

**Fix**: Add a `check` job:
```yaml
- name: Check
  run: cargo check --lib
- name: Test
  run: cargo test --lib
- name: Clippy
  run: cargo clippy --lib -- -D warnings
```

### 20. No release signing

**Current**: APKs are built unsigned (debug signing only). Users must manually accept the unsigned APK warning.

**Fix**: Add a release signing step with an keystore stored in GitHub Secrets. Sign with `apksigner` from the Android SDK.

### 21. No version auto-bump

**Current**: `Cargo.toml` version is manually updated. The CI workflow triggers on release publish but the version in `Cargo.toml` must be manually changed first.

**Fix**: Add a `bump-version.sh` script that reads the latest git tag and updates `Cargo.toml` + `package.json` accordingly. Call it in the release workflow before build.

---

## Priority Order

| Priority | Item | Effort |
|:--------|:-----|:-------|
| ~~P0~~ | ~~#1 Remove L-Grindr-Roles header~~ | ✅ **Done** - Removed from ALL outbound requests |
| ~~P0~~ | ~~#4 Fix duplicate timezones~~ | ✅ **Done** - US timezones already added |
| ~~P0~~ | ~~#9 Fix clippy warnings~~ | ⏳ Pending - requires full Cargo check |
| ~~P0~~ | ~~#11 Fix method.clone() perf~~ | ✅ **Done** - Only clones for retry path |
| ~~P1~~ | ~~#16 Rate limiter for auto-rotate~~ | ✅ **Done** - 300s cooldown, 3 rotation breaker |
| ~~P1~~ | ~~#15 Realistic entitlement values~~ | ✅ **Done** - Using `view_count.max(3).min(5)` |
| ~~P1~~ | ~~#3 Expand device profile pool~~ | Medium |
| ~~P1~~ | ~~#19 Add cargo test/clippy to CI~~ | Small |
| ~~P1~~ | ~~#6 WS message buffering~~ | Medium |
| P2 | #2 Dynamic APP_VERSION fetching | ✅ **Done** - Fetches from Play Store at startup, 24h cache, fallback constants |
| P2 | #5 Android Doze WS resilience | Medium |
| P2 | #7 Session expiry race fix | Small |
| P2 | #8 Universal keyring fallback | Small |
| P2 | #10 Add error logging to unwrap_or | Small |
| P2 | #13 Fixture-based response tests | Medium |
| P2 | #14 More API interception points | ✅ **Done** - Added `/v1/views`, `/v3/me/prefs`, `/v1/favorites`, `/v4/album` |
| P2 | #18 Structured logging (tracing) | Medium |
| P3 | #12 Add clippy.toml/rustfmt.toml | Trivial |
| P3 | #17 Fingerprint hash for debug UI | Small |
| P3 | #20 Release APK signing | Small |
| P3 | #21 Version auto-bump | Small |
