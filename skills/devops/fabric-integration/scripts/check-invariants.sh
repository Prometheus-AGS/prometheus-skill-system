#!/usr/bin/env bash
# check-invariants.sh — enforce the fabric version invariants.
#
# Usage:
#   check-invariants.sh [--json] [--allowlist <file>]
#
# Exit: 0 all enforced invariants hold and the allowlist is exact
#       1 usage
#       2 an invariant is violated and NOT allowlisted
#       3 an allowlisted violation has been FIXED — remove the stale entry
#
# WHY EXIT 3 EXISTS
# A quarantine that never shrinks is a suppressed check. Three of the four
# invariants hold today and are enforced outright; the fourth (WIT world
# version) is already violated, so gating on it would block every PR for a
# pre-existing condition. It is therefore allowlisted — but the allowlist is
# itself enforced in BOTH directions: an un-allowlisted violation fails (2), and
# an allowlisted entry that no longer reproduces also fails (3), forcing the
# entry to be deleted rather than lingering as permanent permission.
#
# WHY A MISSING REPO IS NOT A PASS
# Three invariants compare versions ACROSS repositories that may not be checked
# out. An absent repo makes the invariant UNVERIFIABLE, which is reported as
# SKIP and is never counted as a hold. Reporting "aligned" because a file could
# not be read is how a check becomes decorative.
#
# bash 3.2 compatible. No LLM calls, no network.
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PACK="$(cd "$HERE/../../../.." && pwd)"
ALLOWLIST="$HERE/../assets/known-violations.json"
JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json)      JSON=1; shift ;;
    --allowlist) ALLOWLIST="${2:-}"; shift 2 ;;
    *) echo "usage: $0 [--json] [--allowlist <file>]" >&2; exit 1 ;;
  esac
done
command -v python3 >/dev/null 2>&1 || { echo "[fabric] ERROR: python3 required" >&2; exit 1; }

# External repos are siblings of the pack by default; overridable for CI.
FRF="${FRF_ROOT:-$(dirname "$PACK")/flint-realtime-fabric}"
UAR="${UAR_ROOT:-$(dirname "$PACK")/universal-agent-runtime}"
KM="${KNOWME_ROOT:-$(dirname "$(dirname "$PACK")")/know-me/know-me-system}"

PACK="$PACK" FRF="$FRF" UAR="$UAR" KM="$KM" ALLOWLIST="$ALLOWLIST" JSON="$JSON" python3 <<'PY'
import json, os, re, sys, glob

pack, frf, uar, km = (os.environ[k] for k in ("PACK", "FRF", "UAR", "KM"))
allow_path, want_json = os.environ["ALLOWLIST"], os.environ["JSON"] == "1"

def dep_version(path, name):
    """Return the version string declared for `name`, or None if unreadable."""
    try:
        txt = open(path, encoding="utf-8", errors="replace").read()
    except Exception:
        return None
    m = re.search(r'^\s*%s\s*=\s*"([^"]+)"' % re.escape(name), txt, re.M)
    if m:
        return m.group(1)
    m = re.search(r'^\s*%s\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"' % re.escape(name), txt, re.M | re.S)
    return m.group(1) if m else None

def minor(v):
    p = (v or "").lstrip("^~=><! ").split(".")
    return ".".join(p[:2]) if len(p) >= 2 else (p[0] if p else "")

def major(v):
    p = (v or "").lstrip("^~=><! ").split(".")
    return p[0] if p else ""

def ge(v, floor):
    def t(x):
        return tuple(int(n) for n in re.findall(r"\d+", x or "")[:3] or [0])
    return t(v) >= t(floor)

results = []

# ── 1. Loro minor aligned ────────────────────────────────────────────────────
a = dep_version(os.path.join(frf, "Cargo.toml"), "loro")
b = dep_version(os.path.join(pack, "substrate/storage-provider/Cargo.toml"), "loro")
if a is None or b is None:
    results.append(("loro-minor-aligned", "SKIP",
                    "unverifiable: %s" % ("flint-realtime-fabric not present" if a is None else "storage-provider Cargo.toml unreadable")))
else:
    ok = minor(a) == minor(b)
    results.append(("loro-minor-aligned", "PASS" if ok else "FAIL",
                    "frf=%s pack=%s (minor %s vs %s)" % (a, b, minor(a), minor(b))))

