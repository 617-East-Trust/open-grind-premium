//! Dynamic app version fetching.
//!
//! Fetches the latest Grindr app version from the Google Play Store at startup
//! and caches it for 24 hours. Falls back to hardcoded constants on failure.
//!
//! This prevents the client from being fingerprinted by a stale version string
//! after Grindr publishes an update (they update every 2-4 weeks).

use std::sync::OnceLock;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

const PLAY_STORE_URL: &str =
    "https://play.google.com/store/apps/details?id=com.grindrapp.android&hl=en&gl=us";
const CACHE_TTL_SECS: u64 = 24 * 60 * 60; // 24 hours

/// Hardcoded fallback — used before the first successful fetch and whenever
/// the Play Store is unreachable.
const FALLBACK_APP_VERSION: &str = "26.9.1.163471";
const FALLBACK_BUILD_NUMBER: &str = "163471";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub app_version: String,
    pub build_number: String,
    /// Unix timestamp (seconds) of when this version was fetched.
    pub fetched_at: u64,
    /// Whether this version came from a live fetch (true) or is the fallback (false).
    pub is_dynamic: bool,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            app_version: FALLBACK_APP_VERSION.to_string(),
            build_number: FALLBACK_BUILD_NUMBER.to_string(),
            fetched_at: 0,
            is_dynamic: false,
        }
    }
}

/// Global version store, initialized lazily with the fallback values.
static VERSION_INFO: OnceLock<RwLock<VersionInfo>> = OnceLock::new();

fn version_lock() -> &'static RwLock<VersionInfo> {
    VERSION_INFO.get_or_init(|| RwLock::new(VersionInfo::default()))
}

/// Returns the current version info (may be the fallback if no fetch has
/// succeeded yet).
pub fn current() -> VersionInfo {
    version_lock().read().unwrap().clone()
}

/// Returns the current app version string (e.g. `"26.9.1.163471"`).
pub fn app_version() -> String {
    version_lock().read().unwrap().app_version.clone()
}

/// Returns the current build number string (e.g. `"163471"`).
pub fn build_number() -> String {
    version_lock().read().unwrap().build_number.clone()
}

/// Fetch the latest version from the Play Store and update the global store.
///
/// This is safe to call from a background task at startup. On failure it
/// logs the error and leaves the existing (fallback or cached) version in
/// place — it never panics.
pub async fn fetch_and_update() {
    match fetch_from_play_store().await {
        Ok(info) => {
            eprintln!(
                "[version] fetched dynamic version: {} (build {})",
                info.app_version, info.build_number
            );
            save_to_cache(&info);
            *version_lock().write().unwrap() = info;
        }
        Err(e) => {
            eprintln!("[version] Play Store fetch failed: {e}");
            // Try loading from cache before falling back
            if let Some(cached) = load_from_cache() {
                eprintln!(
                    "[version] using cached version: {} (build {}, age {}s)",
                    cached.app_version,
                    cached.build_number,
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .saturating_sub(cached.fetched_at)
                );
                *version_lock().write().unwrap() = cached;
            } else {
                eprintln!("[version] no cache available — using hardcoded fallback");
            }
        }
    }
}

/// Fetch and parse the version from the Google Play Store listing.
async fn fetch_from_play_store() -> Result<VersionInfo, AppError> {
    let client = wreq::Client::builder()
        .gzip(true)
        .no_deflate()
        .no_brotli()
        .no_zstd()
        .build()
        .map_err(|e| AppError::Http(format!("HTTP client build failed: {e}")))?;

    let response = client
        .get(PLAY_STORE_URL)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Linux; Android 14; Pixel 8) \
             AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/124.0.0.0 Mobile Safari/537.36",
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

    let html = response
        .text()
        .await
        .map_err(|e| AppError::Http(format!("Failed to read Play Store response: {e}")))?;

    let version_str = parse_play_store_version(&html)
        .ok_or_else(|| AppError::Http("Could not parse version from Play Store HTML".into()))?;

    // The Play Store exposes the semver (e.g. "26.9.1") but not the build
    // number. We keep the fallback build number — sending a plausible build
    // is far less risky than sending a stale full version.
    let build_number = FALLBACK_BUILD_NUMBER.to_string();
    let app_version = format!("{version_str}.{build_number}");

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(VersionInfo {
        app_version,
        build_number,
        fetched_at: now,
        is_dynamic: true,
    })
}

/// Parse the version string from Play Store HTML using string search.
///
/// The Play Store embeds the version in various places depending on the
/// rendering path. We try several known patterns and return the first match.
fn parse_play_store_version(html: &str) -> Option<String> {
    // Pattern 1: Search for version in the structured data section
    // The Play Store includes version info in JSON-LD or inline scripts
    if let Some(v) = find_version_near_keyword(html, "softwareVersion") {
        return Some(v);
    }

    // Pattern 2: Look for "Current Version" label (older rendering)
    if let Some(v) = find_version_near_keyword(html, "Current Version") {
        return Some(v);
    }

    // Pattern 3: Look for a version string in the page that matches Grindr's pattern
    // Grindr versions are like "26.9.1", "26.10.0", etc.
    if let Some(v) = find_grindr_version_pattern(html) {
        return Some(v);
    }

    None
}

