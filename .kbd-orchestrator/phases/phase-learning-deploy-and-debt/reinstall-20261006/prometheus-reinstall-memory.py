import pathlib,subprocess,os,shutil,json,time,urllib.request,hashlib,re
H=pathlib.Path.home();root=pathlib.Path('/Users/gqadonis/Projects/prometheus/worktrees/ldd-memory-prior-integration');backup=H/'.prometheus/backups/reinstall-20261006'
state=json.load(open('/tmp/prometheus-reinstall-build-state.json'))
assert any(r['name']=='memory' and r['exit_code']==0 for r in state['results']), 'memory build not completed'
server=root/'target/release/surreal-memory-server'
executors=[root/'executors/mlx/.build/release/surreal-memory-mlx-executor',root/'executors/mlx/.build/out/Products/Release/surreal-memory-mlx-executor']
executor=next(p for p in executors if p.is_file())
bundle=root/'executors/mlx/.build/out/Products/Release/mlx-swift_Cmlx.bundle'
assert (bundle/'Contents/Resources/default.metallib').is_file()
assert pathlib.Path('/tmp/prometheus-mlx-build-success').is_file(), 'MLX build not confirmed complete'
def run(args):return subprocess.run(args,check=True,capture_output=True,text=True).stdout
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
plist=H/'Library/LaunchAgents/ai.prometheus.surreal-memory-native.plist';assert sha(plist)==sha(backup/plist.name),'service config changed since backup'
label='gui/'+str(os.getuid())+'/ai.prometheus.surreal-memory-native';domain='gui/'+str(os.getuid())
oldinfo=run(['launchctl','print',label]);oldpid=int(re.search(r'\bpid = (\d+)',oldinfo).group(1))
staged=[]
for dest in [pathlib.Path('/usr/local/bin'),H/'.local/bin']:
 for source in [server,executor]:
  temp=dest/('.'+source.name+'.reinstall-20261006');assert not temp.exists()
  shutil.copy2(source,temp);temp.chmod(0o755);run(['codesign','--force','--sign','-',str(temp)])
  staged.append((temp,dest/source.name))
 temp=dest/'.mlx-swift_Cmlx.bundle.reinstall-20261006';assert not temp.exists()
 run(['ditto',str(bundle),str(temp)]);staged.append((temp,dest/bundle.name))
run(['launchctl','bootout',label])
for _ in range(60):
 try:os.kill(oldpid,0)
 except ProcessLookupError:break
 time.sleep(0.5)
else:raise RuntimeError('old memory process did not exit; no replacement applied')
try:
 for temp,dest in staged:
  if temp.is_dir():
   prior=backup/('activation-'+('usr-local' if str(dest).startswith('/usr/local') else 'home-local')+'-'+dest.name)
   if dest.exists():dest.rename(prior)
   temp.rename(dest)
  else:os.replace(temp,dest)
 run(['launchctl','bootstrap',domain,str(plist)])
 run(['launchctl','enable',label]);run(['launchctl','kickstart',label])
except BaseException:
 subprocess.run(['launchctl','bootstrap',domain,str(plist)],capture_output=True)
 raise
receipt={'sourceCommit':'bfb6d49a7fe25d82ef4ad55f49c8f97e6e82c54a','oldPid':oldpid,'label':label,'plistUnchanged':sha(plist)==sha(backup/plist.name),'version':run(['/usr/local/bin/surreal-memory-server','--version']).strip(),'serverSha256':sha(pathlib.Path('/usr/local/bin/surreal-memory-server')),'localServerSha256':sha(H/'.local/bin/surreal-memory-server'),'executorSha256':sha(pathlib.Path('/usr/local/bin/surreal-memory-mlx-executor')),'localExecutorSha256':sha(H/'.local/bin/surreal-memory-mlx-executor')}
for _ in range(60):
 try:
  with urllib.request.urlopen('http://127.0.0.1:23001/ready',timeout=5) as response:ready=json.load(response)
  if ready.get('status')=='ready' and all(ready.get('capabilities',{}).values()):receipt['ready']=ready;break
 except Exception:pass
 time.sleep(3)
receipt['launchdPid']=int(re.search(r'\bpid = (\d+)',run(['launchctl','print',label])).group(1))
pathlib.Path('/tmp/prometheus-memory-install-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2))
assert receipt.get('ready'), 'replacement running but readiness not confirmed'
assert receipt['launchdPid']!=oldpid
assert receipt['serverSha256']==receipt['localServerSha256']
assert receipt['executorSha256']==receipt['localExecutorSha256']
