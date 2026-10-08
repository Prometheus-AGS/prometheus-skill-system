# Verification — change-tlm-003-revised-implementation-plan

Repository: `prometheus-skill-pack`
Depends on: change-tlm-002-team-aware-learning-design, change-tlm-004-hook-timeout-seconds

## Acceptance criteria

- Every PR in the plan names a repo, files, the design section it implements, dependencies and an integration gate that runs a production entry point.
- The plan contains per-agent end-to-end gates for both harnesses (two-role team, delivered bytes within budget, addressed lesson crosses roles).
- Upstream pk and surreal-memory prerequisites precede the pack PRs that depend on them, followed by a pin bump.
- The session plan file references the revised plan.

## Verify commands

```verify
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
test -f .kbd-orchestrator/phases/team-aware-learning-memory/evidence/parent-plan.before.md
python3 -c 'import os;a=open(os.path.expanduser("~/.claude/plans/yes-fix-it-in-logical-mccarthy.md")).read();b=open(".kbd-orchestrator/phases/team-aware-learning-memory/evidence/parent-plan.before.md").read();assert a.startswith(b)'
```
