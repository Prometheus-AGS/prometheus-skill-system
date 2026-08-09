#!/usr/bin/env bash
# mint-certification-bundle.sh — assemble a self-contained certification
# bundle for a module from a cucumber test run.
#
# Layout produced:
#   docs/certifications/<module>/<git-sha>/
#     ├── manifest.json          SHA-256 of every artifact + module fingerprint
#     ├── cucumber-report.json   raw cucumber output
#     ├── videos/*.mp4           ffmpeg-remuxed from Playwright WebM
#     ├── screenshots/**         copied verbatim
#     └── report.html            human-readable index
#
# Usage:
#   mint-certification-bundle.sh \
#       --module <name> \
#       --cucumber-json <path> \
#       [--videos-dir <path>] \
#       [--screenshots-dir <path>] \
#       [--source-dir <path>]       # for module_fingerprint (default: src/<module>)
#       [--out-root <path>]         # default: docs/certifications
#       [--dry-run]
#
# Exit codes: 0 OK, 1 arg error, 2 missing dep, 3 mint failure

set -euo pipefail

# ---- args ----
MODULE=""
CUCUMBER_JSON=""
VIDEOS_DIR=""
SCREENSHOTS_DIR=""
SOURCE_DIR=""
OUT_ROOT="docs/certifications"
DRY_RUN=0

while [ $# -gt 0 ]; do
    case "$1" in
        --module)           MODULE="$2"; shift 2 ;;
        --cucumber-json)    CUCUMBER_JSON="$2"; shift 2 ;;
        --videos-dir)       VIDEOS_DIR="$2"; shift 2 ;;
        --screenshots-dir)  SCREENSHOTS_DIR="$2"; shift 2 ;;
        --source-dir)       SOURCE_DIR="$2"; shift 2 ;;
        --out-root)         OUT_ROOT="$2"; shift 2 ;;
        --dry-run)          DRY_RUN=1; shift ;;
        -h|--help)          sed -n 's/^# \{0,1\}//p' "$0" | head -30; exit 0 ;;
        *) echo "mint: unknown arg $1" >&2; exit 1 ;;
    esac
done

[ -z "$MODULE" ]         && { echo "mint: --module required" >&2; exit 1; }
[ -z "$CUCUMBER_JSON" ]  && { echo "mint: --cucumber-json required" >&2; exit 1; }
[ -f "$CUCUMBER_JSON" ]  || { echo "mint: cucumber json not found: $CUCUMBER_JSON" >&2; exit 1; }

# ---- prereqs ----
for tool in jq git shasum; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        # shasum is not on Linux by default; fall back to sha256sum
        if [ "$tool" = "shasum" ] && command -v sha256sum >/dev/null 2>&1; then
            continue
        fi
        echo "mint: $tool is required" >&2
        exit 2
    fi
done

if ! command -v ffmpeg >/dev/null 2>&1; then
    if [ -n "$VIDEOS_DIR" ]; then
        echo "mint: ffmpeg is required to remux videos (or omit --videos-dir)" >&2
        exit 2
    fi
fi

# sha256 helper — prefer shasum -a 256, fall back to sha256sum
sha256() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        sha256sum "$1" | awk '{print $1}'
    fi
}

# ---- resolve context ----
GIT_SHA_FULL="$(git rev-parse HEAD)"
GIT_SHA_SHORT="$(git rev-parse --short HEAD)"

if [ -z "$SOURCE_DIR" ]; then
    if [ -d "src/$MODULE" ]; then
        SOURCE_DIR="src/$MODULE"
    elif [ -d "$MODULE" ]; then
        SOURCE_DIR="$MODULE"
    fi
fi

if [ -n "$SOURCE_DIR" ] && [ -d "$SOURCE_DIR" ]; then
    MODULE_FINGERPRINT="sha256:$(
        find "$SOURCE_DIR" -type f -not -path '*/target/*' -not -path '*/node_modules/*' \
        | LC_ALL=C sort \
        | while IFS= read -r f; do sha256 "$f"; done \
        | { command -v shasum >/dev/null && shasum -a 256 || sha256sum; } \
        | awk '{print $1}'
    )"
else
    MODULE_FINGERPRINT="sha256:0000000000000000000000000000000000000000000000000000000000000000"
fi

BUNDLE_DIR="${OUT_ROOT}/${MODULE}/${GIT_SHA_SHORT}"

if [ "$DRY_RUN" = "1" ]; then
    echo "mint: DRY-RUN — would write to $BUNDLE_DIR"
    echo "  module:              $MODULE"
    echo "  git_sha:             $GIT_SHA_FULL"
    echo "  module_fingerprint:  $MODULE_FINGERPRINT"
    echo "  cucumber_json:       $CUCUMBER_JSON"
    echo "  videos_dir:          ${VIDEOS_DIR:-<none>}"
    echo "  screenshots_dir:     ${SCREENSHOTS_DIR:-<none>}"
    exit 0
fi

mkdir -p "$BUNDLE_DIR/videos" "$BUNDLE_DIR/screenshots"

# ---- copy cucumber report ----
cp "$CUCUMBER_JSON" "$BUNDLE_DIR/cucumber-report.json"

