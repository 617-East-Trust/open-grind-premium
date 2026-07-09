//! Grindr client app version used in the User-Agent.
//!
//! Falls back to compile-time constants, then prefers a keyring-cached value,
//! and optionally refreshes from the Google Play Store listing at runtime.

use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use keyring_core::Entry;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Compile-time fallback when network discovery and cache are unavailable.
pub const FALLBACK_APP_VERSION: &str = "26.9.1.163471";
pub const FALLBACK_BUILD_NUMBER: &str = "163471";

/// Re-fetch at most once per day.
const CACHE_TTL_SECS: u64 = 24 * 60 * 60;

const KEYRING_SERVICE: &str = "open-grind";
const KEYRING_USER: &str = "grindr-api-version";

const PLAY_STORE_URL: &str =
    "https://play.google.com/store/apps/details?id=com.grindrapp.android&hl=en&gl=US";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppVersionInfo {
    pub app_version: String,
    pub build_number: String,
    /// Unix epoch seconds when this value was discovered/refreshed.
    pub fetched_at: u64,
}

impl AppVersionInfo {
    pub fn fallback() -> Self {
        Self {
            app_version: FALLBACK_APP_VERSION.to_owned(),
            build_number: FALLBACK_BUILD_NUMBER.to_owned(),
            fetched_at: 0,
        }
    }

    /// Parse a dotted version such as `26.9.1` or `26.9.1.163471`.
    pub fn from_version_string(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        let parts: Vec<&str> = raw.split('.').collect();
        if parts.len() < 3 || parts.len() > 4 {
            return None;
        }
        if !parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        {
            return None;
        }
        let (app_version, build_number) = if parts.len() == 4 {
            (raw.to_owned(), parts[3].to_owned())
        } else {
            // Play Store often shows 3-part versions. Append a synthetic build
            // component so the UA keeps the expected grindr3/N.N.N.N form.
            let build = parts[2].to_owned();
            (format!("{raw}.{build}"), build)
        };
        Some(Self {
            app_version,
            build_number,
            fetched_at: now_secs(),
        })
    }
}

static CURRENT: RwLock<Option<AppVersionInfo>> = RwLock::new(None);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn keyring_entry() -> Result<Entry, AppError> {
    Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| AppError::Auth(e.to_string()))
}

/// Load cached version from keyring into memory (best-effort).
pub fn load_cached() {
    let entry = match keyring_entry() {
        Ok(e) => e,
        Err(_) => return,
    };
    let bytes = match entry.get_secret() {
        Ok(b) => b,
        Err(_) => return,
    };
    let Ok(info) = rmp_serde::from_slice::<AppVersionInfo>(&bytes) else {
        return;
    };
    if let Ok(mut guard) = CURRENT.write() {
        *guard = Some(info);
    }
}

fn persist(info: &AppVersionInfo) {
    let Ok(entry) = keyring_entry() else {
        return;
    };
    let Ok(bytes) = rmp_serde::to_vec(info) else {
        return;
    };
    let _ = entry.set_secret(&bytes);
}

fn set_current(info: AppVersionInfo) {
    persist(&info);
    if let Ok(mut guard) = CURRENT.write() {
        *guard = Some(info);
    }
}

/// Snapshot of the version currently used for User-Agents.
pub fn current() -> AppVersionInfo {
    if let Ok(guard) = CURRENT.read()
        && let Some(ref info) = *guard
    {
        return info.clone();
    }
    AppVersionInfo::fallback()
}

pub fn app_version() -> String {
    current().app_version
}

pub fn build_number() -> String {
    current().build_number
}

fn cache_is_fresh(info: &AppVersionInfo) -> bool {
    if info.fetched_at == 0 {
        return false;
    }
    now_secs().saturating_sub(info.fetched_at) < CACHE_TTL_SECS
}

/// Refresh the Grindr app version from the Play Store when the cache is stale.
/// Always leaves a usable version in memory (fallback on total failure).
pub async fn refresh_if_stale() -> AppVersionInfo {
    load_cached();
    let existing = current();
    if cache_is_fresh(&existing) && existing.fetched_at > 0 {
        return existing;
    }
    match fetch_from_play_store().await {
        Ok(info) => {
            tracing::info!(
                version = %info.app_version,
                build = %info.build_number,
                "discovered Grindr app version from Play Store"
            );
            set_current(info.clone());
            info
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Grindr app version fetch failed; using cached/fallback"
            );
            if existing.fetched_at == 0 {
                let mut fb = AppVersionInfo::fallback();
                fb.fetched_at = now_secs();
                set_current(fb.clone());
                fb
            } else {
                existing
            }
        }
    }
}

