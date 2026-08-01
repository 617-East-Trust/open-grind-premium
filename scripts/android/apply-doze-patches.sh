#!/usr/bin/env bash
# Apply Android Doze / lifecycle / packaging / permission patches after
# `tauri android init`. Safe to re-run. Expects gen/android to already exist.
#
# Goals for physical devices (especially Pixel 9 series / Android 15+):
#   1. 16 KB page-size install compatibility (useLegacyPackaging)
#   2. Correct SDK / NDK / build-tools pins so CI and local builds match
#   3. Runtime permissions the system expects before the Tauri plugins
#      can request them (location, notifications, network state)
#   4. Marker so the patch stays idempotent across CI runs
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "$(cd "$(dirname "$0")/../.." && pwd)")"
android_root="$repo_root/src-tauri/gen/android"
gradle_props="$android_root/gradle.properties"
app_src="$android_root/app/src/main"

if [[ ! -d "$app_src" ]]; then
  echo "[doze-patch] gen/android not found — run tauri android init first" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# 1. gradle.properties — pin SDK / NDK / CMake so generated project matches CI
# ---------------------------------------------------------------------------
if [[ -f "$gradle_props" ]]; then
  set_prop() {
    local key="$1"
    local val="$2"
    if grep -q "^${key}=" "$gradle_props"; then
      sed -i "s/^${key}=.*/${key}=${val}/" "$gradle_props"
    else
      [ -n "$(tail -c1 "$gradle_props" 2>/dev/null || echo x)" ] && echo >> "$gradle_props"
      echo "${key}=${val}" >> "$gradle_props"
    fi
  }
  set_prop "opengrind.android.compileSdk" "35"
  set_prop "opengrind.android.targetSdk" "35"
  set_prop "opengrind.android.minSdk" "28"
  set_prop "opengrind.android.buildTools" "35.0.0"
  set_prop "opengrind.android.ndk" "27.0.12077973"
  set_prop "opengrind.android.cmake" "3.31.6"
  echo "[doze-patch] ensured opengrind.android.* SDK props: compileSdk=35 targetSdk=35 minSdk=28 buildTools=35.0.0 ndk=27.0.12077973"
fi

# ---------------------------------------------------------------------------
# 2. 16 KB page-size packaging (Pixel 9 / 9 Pro / 9 Pro XL / Android 15+)
# ---------------------------------------------------------------------------
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
else:
    m2 = re.search(r'^android \{', src, re.M)
    if m2:
        insert_at = m2.end()
        src = src[:insert_at] + "\n" + block + src[insert_at:]
        open(p, "w").write(src)
PY
  echo "[doze-patch] injected jniLibs.useLegacyPackaging=true into build.gradle.kts"
fi

# ---------------------------------------------------------------------------
# 3. AndroidManifest.xml — permissions (Python XML-safe, not sed)
# ---------------------------------------------------------------------------
manifest="$app_src/AndroidManifest.xml"
if [[ ! -f "$manifest" ]]; then
  echo "[doze-patch] AndroidManifest.xml missing" >&2
  exit 1
fi

python3 - "$manifest" <<'PY'
import sys
from pathlib import Path

manifest = Path(sys.argv[1])
text = manifest.read_text(encoding="utf-8")

REQUIRED = [
    "android.permission.ACCESS_NETWORK_STATE",
    "android.permission.ACCESS_COARSE_LOCATION",
    "android.permission.ACCESS_FINE_LOCATION",
    "android.permission.POST_NOTIFICATIONS",
    "android.permission.VIBRATE",
]

missing = [p for p in REQUIRED if p not in text]
if not missing:
    print("[doze-patch] all required permissions already present")
    sys.exit(0)

lines_to_insert = [
    f'    <uses-permission android:name="{p}" />' for p in missing
]
block = "\n".join(lines_to_insert) + "\n"

# Prefer insert right after the INTERNET permission line
needle = 'android.permission.INTERNET"'
idx = text.find(needle)
if idx != -1:
    # Find end of that line
    line_end = text.find("\n", idx)
    if line_end == -1:
        line_end = len(text)
    else:
        line_end += 1  # include the newline
    text = text[:line_end] + block + text[line_end:]
else:
    # Fallback: insert before <application
    app_idx = text.find("<application")
    if app_idx == -1:
        app_idx = text.find("<activity")
    if app_idx == -1:
        print("[doze-patch] warning: could not find insertion point", file=sys.stderr)
        sys.exit(0)
    text = text[:app_idx] + block + text[app_idx:]

manifest.write_text(text, encoding="utf-8")
for p in missing:
    print(f"[doze-patch] added {p}")
PY

# ---------------------------------------------------------------------------
# 4. MainActivity.kt marker
# ---------------------------------------------------------------------------
main_activity="$(find "$app_src/java" -name 'MainActivity.kt' 2>/dev/null | head -1 || true)"
if [[ -n "$main_activity" ]]; then
  if ! grep -q 'OPEN_GRIND_DOZE_PATCH' "$main_activity"; then
    cat >> "$main_activity" <<'EOF'

// OPEN_GRIND_DOZE_PATCH
// WebView visibility is the primary lifecycle signal (see +layout.svelte → set_foreground).
// Keep this marker so CI re-applies network / permission patches without duplicating logic.
// Physical-device Doze reconnect is driven by set_foreground + background health ticker in ws.rs.
EOF
    echo "[doze-patch] annotated MainActivity.kt"
  fi
else
  echo "[doze-patch] warning: MainActivity.kt not found (non-fatal)" >&2
fi

echo "[doze-patch] done"
