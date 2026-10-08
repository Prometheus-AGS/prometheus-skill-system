# Verification — change-tlm-001-subagent-identity-probe

Repository: `prometheus-skill-pack`
Depends on: none

## Acceptance criteria

- The evidence file records a verdict (CONFIRMED / REFUTED / UNVERIFIABLE) for each of behaviours 1–6 in both harnesses where applicable, with captured stdin JSON excerpts.
- For each REFUTED or UNVERIFIABLE behaviour, a named fallback is recorded.
- Each harness run is bracketed by hash sets recorded immediately before and after it. These cover the config files, `~/.claude.json`, the learning-queue and outbox listings, and the Codex memory files. The before and after sets must match apart from declared residuals. Any changes the executing session makes between runs are not attributed to the probe.
- At verify time, the strict files still match the probe's **most recent** post-run hashes. Other apps (the ChatGPT desktop app) rewrite `~/.codex/config.toml` between runs, so the first-run hashes are not a valid baseline.
- The probe script passes `bash -n` and `/bin/bash -n`.

## Verify commands

```verify
bash -n shared/scripts/tests/probe-subagent-identity.sh && /bin/bash -n shared/scripts/tests/probe-subagent-identity.sh
python3 - <<'PY'
import re,json,hashlib,os
t=open('.kbd-orchestrator/phases/team-aware-learning-memory/evidence/subagent-identity-probe.md').read()
need={(1,'claude'),(2,'claude'),(3,'codex'),(4,'codex'),(5,'claude'),(5,'codex'),(6,'codex')}
seen={}
for m in re.finditer(r'^\| *([1-6]) *\| *(claude|codex) *\|(.*)$',t,re.M):
    row=m.group(0); verdict=re.search(r'\b(CONFIRMED|REFUTED|UNVERIFIABLE)\b',row)
    assert verdict, f'row without verdict: {row}'
    if verdict.group(1)!='CONFIRMED': assert 'fallback' in row.lower(), f'no fallback: {row}'
    seen[(int(m.group(1)),m.group(2))]=verdict.group(1)
assert need<=set(seen), f'missing rows: {sorted(need-set(seen))}'
runs=re.findall(r'```json (claude|codex)-before\n(.*?)```.*?```json \1-after\n(.*?)```',t,re.S)
assert {r[0] for r in runs}>={'claude','codex'}, 'each harness needs before/after hash blocks recorded around its run'
STRICT=['~/.claude/settings.json','~/.codex/config.toml','~/.codex/hooks.json','~/.prometheus/plugins/prometheus-skill-pack/pointers/current']
res=json.loads(re.search(r'```json residual\n(.*?)```',t,re.S).group(1)) if '```json residual' in t else []
assert all(isinstance(r,dict) and r.get('key') and r.get('reason') for r in res), 'each residual needs key and reason'
residual=[r['key'] for r in res]
assert not set(residual)&set(STRICT), 'strict keys cannot be residual'
for harness,b,a in runs:
    b,a=json.loads(b),json.loads(a)
    assert b and a and all(k in b and k in a for k in STRICT), f'{harness} hash blocks empty or missing strict keys'
    diff=[k for k in b if b[k]!=a.get(k) and k not in residual]
    assert not diff, f'{harness} run changed persistent state: {diff}'
# Live re-check only for files no normal session touches.
def h(p):
    p=os.path.expanduser(p)
    return hashlib.sha256(open(p,'rb').read()).hexdigest() if os.path.exists(p) else 'absent'
# Compare live state with the most recent post-run hashes. Per-run brackets above are the
# proof that the probe wrote nothing; this detects changes made after the last probe run.
# (Other apps, e.g. the ChatGPT desktop app, also rewrite ~/.codex/config.toml.)
last=json.loads(runs[-1][2])
changed=[k for k in STRICT if last[k]!=h(k)]
assert not changed, f'strict config changed since the last probe run: {changed}'
print('ok', seen)
PY
```