# ── 2. wasmtime major aligned ────────────────────────────────────────────────
a = dep_version(os.path.join(uar, "Cargo.toml"), "wasmtime")
b = dep_version(os.path.join(km, "rust/crates/knowme_plugin_host/Cargo.toml"), "wasmtime")
if a is None or b is None:
    results.append(("wasmtime-major-aligned", "SKIP",
                    "unverifiable: %s not present" % ("universal-agent-runtime" if a is None else "know-me-system")))
else:
    ok = major(a) == major(b)
    results.append(("wasmtime-major-aligned", "PASS" if ok else "FAIL",
                    "uar=%s knowme=%s (major %s vs %s)" % (a, b, major(a), major(b))))

# ── 3. iroh floor >= 1.0.2 ───────────────────────────────────────────────────
# In-repo, so it is always verifiable. 1.0.2 fixed a relay DoS where one
# malformed datagram from any client crashed an entire relay.
bad = []
for f in sorted(glob.glob(os.path.join(pack, "substrate/*/Cargo.toml"))):
    v = dep_version(f, "iroh")
    if v and not ge(v, "1.0.2"):
        bad.append("%s=%s" % (os.path.basename(os.path.dirname(f)), v))
results.append(("iroh-floor-1.0.2", "FAIL" if bad else "PASS",
                ", ".join(bad) if bad else "all substrate crates >= 1.0.2"))

# ── 4. WIT world version pinned ──────────────────────────────────────────────
# Already violated: knowme:plugin is declared at two versions at once.
pkgs = {}
for root in (uar, km):
    if not os.path.isdir(root):
        continue
    for dp, _dn, fn in os.walk(root):
        if "target" in dp or "node_modules" in dp:
            continue
        for f in fn:
            if not f.endswith(".wit"):
                continue
            try:
                txt = open(os.path.join(dp, f), encoding="utf-8", errors="replace").read()
            except Exception:
                continue
            for m in re.finditer(r"^package\s+([a-z0-9-]+:[a-z0-9-]+)@([0-9.]+)\s*;", txt, re.M):
                pkgs.setdefault(m.group(1), set()).add(m.group(2))
if not pkgs:
    results.append(("wit-world-version-pinned", "SKIP",
                    "unverifiable: no .wit files reachable (external repos absent)"))
else:
    dup = {k: sorted(v) for k, v in pkgs.items() if len(v) > 1}
    results.append(("wit-world-version-pinned", "FAIL" if dup else "PASS",
                    "; ".join("%s at %s" % (k, ", ".join(v)) for k, v in dup.items())
                    if dup else "every WIT package declares one version"))

# ── Allowlist reconciliation ─────────────────────────────────────────────────
try:
    allow = json.load(open(allow_path))
except Exception:
    allow = {"known_violations": []}
allowed = {e["invariant"]: e for e in allow.get("known_violations", [])}

failed_unallowed, fixed_but_allowlisted = [], []
for name, status, detail in results:
    if status == "FAIL" and name not in allowed:
        failed_unallowed.append((name, detail))
    if status == "PASS" and name in allowed:
        fixed_but_allowlisted.append(name)

if want_json:
    print(json.dumps({
        "results": [{"invariant": n, "status": s, "detail": d} for n, s, d in results],
        "allowlisted": sorted(allowed),
        "failed_unallowed": [n for n, _ in failed_unallowed],
        "fixed_but_allowlisted": fixed_but_allowlisted,
    }, indent=2))
else:
    print("Fabric version invariants")
    print("-------------------------")
    for n, s, d in results:
        mark = {"PASS": "PASS", "FAIL": "FAIL", "SKIP": "SKIP"}[s]
        note = "  [allowlisted]" if (s == "FAIL" and n in allowed) else ""
        print("  %-4s %-26s %s%s" % (mark, n, d, note))
    if allowed:
        print("\nKnown violations (quarantined, must shrink):")
        for n, e in sorted(allowed.items()):
            print("  - %s: %s" % (n, e.get("reason", "(no reason recorded)")))

if failed_unallowed:
    print("\nFAILED: violated and not allowlisted:", file=sys.stderr)
    for n, d in failed_unallowed:
        print("  %s — %s" % (n, d), file=sys.stderr)
    raise SystemExit(2)

if fixed_but_allowlisted:
    print("\nFAILED: these are FIXED but still allowlisted. Remove them from %s:" % allow_path, file=sys.stderr)
    for n in fixed_but_allowlisted:
        print("  %s" % n, file=sys.stderr)
    raise SystemExit(3)

raise SystemExit(0)
PY
