#!/usr/bin/env bash
# Apply Android Doze / lifecycle patches after `tauri android init`.
# Safe to re-run. Expects gen/android to already exist.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "$(cd "$(dirname "$0")/../.." && pwd)")"
android_root="$repo_root/src-tauri/gen/android"
gradle_props="$android_root/gradle.properties"
app_src="$android_root/app/src/main"

if [[ ! -d "$app_src" ]]; then
  echo "[doze-patch] gen/android not found — run tauri android init first" >&2
  exit 1
fi
# Downgrade SDK targets for Android 15 (API 35) device compatibility.
# tauri android init defaults to API 36, which blocks install on API 35 devices.
if [[ -f "$gradle_props" ]]; then
  sed -i.bak \
    -e 's/^opengrind.android.compileSdk=.*/opengrind.android.compileSdk=35/' \
    -e 's/^opengrind.android.targetSdk=.*/opengrind.android.targetSdk=35/' \
    -e 's/^opengrind.android.buildTools=.*/opengrind.android.buildTools=35.0.0/' \
    "$gradle_props" && rm -f "${gradle_props}.bak"
  echo "[doze-patch] pinned compileSdk/targetSdk=35, buildTools=35.0.0"
fi

# Patch app/build.gradle.kts for 16KB page size compatibility on Pixel 9 Pro XL / Android 15.
# Compressed .so packaging bypasses the ELF PT_LOAD 16KB-alignment install block.
app_gradle="$android_root/app/build.gradle.kts"
if [[ -f "$app_gradle" ]] && ! grep -q 'useLegacyPackaging' "$app_gradle"; then
  python3 - "$app_gradle" <<'PY'
import sys, re
p = sys.argv[1]
src = open(p).read()
block = """    // Workaround for Android 15+ 16KB page size install block.
    // Compressed .so files are extracted at install time, bypassing the
    // ELF PT_LOAD 16KB-alignment check that rejects unaligned libs on
    // 16KB-page-size devices like the Pixel 9 Pro XL.
    packaging {
        jniLibs {
            useLegacyPackaging = true
        }
    }
"""
m = list(re.finditer(r'^    buildFeatures \{[^}]*\n    \}', src, re.M))
if m:
    end = m[-1].end()
    src = src[:end] + "\n" + block + src[end:]
    open(p, "w").write(src)
PY
  echo "[doze-patch] injected jniLibs.useLegacyPackaging=true into build.gradle.kts"
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
