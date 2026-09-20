#!/usr/bin/env bash
# Forbidden edges of feature-based clean architecture, in every layer, plus kebab-case names under web/.
# Tolerant of layers that do not exist yet. ESLint (web) and crate boundaries (Rust) are the primary
# enforcement; this is the same contract as one repo-wide command, and the only check for Dart.
#
#   scripts/check-architecture.sh [root]      exit 0 clean · 1 violations · 2 could not run
set -uo pipefail
command -v python3 >/dev/null 2>&1 || { echo "check-architecture: python3 is required" >&2; exit 2; }
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"   # the project this script belongs to, not the caller's cwd
root="${1:-$(git -C "$here" rev-parse --show-toplevel 2>/dev/null || echo "$here")}"

exec python3 - "$root" <<'PY'
import re, sys
from pathlib import Path

root = Path(sys.argv[1]).resolve()
violations: list[str] = []

def report(path: Path, line_no: int, rule: str, text: str) -> None:
    violations.append(f"{path.relative_to(root)}:{line_no}: {rule}\n      {text.strip()[:140]}")

def lines_of(path: Path):
    try:
        return list(enumerate(path.read_text(errors="replace").splitlines(), 1))
    except OSError:
        return []

def feature_and_layer(path: Path, features_dir: Path):
    parts = path.relative_to(features_dir).parts
    return (parts[0], parts[1] if len(parts) > 2 else "") if parts else ("", "")

SKIP_DIRS = {"node_modules", "dist", "build", "target", ".dart_tool", ".next", "coverage"}
def walk(base: Path, suffixes: tuple[str, ...]):
    if not base.is_dir():
        return
    for p in base.rglob("*"):
        if p.is_file() and p.suffix in suffixes and not (SKIP_DIRS & set(p.parts)):
            yield p

# ---------------------------------------------------------------- web: Component → Hook → Store → Service
for web_src in [p for p in root.glob("*/src") if (p.parent / "package.json").is_file()]:
    features = web_src / "features"
    IMPORT = re.compile(r"""(?:from\s+|import\s*\(\s*|require\s*\(\s*)['"]([^'"]+)['"]""")
    IO = re.compile(r"\bfetch\s*\(|new\s+EventSource\b|new\s+WebSocket\b|\bXMLHttpRequest\b")
    for f in walk(features, (".ts", ".tsx", ".mts")):
        feature, layer = feature_and_layer(f, features)
        if layer == "tests" or ".test." in f.name or ".spec." in f.name:
            continue
        for n, text in lines_of(f):
            if layer != "services" and IO.search(text) and not text.lstrip().startswith(("//", "*")):
                report(f, n, "network I/O outside services/ (B-4: services own all external communication)", text)
            for target in IMPORT.findall(text):
                if target.startswith("@prometheus-ags/") and layer not in ("stores", "services", "entities"):
                    report(f, n, f"PEM imported from {layer or 'feature root'}/ — only stores/, services/, entities/", text)
                if re.match(r"@ag-ui/|@supabase/", target) and layer != "services":
                    report(f, n, "transport client imported outside services/", text)
                if layer in ("components", "pages") and re.search(r"(^|/)(stores|services)(/|$)", target):
                    report(f, n, "component imports a store or service — components import hooks only", text)
                if layer == "hooks" and re.search(r"(^|/)services(/|$)", target):
                    report(f, n, "hook imports a service — hooks import stores only", text)
                if layer in ("stores", "services") and re.search(r"(^|/)(components|pages|hooks)(/|$)", target):
                    report(f, n, f"{layer}/ imports UI — dependencies point inward", text)
                if layer == "services" and re.search(r"(^|/)stores(/|$)|^react(-dom)?$|^zustand", target):
                    report(f, n, "service imports a store or a UI framework — services are framework-free", text)
                other = re.search(r"features/([a-z0-9-]+)", target)
                if other and other.group(1) != feature:
                    report(f, n, f"cross-feature import ({feature} → {other.group(1)}) — share through app/ or shared/", text)
    # kebab-case file and directory names across the whole package, not only src/
    # (vendored registry components keep upstream names; a few tool-mandated names are conventional)
    KEBAB = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*(\.[a-z0-9-]+)*$")
    VENDORED = ("src/components/ui", "src/components/assistant-ui")
    CONVENTIONAL = {"README.md", "AGENTS.md", "CLAUDE.md", "CHANGELOG.md", "LICENSE", "Dockerfile"}
    package = web_src.parent
    for p in package.rglob("*"):
        rel = p.relative_to(package).as_posix()
        if SKIP_DIRS & set(p.parts) or rel.startswith(VENDORED) or ".gen." in p.name:
            continue
        if p.name.startswith(".") or any(part.startswith(".") for part in p.relative_to(package).parts):
            continue
        if p.name in CONVENTIONAL or KEBAB.match(p.name):
            continue
        violations.append(f"{p.relative_to(root)}: file and directory names under {package.name}/ are kebab-case (identifiers keep their case)")