/// Find a version string (digits.digits.digits) near a keyword in the HTML.
fn find_version_near_keyword(html: &str, keyword: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let key_lower = keyword.to_lowercase();
    let pos = lower.find(&key_lower)?;
    // Search within 500 chars after the keyword for a version pattern
    let search_region = &html[pos..html.len().min(pos + 500)];
    find_version_in_text(search_region)
}

/// Find the first occurrence of a `X.Y.Z` version pattern in a text string.
fn find_version_in_text(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            // Try to parse a version like "26.9.1"
            let mut end = i;
            let mut dot_count = 0;
            while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
                if chars[end] == '.' {
                    dot_count += 1;
                }
                end += 1;
            }
            if dot_count >= 2 && end - i <= 20 {
                let candidate: String = chars[i..end].iter().collect();
                // Validate: major.minor.patch where each part is 1-4 digits
                let parts: Vec<&str> = candidate.split('.').collect();
                if parts.len() >= 3
                    && parts.iter().take(3).all(|p| !p.is_empty() && p.len() <= 4)
                {
                    // Check that the version starts with a plausible major (20-30 range)
                    if let Ok(major) = parts[0].parse::<u32>() {
                        if (20..=30).contains(&major) {
                            return Some(format!("{}.{}.{}", parts[0], parts[1], parts[2]));
                        }
                    }
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    None
}

/// Search the entire HTML for a Grindr-style version pattern.
fn find_grindr_version_pattern(html: &str) -> Option<String> {
    find_version_in_text(html)
}

// ── File-based cache ──────────────────────────────────────────────────────

const CACHE_FILENAME: &str = "version_cache.json";

fn cache_file_path() -> Option<std::path::PathBuf> {
    let dir = std::env::var("OPEN_GRIND_DATA_DIR")
        .or_else(|_| std::env::var("HOME"))
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    let path = std::path::PathBuf::from(dir)
        .join(".open-grind")
        .join(CACHE_FILENAME);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    Some(path)
}

fn load_from_cache() -> Option<VersionInfo> {
    let path = cache_file_path()?;
    let data = std::fs::read_to_string(&path).ok()?;
    let info: VersionInfo = serde_json::from_str(&data).ok()?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if now.saturating_sub(info.fetched_at) > CACHE_TTL_SECS {
        eprintln!("[version] cache expired (age {}s)", now.saturating_sub(info.fetched_at));
        return None;
    }

    Some(info)
}

fn save_to_cache(info: &VersionInfo) {
    if let Some(path) = cache_file_path() {
        if let Ok(json) = serde_json::to_string(info) {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("[version] could not write cache to {}: {e}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_values_are_sane() {
        let info = VersionInfo::default();
        assert!(info.app_version.starts_with("26."));
        assert!(!info.is_dynamic);
        assert!(!info.app_version.is_empty());
        assert!(!info.build_number.is_empty());
    }

    #[test]
    fn app_version_returns_nonempty() {
        let v = app_version();
        assert!(!v.is_empty());
    }

    #[test]
    fn build_number_returns_nonempty() {
        let v = build_number();
        assert!(!v.is_empty());
    }

    #[test]
    fn parse_finds_version_in_play_store_html() {
        let html = r#"<html><head><script>{"@type":"SoftwareApplication","softwareVersion":"26.10.0"}</script></head></html>"#;
        let v = parse_play_store_version(html);
        assert_eq!(v, Some("26.10.0".to_string()));
    }

    #[test]
    fn parse_finds_version_near_current_version_label() {
        let html = r#"<div><span>Current Version</span><div>26.9.1</div></div>"#;
        let v = parse_play_store_version(html);
        assert_eq!(v, Some("26.9.1".to_string()));
    }

    #[test]
    fn parse_returns_none_for_no_version() {
        let html = r#"<html><body>No version here</body></html>"#;
        let v = parse_play_store_version(html);
        assert_eq!(v, None);
    }

    #[test]
    fn find_version_in_text_works() {
        assert_eq!(
            find_version_in_text("blah 26.9.1 blah"),
            Some("26.9.1".to_string())
        );
        assert_eq!(
            find_version_in_text("version 27.0.0 released"),
            Some("27.0.0".to_string())
        );
    }

    #[test]
    fn find_version_ignores_non_grindr_versions() {
        // Versions outside the 20-30 range should be ignored
        assert_eq!(find_version_in_text("version 1.2.3"), None);
        assert_eq!(find_version_in_text("version 50.0.0"), None);
    }

    #[test]
    fn version_info_is_serializable() {
        let info = VersionInfo {
            app_version: "26.10.0.163471".to_string(),
            build_number: "163471".to_string(),
            fetched_at: 1234567890,
            is_dynamic: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: VersionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.app_version, info.app_version);
        assert_eq!(deserialized.is_dynamic, info.is_dynamic);
    }
}
