# Open Grind Premium — Enhancement Status

*Verified against code on `main` @ `4e15890` (2026-07-14). Supersedes the 2025-06-29 roadmap.*

All originally-planned roadmap items are now implemented. What remains is a short
list of minor polish items plus repo hygiene.

## ✅ Completed (with code evidence)

| Item | Status | Evidence |
|------|--------|----------|
| Remove outbound `L-Grindr-Roles: [PREMIUM,UNLIMITED]` header | DONE | `src-tauri/src/api/headers.rs:900` (comment), `grindr_roles_header_value()` returns `None`, omitted in `rest.rs` |
| WebSocket silent-drop buffering | DONE | `src-tauri/src/api/ws.rs` — `ws_buffer` (128 cap), flush-on-reconnect, `try_send` with oldest-drop re-queue |
| Consistent version numbers + live version fetch | DONE | `src-tauri/Cargo.toml` `0.1.0-alpha.19`, `tauri.conf.json`, `src-tauri/src/api/version.rs` (`FALLBACK_APP_VERSION` + Play Store fetch) |
| Realistic device fingerprint pool | DONE | 87 `DeviceProfile` instances in `src-tauri/src/api/headers.rs` |
| Fingerprint rotation circuit breaker | DONE | `src-tauri/src/api/rest.rs:136` `rotation_circuit_breaker_tripped()`, `rotate_fingerprint()` at :142/:185 |
| Realistic entitlement values (no `999` detection flag) | DONE | `src-tauri/src/api/rewrite.rs:84` "Realistic single-digit allocations (not 999 — detection risk)" |
| Broaden API interception points | DONE | `src-tauri/src/api/rewrite.rs` adds `/v1/views`, `/v1/favorites` (maxFavorites 9999), `/v3/explore`, `/v4/album`, `/v2/albums`, `/v3/me/prefs`, `/v3/me/settings`, `/v4/subscriptions`, `/v2/inbox`, `/v3/inbox` |
| Universal keyring fallback (FileStore) | DONE | `src-tauri/src/lib.rs` / `storage.rs` — `FileStore` set as default store on all targets |
| `SAFE_TIMEZONES` de-duplicated + expanded | DONE | 87 entries in `src-tauri/src/api/headers.rs`, no duplicates, includes US zones |
| CI: clippy + tests + fmt + signing + version bump | DONE | `.github/workflows/build-apk.yml` |
| `clippy.toml` / `rustfmt.toml` | DONE | `src-tauri/.clippy.toml`, `src-tauri/rustfmt.toml` |
| Fixture-based response tests | DONE (partial) | `src-tauri/src/api/fixtures/*.json` |
| Structured `tracing` logging | DONE | `src-tauri/src/api/version.rs`, `ws.rs` use `tracing` |
| Release APK signing | DONE | `build-apk.yml` keystore secrets; commit `68a725e` |
| Cascade v3 card `@type` literals | DONE | `src/lib/model/grid/cascade/response/v3.ts` — all 8 incomplete schemas completed (2026-07-14) |
| Stop tracking generated `src-tauri/gen/` | DONE | `.gitignore` + commit `4e15890` (removed 47 files incl. a committed 25 MB signed APK) |

## 🟡 Remaining (minor / optional)

| Item | Notes |
|------|-------|
| Fingerprint-hash debug UI (#17) | Not found in code; optional nicety. |
| Cloudflare block detection is exact-string match (AUDIT #3.5) | `src/lib/.../grid.svelte` matches specific phrases; brittle to copy changes. Low risk. |
| `rotate_api_params` rebuilds all clients per call (AUDIT #3.7) | Minor perf cost on rotation; premature-optimization grade. |
| No integration tests (AUDIT #3.9) | Rust unit suite is solid (56 tests); no end-to-end. Acceptable for alpha. |
| `scripts/no-bun-test.ts` misnamed | Cosmetic; name implies it's a test but it isn't. |
| `ToastUnimplemented` placeholder | Expected app behavior (feature flags), not a defect. |

## 🧹 Branch hygiene

Five stale remote branches remain. Only `feature/android-fixes-final` is reachable
from `main` (fully redundant) — deleted 2026-07-14. The other four
(`feat/roadmap-remaining-enhancements`, `fix/audit-enhancements-20260709`,
`feature/android-fixes`, `fix/crash-bugs-alpha10`) contain commits NOT in `main`'s
history (verified via `git merge-base --is-ancestor`). **Do not delete blindly** —
review/merge them first, or they will lose unique work.
