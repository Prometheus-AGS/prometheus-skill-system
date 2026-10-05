#!/usr/bin/env bash
# Actual install/verify/MCP/rollback/prune/uninstall in an isolated selected root.
# No build, regeneration, background service or real-home mutation. 0/1/2 results.
set -euo pipefail
command -v python3 >/dev/null 2>&1 || { echo 'BLOCKED: Python 3 is required' >&2; exit 2; }
exec python3 -B - "$@" <<'PY'
import argparse, datetime, hashlib, json, os, pathlib, shutil, signal, stat, subprocess, sys, tempfile
try:
    import tomllib
except ImportError:
    print('BLOCKED: Python 3.11+ is required'); sys.exit(2)
p = argparse.ArgumentParser(description='Actual custom CODEX_HOME installation lifecycle')
for name in ('full','install-source','evidence','scratch','doctor-bin'): p.add_argument('--' + name,required=True)
a = p.parse_args(); evidence = pathlib.Path(a.evidence)
records, cases, owned, reason, code, identities = [], [], None, None, 1, {}
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
class Blocked(Exception): pass
def interrupted(signum,frame):raise KeyboardInterrupt('signal '+str(signum))
for sig in (signal.SIGINT,signal.SIGTERM):signal.signal(sig,interrupted)
def require(value,message):
    if not value: raise AssertionError(message)
def absolute(value):
    path = pathlib.Path(value)
    if not path.is_absolute(): raise Blocked('absolute input required: ' + value)
    try: return path.resolve(strict=True)
    except OSError: raise Blocked('unavailable explicit input: '+value)
def snapshot(root):
    result = {}
    for path in sorted(root.rglob('*')):
        info=path.lstat(); data=os.readlink(path).encode() if path.is_symlink() else path.read_bytes() if path.is_file() else b''
        result[str(path.relative_to(root))] = [stat.S_IMODE(info.st_mode),hashlib.sha256(data).hexdigest()]
    return result
