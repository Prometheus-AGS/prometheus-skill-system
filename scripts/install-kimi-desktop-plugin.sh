#!/usr/bin/env bash
# install-kimi-desktop-plugin.sh — publish the pack as a Kimi Desktop plugin package.
#
# Kimi Desktop (the Electron app, /Applications/Kimi.app) does NOT read a flat
# skills directory the way Claude Code / Cursor / Kimi Code do. Its agent runtime
# ("daimon") loads skills only from inside a plugin package:
#
#   <daimon-share>/daimon/plugin-packages/<pkg>/
#       kimi.plugin.json          manifest; `skills` points at a directory
#       skills/<skill-name>/SKILL.md
#
# Shape confirmed against the vendor-installed `github` package, whose manifest
# declares  "skills": "./skills/"  — a directory, so one package can carry the
# whole pack rather than one package per skill.
#
# Skills are COPIED, not symlinked. plugin-packages is app-managed state with
# version pinning (plugin-host/release-pins.v2.json); a symlink farm there is
# more likely to be pruned than a self-contained directory. Copies go stale, so
# this script is idempotent and re-run by install-skills-flat.sh.
#
# Usage:
#   bash scripts/install-kimi-desktop-plugin.sh [--uninstall] [--dry-run]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PKG_NAME="prometheus-skill-pack"
DAIMON="$HOME/Library/Application Support/kimi-desktop/daimon-share/daimon"
PLUGIN_PACKAGES="$DAIMON/plugin-packages"
DEST="$PLUGIN_PACKAGES/$PKG_NAME"

UNINSTALL=false
DRY_RUN=false
for arg in "$@"; do
    case "$arg" in
        --uninstall) UNINSTALL=true ;;
        --dry-run)   DRY_RUN=true ;;
        *) echo "Unknown argument: $arg" >&2; exit 2 ;;
    esac
done

# Kimi Desktop may simply not be installed. That is not an error — the pack
# supports 14 targets and this is one optional surface.
if [ ! -d "$PLUGIN_PACKAGES" ]; then
    echo "  — kimi-desktop: plugin-packages not found, skipping"
    echo "    (expected at $PLUGIN_PACKAGES)"
    exit 0
fi

if $UNINSTALL; then
    if [ -d "$DEST" ]; then
        $DRY_RUN || rm -rf "$DEST"
        echo "  ✅ kimi-desktop: removed $PKG_NAME"
    else
        echo "  — kimi-desktop: $PKG_NAME not installed"
    fi
    exit 0
fi

VERSION="$(python3 -c "import json;print(json.load(open('$REPO_ROOT/skill-system.json'))['releaseVersion'])" 2>/dev/null || echo "0.0.0")"
DIST_SKILLS="$REPO_ROOT/dist/plugins/claude/prometheus-skill-pack/skills"
[ -d "$DIST_SKILLS" ] || { echo "Generated skill distribution is missing: $DIST_SKILLS" >&2; exit 1; }

if $DRY_RUN; then
    echo "  → [dry-run] would install $PKG_NAME v$VERSION to $DEST"
    exit 0
fi

# Stage into a temp dir and swap, so an interrupted run never leaves Kimi
# reading a half-populated package.
STAGE="$(mktemp -d "${TMPDIR:-/tmp}/kimi-pkg.XXXXXX")"
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/skills"

COUNT=0
while IFS= read -r -d '' skill_md; do
    skill_dir="$(dirname "$skill_md")"
    name="$(python3 - "$skill_md" <<'PY'
import re, sys, pathlib
p = pathlib.Path(sys.argv[1])
t = p.read_text(encoding="utf-8", errors="replace")
m = re.match(r"^---\n(.*?)\n---", t, re.S)
n = re.search(r"^name:\s*['\"]?([^'\"\n]+)['\"]?", m.group(1), re.M) if m else None
print((n.group(1).strip() if n else p.parent.name))
PY
)"
    [ -n "$name" ] || continue
    # First writer wins on a duplicate name, matching the other installers.
    [ -e "$STAGE/skills/$name" ] && continue
    mkdir -p "$STAGE/skills/$name"
    cp -R "$skill_dir/." "$STAGE/skills/$name/"
    COUNT=$((COUNT + 1))
# The generated distribution is already flattened and includes the two explicit
# imported-skill roots while excluding fixtures, tests, and duplicate imports.
done < <(find "$DIST_SKILLS" -mindepth 2 -maxdepth 2 -name SKILL.md -print0)

python3 - "$STAGE" "$VERSION" "$COUNT" "$REPO_ROOT" <<'PY'
import json, sys, pathlib
stage, version, count = pathlib.Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])
repo_root = pathlib.Path(sys.argv[4])

