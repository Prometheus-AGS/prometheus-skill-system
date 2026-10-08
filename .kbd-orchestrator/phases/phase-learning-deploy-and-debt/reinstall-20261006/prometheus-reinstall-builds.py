import subprocess,time,pathlib,os,json
jobs=[('memory','/Users/gqadonis/Projects/prometheus/worktrees/ldd-memory-prior-integration',['cargo','build','--locked','--release','--no-default-features','--features','embedded,metal,local-embeddings,palace','--bin','surreal-memory-server','--jobs','4']),('prometheus','/Users/gqadonis/Projects/prometheus/worktrees/ldd-machine-main/tools/prometheus-cli',['cargo','build','--locked','--release','-p','prometheus-cli','--jobs','4'])]
state=pathlib.Path('/tmp/prometheus-reinstall-build-state.json')
results=[]
for name,cwd,cmd in jobs:
 while True:
  busy={n:subprocess.run(['pgrep','-x',n],capture_output=True,text=True).stdout.split() for n in ('cargo','rustc')}
  if not any(busy.values()):break
  state.write_text(json.dumps({'waiting_for':busy,'next':name,'results':results}));time.sleep(10)
 state.write_text(json.dumps({'building':name,'command':cmd,'results':results}))
 print('Building '+name,flush=True)
 env=os.environ.copy();env['RUSTC_WRAPPER']='/opt/homebrew/bin/sccache'
 with open('/tmp/prometheus-'+name+'-build.log','w') as log:
  r=subprocess.run(cmd,cwd=cwd,env=env,stdout=log,stderr=subprocess.STDOUT)
 results.append({'name':name,'exit_code':r.returncode,'command':cmd})
 state.write_text(json.dumps({'results':results}))
 print(name+' exit '+str(r.returncode),flush=True)
 if r.returncode:raise SystemExit(r.returncode)
