import json,pathlib,os,shutil,hashlib,sys
H=pathlib.Path.home();store=H/'.prometheus/plugins/prometheus-skill-pack';backup=H/'.prometheus/backups/reinstall-20261006/project-copies';report=[]
def inventory(root):
 out={}
 for base,dirs,files in os.walk(root,followlinks=True):
  dirs[:]=[d for d in dirs if d!='__pycache__']
  for f in files:
   p=pathlib.Path(base)/f;rel=p.relative_to(root).as_posix()
   if rel in ('.prometheus-generation','_meta.json') or f.endswith('.pyc'):continue
   out[rel]=(hashlib.sha256(p.read_bytes()).hexdigest(),bool(p.stat().st_mode&0o111))
 return out
current=store/'current';m=json.loads((current/'manifest.json').read_text());g=m['generation']
apply='--apply' in sys.argv
if apply and m['sourceProvenance']['sourceCommit']!='3e8a891bcd472e18cc1c097c04b208cb93f7f78f':raise RuntimeError('current generation not refreshed')
for r in json.load(open('/tmp/prometheus-project-cache-inventory.json'))['records']:
 if r['kind']!='copy' or r.get('trackedFileCount',1)>0:continue
 p=pathlib.Path(r['path']);old=store/'generations'/r['markerValue']/'skills'/r['name'];new=current/'skills'/r['name']
 a=inventory(p);b=inventory(old);delta=[k for k in a.keys()|b.keys() if a.get(k)!=b.get(k) and not (k not in a and k.startswith('runtime/'))]
 row={'path':str(p),'oldGeneration':r['markerValue'],'differences':delta[:10],'action':'preserved-modified' if delta else 'eligible'}
 if not delta and apply:
  tmp=p.with_name('.'+p.name+'.reinstall-tmp');dest=backup/p.relative_to(H)
  if not new.is_dir() or tmp.exists() or dest.exists():raise RuntimeError('invalid destination '+str(p))
  shutil.copytree(new,tmp,symlinks=False,ignore=shutil.ignore_patterns('__pycache__','*.pyc'))
  (tmp/'.prometheus-generation').write_text(g+'\n')
  if '.minimax' in p.parts:(tmp/'_meta.json').write_text(json.dumps({'platform':'minimax','name':p.name,'generation':g})+'\n')
  if inventory(tmp)!=inventory(new):raise RuntimeError('copy mismatch '+str(p))
  dest.parent.mkdir(parents=True,exist_ok=True);p.rename(dest)
  try:tmp.rename(p)
  except BaseException:dest.rename(p);raise
  row.update(action='refreshed',generation=g,backup=str(dest))
 report.append(row)
pathlib.Path('/tmp/prometheus-project-cache-refresh.json').write_text(json.dumps(report,indent=2))
print(json.dumps({'candidates':len(report),'counts':{v:sum(x['action']==v for x in report) for v in set(x['action'] for x in report)},'modified':[x for x in report if x['differences']]},indent=2))
