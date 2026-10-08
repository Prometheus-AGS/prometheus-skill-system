import json,subprocess,pathlib
rows=json.load(open('/tmp/prometheus-codex-before.json'))['installed'];results=[]
for r in rows:
 if r.get('marketplaceName')!='prometheus-skill-pack' or not r.get('installed'):continue
 ident=r['pluginId']
 if ident in ('prometheus-skill-pack@prometheus-skill-pack','prometheus-process-skills@prometheus-skill-pack'):continue
 if not r.get('enabled'):raise RuntimeError('preserve disabled plugin '+ident)
 print('Refreshing '+ident,flush=True)
 for cmd in [['codex','plugin','remove',ident,'--json'],['codex','plugin','add',ident,'--json']]:
  p=subprocess.run(cmd,capture_output=True,text=True)
  results.append({'command':cmd,'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
  pathlib.Path('/tmp/prometheus-codex-category-refresh.json').write_text(json.dumps(results,indent=2))
  if p.returncode:raise SystemExit(p.returncode)
print('Refreshed remaining installed Codex categories',flush=True)