# --- mcpServers -------------------------------------------------------------
# Built from scripts/mcp-port-table.json, the pack's source of truth for MCP
# connectivity — NOT hardcoded. A machine running a service on a different port
# then gets a correct manifest from one reinstall, which is what goal 4 asks for.
#
# The contract was read out of the shipped loader
# (agent-core/dist/index.mjs -> readMcpServers / McpServerConfigSchema), not
# inferred from the vendor packages, which turned out to be a biased sample:
#
#   url: z.string().url()          -> NO scheme or host restriction. Loopback
#                                     http:// is accepted; all three vendor
#                                     packages happening to be remote HTTPS was
#                                     a coincidence, not a rule.
#   transport: "http" | "sse"      -> both first-class. surreal-memory's legacy
#                                     two-channel SSE has a declared transport.
#   headers, bearerTokenEnvVar     -> authenticated servers ARE expressible.
#
# stdio entries are deliberately skipped: normalizePluginMcpServer rejects any
# command containing "/" or an absolute path, requiring a PATH command or a
# "./" path inside the plugin root. Shipping shims for them is deferred work
# (analyze D2), not this change.
def build_mcp_servers():
    table = repo_root / "scripts/mcp-port-table.json"
    if not table.is_file():
        return None
    services = json.loads(table.read_text(encoding="utf-8")).get("services", {})
    out = {}
    for name, cfg in services.items():
        kind = cfg.get("type")
        if kind not in ("http", "sse"):
            continue          # stdio: needs a shim; out of scope for this change
        url = cfg.get("url")
        if not url:
            continue
        entry = {"transport": kind, "url": url}
        # bearerTokenEnvVar names an env var the daimon reads at connect time, so
        # a token is never written into the manifest — no secret can reach
        # app-managed state through this path.
        if cfg.get("authEnvVar"):
            entry["bearerTokenEnvVar"] = cfg["authEnvVar"]
        elif cfg.get("requiresAuth"):
            # Declared as needing auth but with no env var named. Emitting it
            # anyway would ship an entry guaranteed to 401 on every connect —
            # a tool that appears in the UI and always fails is worse than an
            # absent one. forge-rs is in this state today.
            continue
        out[name] = entry
    return out or None

mcp_servers = build_mcp_servers()

manifest = {
    "$schema": "https://kimi.com/schemas/kimi.plugin.schema.json",
    "name": "prometheus-skill-pack",
    "version": version,
    "description": (
        f"Prometheus Skill Pack — {count} engineering skills spanning the KBD "
        "lifecycle, Feynman learning loop, Rust/React/DevOps patterns, BDD "
        "testing, and adversarial review."
    ),
    "keywords": ["prometheus", "kbd", "skills", "rust", "react",
                 "devops", "testing", "learning", "adversarial-review"],
    "author": "Prometheus AGS",
    "homepage": "https://github.com/Prometheus-AGS/prometheus-skill-system",
    "license": "MIT",
    "skillInstructions": (
        "The Prometheus Skill Pack provides structured engineering workflows.\n\n"
        "Key families:\n"
        "- kbd-*: the Knowledge-Based Development lifecycle "
        "(assess -> analyze -> plan -> execute -> reflect)\n"
        "- learn-*: the Feynman learning loop with anti-sycophantic grading\n"
        "- adversarial-review: cross-model review where the judge is never the producer\n"
        "- Language patterns: Rust, TypeScript, Go, Python, Swift, Kotlin\n"
        "- bdd-*: behaviour-driven testing with immutable-test rules\n\n"
        "Each skill is a SKILL.md with YAML frontmatter carrying `name` and "
        "`description`. Read a skill in full before following it. Prefer a "
        "matching skill over improvising when one applies to the task."
    ),
    "interface": {
        "displayName": "Prometheus Skill Pack",
        "shortDescription": f"{count} engineering skills: KBD lifecycle, learning loop, language patterns",
        "longDescription": (
            "Enterprise skill collection for AI-assisted development, portable "
            "across Claude Code, Codex, OpenCode, Kimi, MiniMax, Cursor and more.\n\n"
            "Covers the full KBD lifecycle, the Feynman learning loop with "
            "sycophancy-corrected grading, adversarial cross-model review, "
            "language-specific patterns, and BDD testing with signed "
            "certification bundles.\n\n"
            "Skills are instructions a model reads — no process spawning is "
            "required for the manifest-only majority, so they work on any harness."
        ),
        "developerName": "Prometheus AGS",
        "websiteURL": "https://github.com/Prometheus-AGS/prometheus-skill-system",
        "category": "DEVELOPER",
    },
    "skills": "./skills/",
}
if mcp_servers:
    manifest["mcpServers"] = mcp_servers
(stage / "kimi.plugin.json").write_text(
    json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
(stage / "README.md").write_text(
    f"# Prometheus Skill Pack\n\n{count} skills for Kimi Desktop.\n\n"
    "Generated by `scripts/install-kimi-desktop-plugin.sh` — do not edit here.\n"
    "Edit the canonical skill under `skills/` in the repository and re-run the\n"
    "installer; this directory is overwritten on every run.\n",
    encoding="utf-8")
PY

# Validate before swapping: a malformed manifest would break plugin loading.
python3 -c "
import json,sys
m=json.load(open('$STAGE/kimi.plugin.json'))
for k in ('name','version','skills','interface'):
    assert k in m, f'manifest missing {k}'
assert m['skills']=='./skills/', 'skills must point at the skills directory'
" || { echo "  ❌ kimi-desktop: manifest validation failed" >&2; exit 1; }

rm -rf "$DEST"
mkdir -p "$(dirname "$DEST")"
mv "$STAGE" "$DEST"
trap - EXIT

echo "  ✅ kimi-desktop: installed $PKG_NAME v$VERSION with $COUNT skills"
echo "     $DEST"
