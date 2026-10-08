# Verification — change-tlm-002-team-aware-learning-design

Repository: `prometheus-skill-pack`
Depends on: change-tlm-001-subagent-identity-probe

## Acceptance criteria

- `docs/design/team-aware-learning-memory.md` contains the ten numbered sections. Each section states its decision, rejected alternatives and evidence (citing assessment sections A–N, analysis decisions and probe verdicts).
- The schema validates the valid example and rejects the invalid example.
- The design maps every envelope field onto surreal-memory, pk and the Karpathy records, and names an interim behaviour for each missing upstream capability.
- Budgets match analysis D-4. Delivery uses the change-tlm-001 verdicts (primary or named fallback).

## Verify commands

```verify
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
```
