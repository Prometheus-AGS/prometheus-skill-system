#!/usr/bin/env bash
# Generated from verification.md for change-tlm-003-revised-implementation-plan. Run by `kbd-apply verify` via tasks.json.
# Repository files are checked in the change's worktree; phase evidence lives in the main checkout.
set -euo pipefail
ROOT="${TLM_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlm-design}"
cd "$ROOT"
# Base branch: must contain origin/main 20d97f2 (PRs #111, #121-#123).
git merge-base --is-ancestor 20d97f2 HEAD || { echo "base branch does not contain 20d97f2" >&2; exit 1; }
python3 - <<'PY'
import re
t=open('docs/plans/team-aware-learning-memory-implementation.md').read()
prs=re.split(r'(?m)^### PR ',t)[1:]
assert len(prs)>=10, f'expected at least 10 PRs, got {len(prs)}'
for p in prs:
    head=p.splitlines()[0]
    for f in ['**Repo:**','**Files:**','**Design section:**','**Depends on:**','**Integration gate:**']:
        assert f in p, f'PR {head!r} lacks {f}'
def idx(pat):
    for i,p in enumerate(prs):
        if re.search(pat,p.splitlines()[0],re.I): return i
    raise AssertionError(f'no PR matching {pat}')
assert idx('pk|prometheus-knowledge')<idx(r'pin bump|v1\.10')<idx('identity')
assert idx('surreal')<idx(r'pin bump|v1\.10')
for need in ['alpha','beta','Claude Code','Codex','bytes']:
    assert need in t, f'missing: {need}'
for pat in [r'matcher',r'prometheus-skills-mini',r'Cortex|CLAUDE\.md memory']:
    idx(pat)
print('ok')
PY
grep -q "team-aware-learning-memory-implementation.md" "$HOME/.claude/plans/yes-fix-it-in-logical-mccarthy.md"
test -f /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator/phases/team-aware-learning-memory/evidence/parent-plan.before.md
python3 -c 'import os;a=open(os.path.expanduser("~/.claude/plans/yes-fix-it-in-logical-mccarthy.md")).read();b=open("/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator/phases/team-aware-learning-memory/evidence/parent-plan.before.md").read();assert a.startswith(b)'
echo "verify OK: change-tlm-003-revised-implementation-plan"
