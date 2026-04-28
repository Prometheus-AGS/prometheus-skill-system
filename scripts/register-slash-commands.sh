#!/usr/bin/env bash
# Register prometheus-skill-pack skills as slash commands in opencode and codex.
#
# OpenCode: injects command entries into ~/.opencode/opencode.json
# Codex:    creates prompt files in ~/.codex/prompts/ (one per skill)
#
# Usage:
#   ./scripts/register-slash-commands.sh              # install
#   ./scripts/register-slash-commands.sh --uninstall  # remove

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UNINSTALL=false
if [[ "${1:-}" == "--uninstall" ]]; then
    UNINSTALL=true
fi

echo "🔥 Prometheus Skill Pack — Slash Command Registration"
echo "====================================================="
$UNINSTALL && echo "  Mode: UNINSTALL" || echo "  Mode: INSTALL"
echo ""

python3 << PYEOF
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

HOME = Path.home()
REPO_ROOT = Path("$REPO_ROOT")
UNINSTALL = "$UNINSTALL" == "true"

# ── collect skills ────────────────────────────────────────────────────────────
result = subprocess.run(
    ["find", str(REPO_ROOT / "skills"), "-name", "SKILL.md", "-not", "-path", "*/imported/*"],
    capture_output=True, text=True,
)

skills = {}
for path_str in sorted(result.stdout.strip().split("\n")):
    if not path_str:
        continue
    skill_md = Path(path_str)
    skill_name = skill_md.parent.name
    content = skill_md.read_text()
    if not content.startswith("---"):
        continue
    try:
        end = content.index("---", 3)
        fm = content[3:end]
        m = re.search(r"description:\s*>?\s*\n?(.*?)(?=\n[a-z]|\Z)", fm, re.DOTALL)
        desc = ""
        if m:
            desc = " ".join(m.group(1).split()).strip("'\"")[:200]
        skills[skill_name] = desc or f"Prometheus skill: {skill_name}"
    except Exception as e:
        print(f"  WARN: could not parse {skill_md}: {e}", file=sys.stderr)

print(f"  Found {len(skills)} skills")

# ── opencode.json ─────────────────────────────────────────────────────────────
OPENCODE_JSON = HOME / ".opencode" / "opencode.json"
if OPENCODE_JSON.exists():
    backup = str(OPENCODE_JSON) + ".bak." + datetime.now().strftime("%Y%m%d-%H%M%S")
    shutil.copy(OPENCODE_JSON, backup)

    with open(OPENCODE_JSON) as f:
        config = json.load(f)

    cmds = config.setdefault("command", {})

    if UNINSTALL:
        removed = 0
        for skill_name in list(skills.keys()):
            if skill_name in cmds and f"skills/{skill_name}/SKILL.md" in cmds[skill_name].get("template", ""):
                del cmds[skill_name]
                removed += 1
        # restore evolve-instincts → evolve if present
        if "evolve-instincts" in cmds and "evolve" not in cmds:
            cmds["evolve"] = {k: v for k, v in cmds["evolve-instincts"].items() if k != "description"}
            cmds["evolve"]["description"] = "Cluster ECC instincts into skills"
            del cmds["evolve-instincts"]
        print(f"  opencode: removed {removed} commands")
    else:
        # rename existing ECC evolve to avoid collision
        if "evolve" in cmds and f"skills/evolve/SKILL.md" not in cmds["evolve"].get("template", ""):
            if "evolve-instincts" not in cmds:
                cmds["evolve-instincts"] = {**cmds["evolve"], "description": "Cluster ECC instincts into skills (evolve-instincts)"}
            del cmds["evolve"]

        added = skipped = 0
        for skill_name, desc in sorted(skills.items()):
            if skill_name in cmds and f"skills/{skill_name}/SKILL.md" in cmds[skill_name].get("template", ""):
                skipped += 1
                continue
            cmds[skill_name] = {
                "description": desc,
                "template": f"{{file:skills/{skill_name}/SKILL.md}}\n\n\$ARGUMENTS",
            }
            added += 1
        print(f"  opencode: added {added}, skipped {skipped} (total {len(cmds)} commands)")

    config["command"] = cmds
    with open(OPENCODE_JSON, "w") as f:
        json.dump(config, f, indent=2)
        f.write("\n")
else:
    print("  SKIP opencode: ~/.opencode/opencode.json not found")

# ── codex prompts ─────────────────────────────────────────────────────────────
PROMPTS_DIR = HOME / ".codex" / "prompts"
if PROMPTS_DIR.parent.exists():
    PROMPTS_DIR.mkdir(exist_ok=True)
    count = 0
    if UNINSTALL:
        for skill_name in skills:
            pf = PROMPTS_DIR / f"{skill_name}.md"
            if pf.exists() and f"/.codex/skills/{skill_name}/SKILL.md" in pf.read_text():
                pf.unlink()
                count += 1
        print(f"  codex: removed {count} prompt files")
    else:
        for skill_name, desc in sorted(skills.items()):
            pf = PROMPTS_DIR / f"{skill_name}.md"
            skill_path = HOME / ".codex" / "skills" / skill_name / "SKILL.md"
            content = f"""---
description: {desc}
argument-hint: task description or arguments
---

Follow the instructions in \`{skill_path}\` to execute this skill.

\$ARGUMENTS
"""
            pf.write_text(content)
            count += 1
        print(f"  codex: wrote {count} prompt files")
else:
    print("  SKIP codex: ~/.codex not found")

print("")
if UNINSTALL:
    print("✨ Slash commands unregistered")
else:
    print("✨ Slash commands registered")
    print("   opencode: /kbd-plan, /evolve, /gitops-bootstrap, ... (53 new commands)")
    print("   codex:    /kbd-plan, /evolve, /gitops-bootstrap, ... (53 new prompts)")
PYEOF