async fn fetch_from_play_store() -> Result<AppVersionInfo, AppError> {
    let client = wreq::Client::builder()
        .gzip(true)
        .build()
        .map_err(|e| AppError::Http(format!("version client build failed: {e}")))?;

    let response = client
        .get(PLAY_STORE_URL)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36",
        )
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| AppError::Http(format!("Play Store request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::Http(format!(
            "Play Store returned HTTP {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| AppError::Http(format!("Play Store body read failed: {e}")))?;

    parse_version_from_play_html(&body).ok_or_else(|| {
        AppError::Http("could not parse Grindr version from Play Store HTML".into())
    })
}

/// Find dotted app versions (3 or 4 numeric segments) in text.
fn find_version_candidates(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        let mut dots = 0;
        let mut ok = true;
        while i < bytes.len() {
            let b = bytes[i];
            if b.is_ascii_digit() {
                i += 1;
            } else if b == b'.' {
                dots += 1;
                if dots > 3 {
                    ok = false;
                    break;
                }
                // require a digit after the dot
                if i + 1 >= bytes.len() || !bytes[i + 1].is_ascii_digit() {
                    ok = false;
                    break;
                }
                i += 1;
            } else {
                break;
            }
        }
        if ok && (dots == 2 || dots == 3) {
            if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                // Prefer versions that look like modern Grindr (20+ major).
                if s.starts_with('2') {
                    out.push(s.to_owned());
                }
            }
        }
        // continue after this run
        if i == start {
            i += 1;
        }
    }
    out
}

/// Extract a version string from Play Store HTML/embedded JSON.
pub fn parse_version_from_play_html(html: &str) -> Option<AppVersionInfo> {
    // Prefer structured markers when present.
    for marker in [r#""softwareVersion":""#, "[[[\"", "Current Version"] {
        if let Some(idx) = html.find(marker) {
            let window = html.get(idx..idx.saturating_add(80)).unwrap_or("");
            for cand in find_version_candidates(window) {
                if let Some(info) = AppVersionInfo::from_version_string(&cand) {
                    return Some(info);
                }
            }
        }
    }

    // Global scan: take the highest-looking 2x.x.x candidate (lexicographic on full string
    // is imperfect but good enough; prefer 4-part over 3-part of same prefix).
    let mut best: Option<AppVersionInfo> = None;
    for cand in find_version_candidates(html) {
        if let Some(info) = AppVersionInfo::from_version_string(&cand) {
            let replace = match &best {
                None => true,
                Some(prev) => {
                    info.app_version > prev.app_version
                        || (info.app_version.starts_with(&prev.app_version)
                            && info.app_version.len() > prev.app_version.len())
                }
            };
            if replace {
                best = Some(info);
            }
        }
    }
    best
}

/// Force a network refresh (ignores TTL). Used by optional debug UI.
pub async fn force_refresh() -> Result<AppVersionInfo, AppError> {
    let info = fetch_from_play_store().await?;
    set_current(info.clone());
    Ok(info)
}

#[tauri::command]
pub async fn grindr_app_version() -> Result<AppVersionInfo, AppError> {
    Ok(current())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_four_part_version() {
        let info = AppVersionInfo::from_version_string("26.9.1.163471").unwrap();
        assert_eq!(info.app_version, "26.9.1.163471");
        assert_eq!(info.build_number, "163471");
    }

    #[test]
    fn parses_three_part_version() {
        let info = AppVersionInfo::from_version_string("26.12.0").unwrap();
        assert_eq!(info.app_version, "26.12.0.0");
        assert_eq!(info.build_number, "0");
    }

    #[test]
    fn rejects_garbage() {
        assert!(AppVersionInfo::from_version_string("").is_none());
        assert!(AppVersionInfo::from_version_string("abc").is_none());
        assert!(AppVersionInfo::from_version_string("1.2").is_none());
    }

    #[test]
    fn parse_play_html_embedded_array() {
        let html = r#"window.AF_initDataCallback({data:[[[["26.15.2"]]]]})"#;
        let info = parse_version_from_play_html(html).unwrap();
        assert!(info.app_version.starts_with("26.15.2"));
    }

    #[test]
    fn parse_play_html_software_version() {
        let html = r#"{"softwareVersion":"26.10.3.170001","foo":1}"#;
        let info = parse_version_from_play_html(html).unwrap();
        assert_eq!(info.app_version, "26.10.3.170001");
        assert_eq!(info.build_number, "170001");
    }

    #[test]
    fn fallback_constants_match_format() {
        let info = AppVersionInfo::from_version_string(FALLBACK_APP_VERSION).unwrap();
        assert_eq!(info.build_number, FALLBACK_BUILD_NUMBER);
    }
}