# ---------------------------------------------------------------- Rust: interface → application → domain ← infrastructure
FRAMEWORK = re.compile(r"^\s*(?:pub\s+)?use\s+(axum|reqwest|sqlx|tokio|hyper|tower|tower_http|tonic)\b")
for features in root.glob("**/src/features"):
    if SKIP_DIRS & set(features.parts) or not list(features.rglob("*.rs")):
        continue
    for f in walk(features, (".rs",)):
        feature, layer = feature_and_layer(f, features)
        layer = layer.removesuffix(".rs")
        for n, text in lines_of(f):
            if layer == "domain" and FRAMEWORK.search(text):
                report(f, n, "framework or I/O crate used in domain/ — domain is pure", text)
            use = re.search(r"\b(?:crate|super(?:::super)*)::features::([a-z0-9_]+)(?:::([a-z_]+))?", text)
            if use:
                if use.group(1) != feature:
                    report(f, n, f"cross-feature use ({feature} → {use.group(1)}) — share through core/ or shared/", text)
                elif layer == "domain" and use.group(2) in ("application", "infrastructure", "interface"):
                    report(f, n, "domain depends outward — dependencies point inward", text)
                elif layer == "application" and use.group(2) == "interface":
                    report(f, n, "application depends on interface", text)
                elif layer == "infrastructure" and use.group(2) == "interface":
                    report(f, n, "infrastructure depends on interface", text)
for cargo in root.glob("**/crates/*_domain/Cargo.toml"):
    for n, text in lines_of(cargo):
        if re.match(r"\s*(axum|reqwest|sqlx|tokio|hyper|tower)\b", text):
            report(cargo, n, "domain crate depends on a framework or I/O crate", text)

# ---------------------------------------------------------------- Dart: presentation → domain ← data
for features in root.glob("*/lib/features"):
    for f in walk(features, (".dart",)):
        if f.name.endswith((".g.dart", ".freezed.dart")):
            continue
        feature, layer = feature_and_layer(f, features)
        for n, text in lines_of(f):
            m = re.match(r"\s*import\s+['\"]([^'\"]+)['\"]", text)
            if not m:
                continue
            target = m.group(1)
            if layer == "presentation" and re.search(r"(^|/)data/", target):
                report(f, n, "presentation imports data — go through a domain interface", text)
            if layer == "domain" and re.search(r"package:flutter/|riverpod|(^|/)(data|presentation)/", target):
                report(f, n, "domain imports Flutter, Riverpod, data or presentation — domain is pure", text)
            other = re.search(r"features/([a-z0-9_]+)/", target)
            if other and other.group(1) != feature:
                report(f, n, f"cross-feature import ({feature} → {other.group(1)})", text)

if violations:
    print("\n".join(violations))
    print(f"\ncheck-architecture: {len(violations)} violation(s). Rules: .claude/rules/architecture.md", file=sys.stderr)
    sys.exit(1)
print("check-architecture: clean")
PY
