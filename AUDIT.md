# Open Grind Premium — Code Audit (Status Update)

**Last verified:** 2026-07-14 (commit `4e15890`) — supersedes the 2026-07-08 audit.
**Scope:** Rust backend (`src-tauri/src/`), SvelteKit frontend (`src/`), CI, config, repo hygiene.
**Version:** `0.1.0-alpha.19` (consistent across `Cargo.toml`, `tauri.conf.json`, `version.rs`; live version fetched from Play Store).

## Summary

The codebase is in good shape. Every detection-risk and reliability finding from the
prior audit has been resolved in code. The only outstanding items are minor polish
(passed to `ENHANCEMENT-ROADMAP.md`). Repo hygiene was improved this pass by
untracking the generated `src-tauri/gen/` tree (47 files, ~27 MB, including a
committed 25 MB signed release APK).

## Prior findings — resolution

| # | Prior finding | Resolution |
|---|---------------|------------|
| 3.1 | Outbound `L-Grindr-Roles: [PREMIUM,UNLIMITED]` header | **Removed.** `grindr_roles_header_value()` returns `None`; omitted in `rest.rs`. |
| 3.2 | WebSocket silent-drops lose messages | **Fixed.** `ws_buffer` (128) + flush-on-reconnect + oldest-drop re-queue in `ws.rs`. |
| 3.3 | Fingerprint rotation ran unconditionally / no breaker | **Fixed.** `rotation_circuit_breaker_tripped()` gates `rotate_fingerprint()` in `rest.rs`. |
| 3.4 | Version numbers inconsistent | **Fixed.** Single source + live fetch in `version.rs`. |
| 3.5 | Cloudflare block detection brittle (exact strings) | **Open (minor).** Acceptable for alpha. |
| 3.6 | `SAFE_TIMEZONES` duplicates / narrow | **Fixed.** 87 de-duplicated entries incl. US zones. |
| 3.7 | `rotate_api_params` rebuilds all clients per call | **Open (minor).** Premature-optimization grade. |
| 3.8 | `rest.rs` monolithic | **Mitigated.** Interception extracted to `rewrite.rs`. |
| 3.9 | No integration tests | **Open (minor).** 56 Rust unit tests + fixtures cover the critical paths. |
| 3.10 | `L-Grindr-Roles` missing from image upload | **N/A** — the header was removed entirely (#3.1). |

## Security posture

- No hardcoded secrets in tree (keystore password / API keys are CI secrets).
- Entitlement injection uses realistic single-digit values (`rewrite.rs`) — no `999` detection flag.
- Keyring storage falls back to an encrypted `FileStore` on every platform.

## Build / CI

- `build-apk.yml`: clippy + `cargo test` + `cargo fmt --check` + sign + `--version` bump.
- Frontend: `bun install && bun run build && bun run tauri build`.
- Releases are signed; APK artifact retained 30 days.

## Recommended next steps

1. Review/merge the 4 non-redundant stale branches before deleting them
   (see `ENHANCEMENT-ROADMAP.md` → Branch hygiene).
2. Add a CI job that fails if `src-tauri/gen/` is ever re-committed.
3. Optional: replace exact-string Cloudflare detection with a normalized matcher.