# ---- remux videos (WebM → MP4, lossless stream copy) ----
declare -a VIDEO_ARTIFACTS=()
if [ -n "$VIDEOS_DIR" ] && [ -d "$VIDEOS_DIR" ]; then
    while IFS= read -r -d '' webm; do
        name="$(basename "$webm" .webm)"
        out="$BUNDLE_DIR/videos/${name}.mp4"
        ffmpeg -y -loglevel error -i "$webm" -c copy "$out" \
            || { echo "mint: ffmpeg failed on $webm" >&2; exit 3; }
        VIDEO_ARTIFACTS+=("videos/${name}.mp4")
    done < <(find "$VIDEOS_DIR" -type f -name '*.webm' -print0)
fi

# ---- copy screenshots verbatim ----
declare -a SHOT_ARTIFACTS=()
if [ -n "$SCREENSHOTS_DIR" ] && [ -d "$SCREENSHOTS_DIR" ]; then
    rsync -a --exclude='.git' "$SCREENSHOTS_DIR/" "$BUNDLE_DIR/screenshots/" \
        2>/dev/null || cp -R "$SCREENSHOTS_DIR/." "$BUNDLE_DIR/screenshots/"
    while IFS= read -r -d '' shot; do
        rel="${shot#$BUNDLE_DIR/}"
        SHOT_ARTIFACTS+=("$rel")
    done < <(find "$BUNDLE_DIR/screenshots" -type f -print0)
fi

# ---- build manifest.json ----
ARTIFACTS_JSON="$(mktemp)"
{
    echo '['
    first=1
    add_entry() {
        local rel="$1"
        local abs="$BUNDLE_DIR/$rel"
        [ -f "$abs" ] || return 0
        local bytes hash
        bytes="$(wc -c < "$abs" | tr -d ' ')"
        hash="$(sha256 "$abs")"
        [ "$first" = 1 ] && first=0 || echo ','
        printf '  {"path":"%s","sha256":"%s","bytes":%s}' "$rel" "$hash" "$bytes"
    }
    add_entry "cucumber-report.json"
    for v in "${VIDEO_ARTIFACTS[@]:-}"; do [ -n "$v" ] && add_entry "$v"; done
    for s in "${SHOT_ARTIFACTS[@]:-}"; do [ -n "$s" ] && add_entry "$s"; done
    echo
    echo ']'
} > "$ARTIFACTS_JSON"

TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

jq -n \
    --arg ts "$TIMESTAMP" \
    --arg sha "$GIT_SHA_FULL" \
    --arg mod "$MODULE" \
    --arg fp "$MODULE_FINGERPRINT" \
    --slurpfile arts "$ARTIFACTS_JSON" \
    '{
       schema_version: 1,
       generated_at: $ts,
       git_sha: $sha,
       module: $mod,
       module_fingerprint: $fp,
       artifacts: $arts[0],
       runtime: {
         cucumber_version: (env.CUCUMBER_VERSION // "unknown"),
         playwright_version: (env.PLAYWRIGHT_VERSION // "unknown"),
         runner: (env.BDD_RUNNER // "cucumber-js")
       }
     }' > "$BUNDLE_DIR/manifest.json"

rm -f "$ARTIFACTS_JSON"

# ---- report.html ----
python3 - "$BUNDLE_DIR" "$MODULE" "$GIT_SHA_SHORT" <<'PY' > "$BUNDLE_DIR/report.html" || true
import json, os, sys
bundle, module, sha = sys.argv[1], sys.argv[2], sys.argv[3]
manifest = json.load(open(os.path.join(bundle, "manifest.json")))
print(f"""<!doctype html>
<meta charset="utf-8">
<title>{module} @ {sha} — certification bundle</title>
<style>
  body {{ font-family: system-ui, sans-serif; max-width: 960px; margin: 2rem auto; padding: 0 1rem; }}
  h1 {{ font-size: 1.4rem }}
  .meta {{ color: #666; font-size: 0.9rem }}
  video {{ display: block; margin: 1rem 0; max-width: 100%; }}
  code {{ background: #f4f4f4; padding: 0.1rem 0.3rem; border-radius: 3px }}
</style>
<h1>{module} — certification bundle</h1>
<p class="meta">Commit <code>{manifest['git_sha']}</code> · Generated {manifest['generated_at']}</p>
<h2>Videos</h2>""")
for a in manifest["artifacts"]:
    if a["path"].startswith("videos/"):
        print(f'<video controls src="{a["path"]}" preload="metadata"></video>')
        print(f'<p class="meta">{a["path"]} — sha256 <code>{a["sha256"][:16]}…</code> ({a["bytes"]} bytes)</p>')
print("<h2>Screenshots</h2>")
for a in manifest["artifacts"]:
    if a["path"].startswith("screenshots/"):
        print(f'<p><a href="{a["path"]}">{a["path"]}</a> — <code>{a["sha256"][:16]}…</code></p>')
print(f'<h2>manifest.json</h2><pre>{json.dumps(manifest, indent=2)}</pre>')
PY

echo "mint: bundle written to $BUNDLE_DIR"
echo "mint: git_sha=$GIT_SHA_FULL module_fingerprint=$MODULE_FINGERPRINT"
