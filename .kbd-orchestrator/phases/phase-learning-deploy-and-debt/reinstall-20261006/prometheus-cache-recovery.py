import pathlib,os,json,hashlib,shutil,sys
H=pathlib.Path.home(); store=H/'.prometheus/plugins/prometheus-skill-pack/generations'; backup=H/'.prometheus/backups/reinstall-20261006/skill-copies'
report=[]
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def inventory(root):
 out={}
 for base,dirs,files in os.walk(root,followlinks=True):
  dirs[:]=[d for d in dirs if d!='__pycache__']
  for f in files:
   p=pathlib.Path(base)/f; rel=p.relative_to(root).as_posix()
   if rel in ('.prometheus-generation','_meta.json') or f.endswith('.pyc'): continue
   out[rel]=(digest(p),bool(p.stat().st_mode&0o111))
 return out
for target in ('.codex/skills','.minimax/skills'):
 for p in (H/target).iterdir():
  marker=p/'.prometheus-generation'
  if p.is_symlink() or not marker.is_file(): continue
  g=marker.read_text().strip(); owner=store/g
  if not (owner/'manifest.json').is_file(): continue
  m=json.loads((owner/'manifest.json').read_text())
  if m.get('sourceVersion')=='1.11.3': continue
  name=p.name
  source=owner/'skills'/name
  if not source.exists() and name.startswith('prometheus-'): source=owner/'skills'/name[len('prometheus-'):]
  if not source.exists(): continue
  a=inventory(p); b=inventory(source)
  delta=[k for k in a.keys()|b.keys() if a.get(k)!=b.get(k)]
  entry={'path':str(p),'generation':g,'matches_owner':not delta,'differences':delta[:8]}
  if not delta and '--apply' in sys.argv:
   dest=backup/p.relative_to(H);dest.parent.mkdir(parents=True,exist_ok=True)
   if dest.exists(): raise RuntimeError('backup exists '+str(dest))
   p.rename(dest);entry['archived']=str(dest)
  report.append(entry)
pathlib.Path('/tmp/prometheus-cache-recovery.json').write_text(json.dumps(report,indent=2))
print(json.dumps({'copies':len(report),'matching':sum(x['matches_owner'] for x in report),'modified':[x for x in report if not x['matches_owner']],'archived':sum('archived'in x for x in report)},indent=2))