def call(argv,env,label,expected=0,cwd=None,timeout=180):
    start=datetime.datetime.now(datetime.timezone.utc).isoformat()
    process=subprocess.Popen([str(x) for x in argv],env=env,cwd=cwd or owned,text=True,
        stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    timed_out=False
    try:stdout_text,stderr_text=process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out=True
        os.killpg(process.pid,signal.SIGKILL)
        stdout_text,stderr_text=process.communicate()
    finally:
        try:os.killpg(process.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        if process.poll() is None:process.wait()
    run=subprocess.CompletedProcess(argv,process.returncode,stdout_text,stderr_text)
    index=len(records); out,err=evidence/('home-%03d.stdout'%index),evidence/('home-%03d.stderr'%index)
    out.write_text(run.stdout);err.write_text(run.stderr)
    records.append(dict(label=label,argv=[str(x) for x in argv],cwd=str(cwd or owned),start=start,
        end=datetime.datetime.now(datetime.timezone.utc).isoformat(),exitCode=run.returncode,stdout=str(out),stderr=str(err)))
    require(not timed_out,label+': process deadline exceeded')
    if expected is not None: require(run.returncode==expected,label+': unexpected exit '+str(run.returncode))
    return run
try:
    full,source,scratch,doctor=map(absolute,(a.full,a.install_source,a.scratch,a.doctor_bin))
    for name,path in (('compiledDoctor',doctor),('installer',source/'scripts/install-system.js'),('scenario',full/'scripts/tests/test-codex-home-integration.sh')):
        identities[name]=dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    if not source.is_relative_to(scratch) or source==scratch: raise Blocked('install-source must be prepared beneath scratch')
    if scratch==pathlib.Path.home().resolve(): raise Blocked('real HOME is not scratch')
    for git in [source/'.git']+list(source.rglob('.git')):
        if git.is_symlink(): raise Blocked('symlinked Git metadata in candidate')
        if git.is_file():
            line=git.read_text().strip()
            if not line.startswith('gitdir: ') or not (git.parent/line[8:]).resolve().is_relative_to(scratch): raise Blocked('Git pointer escapes scratch')
    if not evidence.is_absolute(): raise Blocked('evidence must be absolute')
    evidence.mkdir(parents=True,exist_ok=True)
    owned=pathlib.Path(tempfile.mkdtemp(prefix='codex-home-',dir=scratch));(owned/'.owned-integration').write_text('home\n')
    tools=owned/'tools';tools.mkdir()
    for name in ('node','bash','python3','git','ssh-keygen','basename','awk','cat','head','mktemp','dirname','seq','sleep','rm','mkdir','shasum','sha256sum','uname','tr','cut','date','find','sed','wc','sort','grep'):
        tool=shutil.which(name)
        if tool:(tools/name).symlink_to(pathlib.Path(tool).resolve())
    for name in ('node','bash','python3','git','ssh-keygen','basename'):
        if not (tools/name).exists():raise Blocked('missing tool: '+name)
    home,codex=owned/'selected home',owned/"Codex sibling's root"
    for path in (home,codex,owned/'tmp',owned/'unrelated'):path.mkdir()
    env=dict(PATH=str(tools),HOME=str(home),CODEX_HOME=str(codex),XDG_CONFIG_HOME=str(owned/'config'),XDG_DATA_HOME=str(owned/'data'),
        PROMETHEUS_DATA_DIR=str(owned/'data'),CLAUDE_CONFIG_DIR=str(owned/'claude'),CORTEX_DATA_DIR=str(owned/'cortex'),TMPDIR=str(owned/'tmp'),
        PROMETHEUS_LEARNING_QUEUE=str(owned/'queue'),PROMETHEUS_LEARNING_LOG_DIR=str(owned/'log'),PROMETHEUS_LEARNING_INDEX_DIR=str(owned/'index'),
        PROMETHEUS_TEAM_DIGEST_DIR=str(owned/'digest'),PROMETHEUS_LEARNING_CORTEX='0',GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL=os.devnull,
        GIT_TERMINAL_PROMPT='0',LANG='C',LC_ALL='C')
    call([doctor,'--version'],env,'compiled-cli-version')
    call([tools/'node','--version'],env,'node-version')
    call([tools/'bash','--version'],env,'bash-version')
    call([tools/'python3','--version'],env,'python-version')
    fallback=home/'.codex';fallback.mkdir();(fallback/'config.toml').write_text('# unused fallback\n');(fallback/'sentinel').write_text('retain')
    fallback_before=snapshot(fallback)
    unowned=codex/'skills/operator-owned';unowned.mkdir(parents=True);(unowned/'SKILL.md').write_text('operator data\n')
    unknown=codex/'skills/unknown-owner';unknown.mkdir();(unknown/'SKILL.md').write_text('unknown owner\n');(unknown/'.prometheus-generation').write_text('f'*64+'\n')
    malformed=codex/'skills/malformed-owner';malformed.mkdir();(malformed/'SKILL.md').write_text('malformed owner\n');(malformed/'.prometheus-generation').write_text('not-a-generation\n')
    sentinels={str(path):snapshot(path) for path in (unowned,unknown,malformed)}
    install=[tools/'node',source/'scripts/install-system.js','--source-root',source,'--home',home,'--profile','skills','--targets','codex','--non-interactive','--yes']
    call(install,env,'actual-selected-root-install',cwd=source)
    store=home/'.prometheus/plugins/prometheus-skill-pack';env['PROMETHEUS_PLUGIN_ROOT']=str(store)
    current=(store/'pointers/current').read_text().strip();generation=store/current
    require(current.startswith('generations/'),'missing logical activation pointer')
    first=generation.name
    copies=[path for path in (codex/'skills').iterdir() if (path/'.prometheus-generation').is_file() and (path/'.prometheus-generation').read_text().strip()==first]
    require(copies and all(not path.is_symlink() for path in copies),'selected Codex surface is not a real copied projection')
    receipts=list((store/'receipts'/first).glob('*.json'))
    require(receipts,'logical signed target receipts missing')
    # Actual verification checks the selected copies in addition to store receipts.
    call(install+['--verify'],env,'actual-selected-root-verification',cwd=source)
    helper=json.loads(call([tools/'bash',source/'shared/scripts/codex-memories-config.sh','--check'],env,'selected-memory-helper').stdout)
    require(helper['codex_home']==str(codex) and helper['ok'],'memory helper selected a different root')
    report=json.loads(call([doctor,'doctor','--check','codex.memories','--json'],env,'selected-root-compiled-doctor',cwd=owned/'unrelated').stdout)
    check=next(row for row in report['checks'] if row['id']=='codex.memories')
    require(check['status']=='pass' and any(str(codex) in detail for detail in check['details']),'doctor root/predicate mismatch')
    # MCP config writer uses fixture strings only; it does not contact Tavily,
    # Firecrawl or any service endpoint and receives no inherited credential.
    mcp_env=dict(env,TAVILY_API_KEY='scratch-unused-config-fixture',FIRECRAWL_API_URL='http://127.0.0.1:1')
    call([tools/'bash',source/'scripts/configure-mcp-all-tools.sh','--tool','codex'],mcp_env,'actual-selected-mcp-config')
    require(tomllib.loads((codex/'config.toml').read_text()).get('mcp_servers'),'MCP configuration was not written to selected root')
    require(snapshot(fallback)==fallback_before,'unused fallback changed during install/MCP')
    # Additional generations are legitimate scratch payloads signed by the real
    # installer. Only a skill body varies; runtime/release manifests stay exact.
    payload=owned/'variant-payload';shutil.copytree(full/'dist/plugins/codex/prometheus-skill-pack',payload,symlinks=True)
    skill=next(path for path in (payload/'skills').rglob('SKILL.md') if path.is_file())
    original=skill.read_bytes()
    lifecycle=[tools/'node',payload/'scripts/install-plugin-generation.js','--source-root',payload,'--home',home,'--plugin-root',store,'--targets','codex']
    skill.write_bytes(original+b'\nScratch integration generation B.\n')
    second=call(lifecycle,env,'actual-generation-B').stdout.strip().splitlines()[-1]
    require(second!=first,'second real generation did not change identity')
    restored=call(lifecycle+['--rollback'],env,'actual-custom-root-rollback').stdout.strip().splitlines()[-1]
    require(restored==first,'rollback did not restore intended first generation')
    call(install+['--verify'],env,'actual-verify-after-rollback',cwd=source)
    require(all((path/'.prometheus-generation').read_text().strip()==first for path in copies),'rollback did not update actual custom-root copies')
    skill.write_bytes(original+b'\nScratch integration generation C.\n')
    third=call(lifecycle,env,'actual-generation-C').stdout.strip().splitlines()[-1]
    require(third not in (first,second),'third real generation did not change identity')
    prune=json.loads(call(lifecycle+['--prune-obsolete'],env,'actual-prune-custom-root').stdout)
    require((store/'generations'/third).is_dir() and (store/'generations'/first).is_dir(),'pruning removed active or previous generation')
    call(install+['--verify'],env,'actual-verify-after-prune',cwd=source)
    # A changed selected projection must fail actual verification even while the
    # generation and signed logical receipts remain intact.
    projected=next(path for path in copies if (path/'SKILL.md').is_file());projected_bytes=(projected/'SKILL.md').read_bytes()
    (projected/'SKILL.md').write_bytes(projected_bytes+b'\nchanged selected projection\n')
    require(call(install+['--verify'],env,'changed-projection-rejected',expected=None,cwd=source).returncode!=0,'verification ignored selected projection corruption')
    (projected/'SKILL.md').write_bytes(projected_bytes)
    call(install+['--verify'],env,'restored-projection-verified',cwd=source)
    call(install+['--uninstall'],env,'actual-selected-root-uninstall',cwd=source)
    for path,before in sentinels.items():require(snapshot(pathlib.Path(path))==before,'uninstall changed unowned/unknown/malformed entry')
    require(not any(path.exists() for path in copies),'uninstall retained managed selected-root entries')
    require(snapshot(fallback)==fallback_before,'unused fallback changed during lifecycle')
    # Empty inherited CODEX_HOME means unset. Explicit --home supplies only the
    # fallback; a nonempty sibling CODEX_HOME above already won over --home.
    explicit=owned/'explicit home';explicit.mkdir();empty_env=dict(env,CODEX_HOME='',HOME=str(owned/'unrelated'))
    empty_install=[tools/'node',source/'scripts/install-system.js','--source-root',source,'--home',explicit,'--profile','skills','--targets','codex','--non-interactive','--yes']
    call(empty_install,empty_env,'empty-codex-home-explicit-home-fallback',cwd=source)
    require((explicit/'.codex/config.toml').is_file() and (explicit/'.codex/skills').is_dir(),'empty CODEX_HOME ignored explicit --home fallback')
    require(not (owned/'unrelated/.codex').exists(),'installer projected through ambient HOME instead of --home')
    call(empty_install+['--verify'],empty_env,'empty-home-actual-verify',cwd=source)
    call(empty_install+['--uninstall'],empty_env,'empty-home-actual-uninstall',cwd=source)
    cases.append(dict(acceptance=['07/AC-1','07/AC-2'],status='PASS',generationIds=[first,second,third],
        selectedRoot=str(codex),logicalReceiptRoot=str(store/'receipts'),prune=prune,
        note='actual installer skills profile, MCP writer, compiled doctor and generation lifecycle; no host services started'))
    code=0
except Blocked as error:code,reason=2,str(error)
except BaseException as error:code,reason=1,type(error).__name__+': '+str(error)
finally:
    if evidence.is_absolute():
        evidence.mkdir(parents=True,exist_ok=True)
        (evidence/'codex-home-result.json').write_text(json.dumps(dict(status=['PASS','FAIL','BLOCKED'][code],exitCode=code,reason=reason,
            start=started,end=datetime.datetime.now(datetime.timezone.utc).isoformat(),full=a.full,installSource=a.install_source,
            inputIdentities=identities,isolatedRoot=str(owned) if owned else None,cases=cases,commands=records),indent=2))
    if owned is not None and (owned/'.owned-integration').is_file():shutil.rmtree(owned)
print(['PASS','FAIL','BLOCKED'][code]+': Codex home integration'+(': '+reason if reason else ''));sys.exit(code)
PY
