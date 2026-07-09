#!/usr/bin/env bash
# Auto-bump app version manifests from the latest git tag (or explicit VERSION=).
# Syncs: src-tauri/Cargo.toml, package.json, src-tauri/tauri.conf.json (version + versionCode).
# Does NOT change Grindr API FALLBACK_* in version.rs (those track the official app, not Open Grind).
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "$(cd "$(dirname "$0")/.." && pwd)")"
cd "$repo_root"

if [[ -n "${VERSION:-}" ]]; then
  ver="${VERSION#v}"
  echo "[bump-version] using VERSION env: $ver"
else
  latest_tag="$(git describe --tags --abbrev=0 --match 'v*' 2>/dev/null || echo "")"
  if [[ -z "$latest_tag" ]]; then
    echo "[bump-version] no v* tags found and VERSION unset; skipping bump" >&2
    exit 0
  fi
  ver="${latest_tag#v}"
  echo "[bump-version] latest tag: $latest_tag → version: $ver"
fi

# Derive a monotonic Android versionCode from the version string.
# Prefer a numeric override via VERSION_CODE=; else hash-like: major*1e6 + minor*1e3 + patch + alpha.
version_code_from() {
  local v="$1"
  if [[ -n "${VERSION_CODE:-}" ]]; then
    echo "$VERSION_CODE"
    return
  fi
  # Strip pre-release suffix for base numbers: 0.1.0-alpha.19 → 0.1.0 + 19
  local base pre n_major n_minor n_patch n_pre
  base="${v%%-*}"
  pre=""
  if [[ "$v" == *-* ]]; then
    pre="${v#*-}"
  fi
  IFS=. read -r n_major n_minor n_patch _ <<<"${base}.0.0.0"
  n_major="${n_major:-0}"
  n_minor="${n_minor:-0}"
  n_patch="${n_patch:-0}"
  n_pre=0
  if [[ "$pre" =~ ([0-9]+)$ ]]; then
    n_pre="${BASH_REMATCH[1]}"
  fi
  # Keep within 32-bit positive int used by Android (max ~2e9)
  echo $(( n_major * 1000000 + n_minor * 1000 + n_patch * 100 + n_pre ))
}

version_code="$(version_code_from "$ver")"
echo "[bump-version] android versionCode: $version_code"

# --- Cargo.toml (src-tauri) ---
cargo="src-tauri/Cargo.toml"
if [[ -f "$cargo" ]]; then
  python3 - "$cargo" "$ver" <<'PY'
import re, sys
path, ver = sys.argv[1], sys.argv[2]
src = open(path, encoding="utf-8").read()
new = re.sub(r'^version = "[^"]*"', f'version = "{ver}"', src, count=1, flags=re.M)
open(path, "w", encoding="utf-8").write(new)
PY
  echo "[bump-version] updated $cargo"
fi

# --- package.json (frontend) ---
pkg="package.json"
if [[ -f "$pkg" ]]; then
  python3 - "$pkg" "$ver" <<'PY'
import json, sys
path, ver = sys.argv[1], sys.argv[2]
with open(path, encoding="utf-8") as f:
    d = json.load(f)
d["version"] = ver
with open(path, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2)
    f.write("\n")
PY
  echo "[bump-version] updated $pkg"
fi

# --- tauri.conf.json (app metadata + Android versionCode) ---
tauri_conf="src-tauri/tauri.conf.json"
if [[ -f "$tauri_conf" ]]; then
  python3 - "$tauri_conf" "$ver" "$version_code" <<'PY'
import json, sys
path, ver, code = sys.argv[1], sys.argv[2], int(sys.argv[3])
with open(path, encoding="utf-8") as f:
    d = json.load(f)
d["version"] = ver
bundle = d.setdefault("bundle", {})
android = bundle.setdefault("android", {})
android["versionCode"] = code
with open(path, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2)
    f.write("\n")
PY
  echo "[bump-version] updated $tauri_conf (version + versionCode)"
fi

echo "[bump-version] done → $ver (versionCode=$version_code)"
