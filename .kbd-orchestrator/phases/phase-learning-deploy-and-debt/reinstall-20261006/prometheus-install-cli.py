import pathlib,subprocess,json,hashlib,shutil,os
h=pathlib.Path.home();state=json.load(open('/tmp/prometheus-reinstall-build-state.json'))
assert any(r['name']=='prometheus' and r['exit_code']==0 for r in state['results'])
p=h/'.local/bin/prometheus';b=h/'.prometheus/backups/reinstall-20261006/home-local-prometheus';src=pathlib.Path('/Users/gqadonis/Projects/prometheus/worktrees/ldd-machine-main/tools/prometheus-cli/target/release/prometheus')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(p)==sha(b),'installed CLI changed since backup'
tmp=p.with_name('.prometheus.reinstall-20261006');assert not tmp.exists();shutil.copy2(src,tmp);tmp.chmod(0o755)
subprocess.run(['codesign','--force','--sign','-',str(tmp)],check=True,capture_output=True);os.replace(tmp,p)
v=subprocess.run([str(p),'--version'],check=True,capture_output=True,text=True).stdout.strip()
r={'sourceCommit':'3e8a891bcd472e18cc1c097c04b208cb93f7f78f','path':str(p),'version':v,'sha256':sha(p),'backup':str(b)}
pathlib.Path('/tmp/prometheus-cli-install-receipt.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
