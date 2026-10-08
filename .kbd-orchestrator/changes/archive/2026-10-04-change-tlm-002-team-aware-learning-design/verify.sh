#!/usr/bin/env bash
# Generated from verification.md for change-tlm-002-team-aware-learning-design. Run by `kbd-apply verify` via tasks.json.
# Repository files are checked in the change's worktree; phase evidence lives in the main checkout.
set -euo pipefail
ROOT="${TLM_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlm-design}"
cd "$ROOT"
# Base branch: must contain origin/main 20d97f2 (PRs #111, #121-#123).
git merge-base --is-ancestor 20d97f2 HEAD || { echo "base branch does not contain 20d97f2" >&2; exit 1; }
python3 - <<'PY'
import re
t=open('docs/design/team-aware-learning-memory.md').read()
for n in range(1,11): assert re.search(rf'^## {n}\. ',t,re.M), f'section {n} missing'
assert len(re.findall(r'(?im)^\*\*rejected alternatives?\*\*',t))>=10, 'each section needs a Rejected alternatives block'
for need in ['8,000','2,000 tokens','total per-subagent','@project','<team>/@team','user_id','agent_id','categories','subagent-identity-probe','untrusted','exits 0']:
    assert need in t, f'missing: {need}'
assert re.search(r'(?is)\|\s*envelope field\s*\|.*surreal-memory.*\|.*pk.*\|.*karpathy',t), 'store mapping table missing'
import json
props=json.load(open('shared/schemas/learning-envelope.schema.json'))['properties']
table=t[t.lower().index('| envelope field'):]
for p in props: assert f'`{p}`' in table, f'envelope property {p} not mapped'
print('ok')
PY
python3 -c 'import json,jsonschema;s=json.load(open("shared/schemas/learning-envelope.schema.json"));v=jsonschema.Draft202012Validator(s);v.validate(json.load(open("shared/schemas/examples/learning-envelope.valid.json")));e=list(v.iter_errors(json.load(open("shared/schemas/examples/learning-envelope.invalid.json"))));assert e and any("visibility" in list(x.absolute_path) for x in e)'
echo "verify OK: change-tlm-002-team-aware-learning-design"
