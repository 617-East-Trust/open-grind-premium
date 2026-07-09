//! Data-driven premium feature injection and gate removal.
//! Add new rules here — no more giant if-else chains.

use serde_json::{json, Map, Value};
use std::sync::OnceLock;

pub struct RewriteRule {
    pub path_prefix: &'static str,
    pub transform: fn(&mut Value),
}

/// Remove common paywall / upgrade gate fields from a JSON object (or nested objects).
fn strip_upgrade_gates(json: &mut Value) {
    let Some(obj) = json.as_object_mut() else {
        return;
    };
    for key in [
        "upgradeRequired",
        "requiresUpgrade",
        "isUpgradeRequired",
        "paywall",
        "truncatedProfiles",
        "maxFavorites",
        "favoritesLimit",
    ] {
        obj.remove(key);
    }
    // Nested wrappers some endpoints use
    if let Some(data) = obj.get_mut("data") {
        strip_upgrade_gates(data);
    }
    if let Some(meta) = obj.get_mut("meta") {
        strip_upgrade_gates(meta);
    }
}

fn inject_premium_subscription(json: &mut Value) {
    json["subscription"] = json!({
        "premium": true,
        "userRole": "UNLIMITED",
        "subscriptionTier": "UNLIMITED"
    });
}

pub fn get_rewrite_rules() -> &'static [RewriteRule] {
    static RULES: OnceLock<Vec<RewriteRule>> = OnceLock::new();

    RULES.get_or_init(|| {
        vec![
            // === Core Premium ===
            RewriteRule {
                path_prefix: "/v3/bootstrap",
                transform: |json| {
                    json["userRole"] = json!("UNLIMITED");
                    json["subscriptionTier"] = json!("UNLIMITED");

                    let mut flags: Map<String, Value> = json
                        .get("featureFlags")
                        .and_then(|f| f.as_object())
                        .cloned()
                        .unwrap_or_default();

                    for key in [
                        "readReceipts",
                        "tapAndGo",
                        "unlimitedTaps",
                        "unlimitedFavorites",
                        "unlimitedBlocks",
                        "incognitoMode",
                        "typingStatus",
                        "expire24hProfile",
                        "hideDistance",
                        "boosts",
                        "profileViews",
                    ] {
                        flags.insert(key.to_string(), json!(true));
                    }
                    json["featureFlags"] = Value::Object(flags);
                },
            },
            RewriteRule {
                path_prefix: "/v1/entitlements",
                transform: |json| {
                    // Realistic single-digit allocations (not 999 — detection risk).
                    // Only raise values that are lower than our floor so real
                    // Unlimited accounts are never downgraded.
                    let floor = 15i64;
                    for key in ["rightNow", "total"] {
                        let current = json.get(key).and_then(|v| v.as_i64()).unwrap_or(0);
                        if current < floor {
                            json[key] = json!(floor);
                        }
                    }
                },
            },
            RewriteRule {
                path_prefix: "/v1/me",
                transform: inject_premium_subscription,
            },
            RewriteRule {
                path_prefix: "/v3/me/profile",
                transform: inject_premium_subscription,
            },
            RewriteRule {
                path_prefix: "/v4/subscriptions",
                transform: inject_premium_subscription,
            },
            // === Gate Removal ===
            RewriteRule {
                path_prefix: "/v2/inbox",
                transform: strip_upgrade_gates,
            },
            RewriteRule {
                path_prefix: "/v3/inbox",
                transform: strip_upgrade_gates,
            },
            RewriteRule {
                path_prefix: "/v1/views",
                transform: |json| {
                    strip_upgrade_gates(json);
                    if let Some(obj) = json.as_object_mut() {
                        obj.insert("canViewAll".to_string(), json!(true));
                        // Floor remaining views so free-tier 0 does not keep the paywall.
                        let remaining = obj
                            .get("remainingViews")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        if remaining < 50 {
                            obj.insert("remainingViews".to_string(), json!(50));
                        }
                    }
                },
            },
            RewriteRule {
                path_prefix: "/v1/favorites",
                transform: |json| {
                    strip_upgrade_gates(json);
                    if let Some(obj) = json.as_object_mut() {
                        obj.insert("maxFavorites".to_string(), json!(9999));
                        obj.insert("canAddMore".to_string(), json!(true));
                    }
                },
            },
            RewriteRule {
                path_prefix: "/v3/explore",
                transform: |json| {
                    strip_upgrade_gates(json);
                    if let Some(obj) = json.as_object_mut() {
                        // Prefer larger page sizes when the server omits or caps them.
                        let page = obj.get("pageSize").and_then(|v| v.as_i64()).unwrap_or(0);
                        if page > 0 && page < 50 {
                            obj.insert("pageSize".to_string(), json!(50));
                        }
                    }
                },
            },
            RewriteRule {
                path_prefix: "/v4/album",
                transform: strip_upgrade_gates,
            },
            RewriteRule {
                path_prefix: "/v2/albums",
                transform: strip_upgrade_gates,
            },
            // === Settings / prefs injection ===
            RewriteRule {
                path_prefix: "/v3/me/settings",
                transform: |json| {
                    if let Some(obj) = json.as_object_mut() {
                        obj.entry("showDistance".to_string())
                            .or_insert(json!(true));
                        obj.entry("incognito".to_string())
                            .or_insert(json!(false));
                    }
                },
            },
            RewriteRule {
                path_prefix: "/v3/me/prefs",
                transform: |json| {
                    if let Some(obj) = json.as_object_mut() {
                        obj.entry("showDistance".to_string())
                            .or_insert(json!(true));
                        obj.entry("showOnlineStatus".to_string())
                            .or_insert(json!(true));
                        obj.entry("showLastSeen".to_string())
                            .or_insert(json!(true));
                    }
                },
            },
        ]
    })
}

/// Match a path against a rule prefix on segment boundaries.
/// `/v1/me` matches `/v1/me` and `/v1/me/extra`, but not `/v1/media`.
pub fn path_matches(path: &str, prefix: &str) -> bool {
    if path == prefix {
        return true;
    }
    let rest = match path.strip_prefix(prefix) {
        Some(r) => r,
        None => return false,
    };
    rest.starts_with('/') || rest.starts_with('?')
}

pub fn apply_rewrites(path: &str, json: &mut Value) {
    // Strip query string for matching; keep original path for callers if needed.
    let path_lower = path
        .split_once('?')
        .map(|(p, _)| p)
        .unwrap_or(path)
        .to_lowercase();
    for rule in get_rewrite_rules() {
        if path_matches(&path_lower, rule.path_prefix) {
            (rule.transform)(json);
            return;
        }
    }
}

#[cfg(test)]
mod path_match_tests {
    use super::path_matches;

    #[test]
    fn exact_and_subpath_match() {
        assert!(path_matches("/v1/me", "/v1/me"));
        assert!(path_matches("/v1/me/profile", "/v1/me"));
        assert!(path_matches("/v1/me?x=1", "/v1/me"));
    }

    #[test]
    fn does_not_match_shared_prefix_segments() {
        assert!(!path_matches("/v1/media", "/v1/me"));
        assert!(!path_matches("/v1/messages", "/v1/me"));
        assert!(!path_matches("/v3/explorex", "/v3/explore"));
    }
}
