#!/usr/bin/env bash
# Apply Android Doze / lifecycle patches after `tauri android init`.
# Safe to re-run. Expects gen/android to already exist.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "$(cd "$(dirname "$0")/../.." && pwd)")"
android_root="$repo_root/src-tauri/gen/android"
app_src="$android_root/app/src/main"

if [[ ! -d "$app_src" ]]; then
  echo "[doze-patch] gen/android not found — run tauri android init first" >&2
  exit 1
fi

# Locate MainActivity.kt regardless of package path.
main_activity="$(find "$app_src/java" -name 'MainActivity.kt' 2>/dev/null | head -1 || true)"
manifest="$app_src/AndroidManifest.xml"

if [[ -z "$main_activity" || ! -f "$manifest" ]]; then
  echo "[doze-patch] MainActivity.kt or AndroidManifest.xml missing" >&2
  exit 1
fi

# Ensure RECEIVE_BOOT_COMPLETED is not required; we only add WAKE_LOCK-free
# network state so the process can re-establish WS after Doze when allowed.
if ! grep -q 'android.permission.ACCESS_NETWORK_STATE' "$manifest"; then
  # Insert before </manifest> is wrong place — permissions go under root before <application>
  if grep -q 'android.permission.INTERNET' "$manifest"; then
    sed -i.bak 's|android.permission.INTERNET" />|android.permission.INTERNET" />\n    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />|' "$manifest" \
      || sed -i '' 's|android.permission.INTERNET" />|android.permission.INTERNET" />\n    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />|' "$manifest"
    rm -f "${manifest}.bak"
    echo "[doze-patch] added ACCESS_NETWORK_STATE permission"
  fi
fi

# Annotate MainActivity with lifecycle comments if not already patched.
if ! grep -q 'OPEN_GRIND_DOZE_PATCH' "$main_activity"; then
  cat >> "$main_activity" <<'EOF'

// OPEN_GRIND_DOZE_PATCH
// WebView visibility is the primary lifecycle signal (see +layout.svelte → set_foreground).
// Keep this marker so CI re-applies network permission patches without duplicating logic.
EOF
  echo "[doze-patch] annotated MainActivity.kt"
fi

echo "[doze-patch] done"
