#!/usr/bin/env bash
# Measure the skill-listing description budget across EVERY scope the harness
# loads, and report how far over or under it the profile sits.
#
# Why this exists: a repo-local count is the wrong denominator. Measured on one
# estate 2026-08-09 — 56 repo skills, ~916 user, ~1294 plugin, 2266 total,
# ~652k description bytes ≈ ~163k tokens against a ~4k budget. About 41x over.
# A reading that counts only .claude/skills reports headroom that is not there.
#
# Descriptions past the budget are dropped silently: the skill keeps its name and
# stops auto-triggering. Eviction ranks by usage recency, so a newly installed
# skill scores zero and goes dark first.
#
# Exit: 0 within budget, 1 over budget, 2 could not measure.

set -uo pipefail

project_path="."
window=200000
json=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)   project_path="${2:?}"; shift 2 ;;
    --window) window="${2:?}"; shift 2 ;;
    --json)   json=1; shift ;;
    -h|--help)
      echo "Usage: skill-budget.sh [--path <root>] [--window <ctx-tokens>] [--json]"
      exit 0 ;;
    *) echo "skill-budget: unknown flag: $1" >&2; exit 2 ;;
  esac
done
[[ -d "$project_path" ]] || { echo "skill-budget: not a directory: $project_path" >&2; exit 2; }
project_path="$(cd "$project_path" && pwd)"

fraction="0.01"
settings="$project_path/.claude/settings.json"
if [[ -f "$settings" ]] && command -v jq >/dev/null 2>&1; then
  fraction="$(jq -r '.skillListingBudgetFraction // 0.01' "$settings" 2>/dev/null || echo 0.01)"
fi

# YAML frontmatter needs a real parser. `description: >` is a folded block
# scalar whose text continues on the following lines; grepping the first line
# reports a 1-char description and is simply wrong.
if ! command -v python3 >/dev/null 2>&1; then
  echo "SKIP  skill budget — python3 absent; frontmatter cannot be parsed correctly" >&2
  echo "      Do not substitute grep: 'description: >' is a folded scalar." >&2
  exit 2
fi

python3 - "$project_path" "$fraction" "$window" "$json" <<'PY'
import os, sys, glob

root, fraction, window, as_json = sys.argv[1], float(sys.argv[2]), int(sys.argv[3]), sys.argv[4] == "1"

try:
    import yaml
    have_yaml = True
except ImportError:
    have_yaml = False

def description_of(path):
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return ""
    if not text.startswith("---"):
        return ""
    end = text.find("\n---", 3)
    if end == -1:
        return ""
    fm = text[3:end]
    if have_yaml:
        try:
            data = yaml.safe_load(fm) or {}
            return str(data.get("description", "") or "")
        except Exception:
            pass
    # Fallback: hand-fold block scalars rather than reading one line.
    out, capture = [], False
    for line in fm.splitlines():
        if line.startswith("description:"):
            rest = line.split(":", 1)[1].strip()
            if rest in (">", "|", ">-", "|-", ""):
                capture = True
            else:
                return rest
            continue
        if capture:
            if line[:1] in (" ", "\t"):
                out.append(line.strip())
            else:
                break
    return " ".join(out)

scopes = [
    ("repo",    os.path.join(root, ".claude", "skills")),
    ("user",    os.path.expanduser("~/.claude/skills")),
    ("plugins", os.path.expanduser("~/.claude/plugins")),
]

rows, total_chars, total_skills, empties = [], 0, 0, 0
for name, path in scopes:
    if not os.path.isdir(path):
        rows.append((name, path, 0, 0, 0)); continue
    files = glob.glob(os.path.join(path, "**", "SKILL.md"), recursive=True)
    chars = 0; empty = 0
    for f in files:
        d = description_of(f)
        if not d.strip(): empty += 1
        chars += len(d)
    rows.append((name, path, len(files), chars, empty))
    total_chars += chars; total_skills += len(files); empties += empty

tokens = total_chars // 4
budget = int(window * fraction)
over = tokens > budget
ratio = (tokens / budget) if budget else float("inf")

if as_json:
    import json as J
    print(J.dumps({
        "skills": total_skills, "description_chars": total_chars,
        "est_tokens": tokens, "budget_tokens": budget,
        "fraction": fraction, "window": window,
        "over_budget": over, "ratio": round(ratio, 1),
        "empty_descriptions": empties,
        "scopes": [{"scope": n, "path": p, "skills": c, "chars": ch, "empty": e}
                   for n, p, c, ch, e in rows],
    }, indent=2))
else:
    print(f"{'scope':<10}{'skills':>8}{'desc chars':>13}{'empty':>7}   path")
    for n, p, c, ch, e in rows:
        print(f"{n:<10}{c:>8}{ch:>13}{e:>7}   {p}")
    print()
    print(f"TOTAL      {total_skills:>8}{total_chars:>13}{empties:>7}")
    print(f"estimated  ~{tokens} tokens of descriptions")
    print(f"budget     ~{budget} tokens  ({fraction} x {window})")
    print()
    if over:
        print(f"OVER BUDGET by ~{ratio:.1f}x.")
        print("Descriptions past the budget are dropped silently: the skill keeps its")
        print("name and stops auto-triggering. Eviction favours recently used skills,")
        print("so a newly installed skill goes dark first.")
        print()
        print("Raising skillListingBudgetFraction does not fix a multiple this large.")
        print("Gate the long tail behind plugins and enable one profile at a time.")
    else:
        print(f"Within budget ({ratio:.2f} of it used).")
    if empties:
        print(f"\n{empties} skill(s) have an empty description and cannot auto-trigger.")
    if not have_yaml:
        print("\nNote: PyYAML absent; used a folded-scalar fallback parser.")

sys.exit(1 if over else 0)
PY
