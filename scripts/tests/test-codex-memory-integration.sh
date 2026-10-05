#!/usr/bin/env bash
# Production installer/helper/compiled doctor; real Codex startup is a separate
# prerequisite-bound subcase, never replaced with a synthetic summary writer.
# Exit 0 PASS, 1 FAIL, 2 BLOCKED. No Cargo/build or real configuration discovery.
set -euo pipefail
command -v python3 >/dev/null 2>&1 || { echo 'BLOCKED: Python 3 is required' >&2; exit 2; }
exec python3 -B - "$@" <<'PY'
import argparse, datetime, hashlib, json, os, pathlib, queue, shlex, shutil, signal, sqlite3, stat, subprocess, sys, tempfile, threading, time, urllib.parse
try:
    import tomllib
except ImportError:
    print('BLOCKED: Python 3.11+ stdlib tomllib is required'); sys.exit(2)
p = argparse.ArgumentParser(description='Scratch installer memory policy and verified installed doctor remediation')
for name in ('full','install-source','evidence','scratch','doctor-bin'): p.add_argument('--' + name, required=True)
p.add_argument('--codex-bin', help='Explicit Codex 0.158 binary; never discovered from an operator home')
p.add_argument('--codex-provider-config', help='Explicit auth-free loopback Responses provider TOML; no user state is imported')
p.add_argument('--native-timeout-seconds', type=int, default=300)
a = p.parse_args()
records, cases, owned, reason, code, identities = [], [], None, None, 1, {}
evidence = pathlib.Path(a.evidence)
started = datetime.datetime.now(datetime.timezone.utc).isoformat()
class Blocked(Exception): pass
def interrupted(signum, frame): raise KeyboardInterrupt('signal ' + str(signum))
for sig in (signal.SIGINT, signal.SIGTERM): signal.signal(sig, interrupted)
def require(value, message):
    if not value: raise AssertionError(message)
def absolute(value):
    path = pathlib.Path(value)
    if not path.is_absolute(): raise Blocked('absolute input required: ' + value)
    try: return path.resolve(strict=True)
    except OSError: raise Blocked('unavailable explicit input: ' + value)
def snap(root):
    result = {}
    for path in sorted(root.rglob('*')):
        info = path.lstat()
        data = os.readlink(path).encode() if path.is_symlink() else path.read_bytes() if path.is_file() else b''
        result[str(path.relative_to(root))] = [stat.S_IMODE(info.st_mode), hashlib.sha256(data).hexdigest()]
    return result
def call(argv, env, label, expected=0, cwd=None, timeout=180):
    start = datetime.datetime.now(datetime.timezone.utc).isoformat()
    process = subprocess.Popen([str(x) for x in argv], env=env, cwd=cwd or owned, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    timed_out = False
    try: stdout_text, stderr_text = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(process.pid, signal.SIGKILL)
        stdout_text, stderr_text = process.communicate()
    finally:
        try: os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError: pass
        if process.poll() is None: process.wait()
    run = subprocess.CompletedProcess(argv,process.returncode,stdout_text,stderr_text)
    index = len(records); out, err = evidence / ('memory-%03d.stdout' % index), evidence / ('memory-%03d.stderr' % index)
    out.write_text(run.stdout); err.write_text(run.stderr)
    records.append(dict(label=label, argv=[str(x) for x in argv], cwd=str(cwd or owned), exitCode=run.returncode,
                        start=start, end=datetime.datetime.now(datetime.timezone.utc).isoformat(), stdout=str(out), stderr=str(err)))
    require(not timed_out, label + ': process deadline exceeded')
    if expected is not None: require(run.returncode == expected, label + ': unexpected exit ' + str(run.returncode))
    return run
def check_result(env, label):
    result = call([doctor,'doctor','--check','codex.memories','--json'], env, label, expected=None)
    report = json.loads(result.stdout)
    require((result.returncode==0)==(report['summary']['exit_code']==0), 'doctor exit disagrees with public report')
    checks = [row for row in report['checks'] if row['id'] == 'codex.memories']
    require(len(checks) == 1, 'doctor omitted its public check ID')
    return checks[0]

class NativeSession:
    # Actual Codex stdio JSON-RPC transport. File fixtures only supply historical
    # inputs; the application creates/migrates its databases and runs the worker.
    def __init__(self, binary, env, cwd, label):
        self.label, self.messages, self.next_id = label, queue.Queue(), 0
        self.start = datetime.datetime.now(datetime.timezone.utc).isoformat()
        self.out_path, self.err_path = evidence/(label+'.jsonl'), evidence/(label+'.stderr')
        self.out = self.out_path.open('w'); self.err = self.err_path.open('w')
        self.process = subprocess.Popen([str(binary),'app-server','--listen','stdio://'], env=env, cwd=cwd,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.err, text=True, start_new_session=True)
        def read():
            try:
                for line in self.process.stdout:
                    self.out.write(line); self.out.flush()
                    try: self.messages.put(json.loads(line))
                    except ValueError: self.messages.put({'transport_error':'non-JSON native stdout'})
            finally: self.messages.put({'transport_error':'native transport closed'})
        self.reader = threading.Thread(target=read, daemon=True); self.reader.start()
    def send(self, message):
        self.process.stdin.write(json.dumps(message)+'\n'); self.process.stdin.flush()
    def receive(self, predicate, timeout):
        deadline = time.monotonic()+timeout
        while time.monotonic()<deadline:
            try: message = self.messages.get(timeout=max(0.001,min(1,deadline-time.monotonic())))
            except queue.Empty: continue
            require('transport_error' not in message, str(message))
            # Unexpected approval/tool callbacks cannot be answered with fixtures.
            require(not ('method' in message and 'id' in message), 'unexpected native server request: '+str(message.get('method')))
            if predicate(message): return message
        raise AssertionError(self.label+': native response deadline exceeded')
    def rpc(self, method, params):
        self.next_id += 1; request_id = self.next_id
        self.send(dict(id=request_id,method=method,params=params))
        reply = self.receive(lambda row:row.get('id')==request_id, 60)
        require('error' not in reply, method+': '+str(reply.get('error')))
        return reply['result']
    def initialize(self):
        self.rpc('initialize',dict(clientInfo=dict(name='prometheus-memory-integration',version='1'),
            capabilities=dict(experimentalApi=True)))
        self.send(dict(method='initialized',params={}))
    def thread(self, cwd):
        return self.rpc('thread/start',dict(cwd=str(cwd),approvalPolicy='never',sandbox='workspace-write',ephemeral=False))['thread']['id']
    def infer(self, thread_id):
        turn = self.rpc('turn/start',dict(threadId=thread_id,input=[dict(type='text',text='Reply exactly: PROMETHEUS_NATIVE_PROVIDER_OK. Do not use tools.',textElements=[])]))
        turn_id = turn['turn']['id']
        event = self.receive(lambda row:row.get('method')=='turn/completed' and row.get('params',{}).get('turn',{}).get('id')==turn_id, a.native_timeout_seconds)
        outcome = event['params']['turn']
        if outcome.get('status') != 'completed':
            raise Blocked('real isolated provider did not complete inference: '+str(outcome.get('error')))
        messages=[json.loads(line) for line in self.out_path.read_text().splitlines()]
        require(any(row.get('method')=='item/completed' and row.get('params',{}).get('item',{}).get('type')=='agentMessage'
            and 'PROMETHEUS_NATIVE_PROVIDER_OK' in row['params']['item'].get('text','') for row in messages),
            'real provider did not produce an assistant inference sentinel')
    def close(self):
        try:
            if self.process.poll() is None: os.killpg(self.process.pid, signal.SIGTERM)
            try: self.process.wait(timeout=10)
            except subprocess.TimeoutExpired: os.killpg(self.process.pid,signal.SIGKILL); self.process.wait()
        finally:
            try: os.killpg(self.process.pid,signal.SIGKILL)
            except ProcessLookupError: pass
            self.reader.join(timeout=5); self.out.close(); self.err.close()
            records.append(dict(label=self.label,argv=[str(self.process.args[0]),'app-server','--listen','stdio://'],
                start=self.start,end=datetime.datetime.now(datetime.timezone.utc).isoformat(),exitCode=self.process.returncode,
                stdout=str(self.out_path),stderr=str(self.err_path)))

def native_startup(binary, provider_path, helper, base_env):
    provider_text = provider_path.read_text(); provider = tomllib.loads(provider_text)
    identities['nativeFixtureContract']=dict(engine='0.158',officialSourceCommit='064c6b8c737f5b41d171fdda80bd9ef10ad06eb3',
        stateDatabase='state_5.sqlite',memoryDatabases=['memories_1.sqlite','memories_v2_1.sqlite'],
        jobKind='memory_consolidate_global',jobKey='global')
    for name,path in (('nativeCodex',binary),('nativeProviderConfig',provider_path)):
        identities[name]=dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    require(set(provider)=={'model','model_provider','model_providers'}, 'provider fixture must contain only model/model_provider/model_providers')
    provider_id, model = provider['model_provider'], provider['model']
    require(isinstance(provider_id,str) and isinstance(model,str) and model, 'provider/model must be explicit strings')
    require(set(provider['model_providers'])=={provider_id}, 'exactly one explicit provider required')
    settings = provider['model_providers'][provider_id]
    require(set(settings).issubset({'name','base_url','wire_api','requires_openai_auth','request_max_retries','stream_max_retries','stream_idle_timeout_ms','supports_websockets'}), 'provider fixture contains unsupported/auth-bearing settings')
    url = urllib.parse.urlparse(settings.get('base_url',''))
    require(url.scheme=='http' and url.hostname in ('127.0.0.1','::1') and url.port and url.port!=23001
        and not url.username and not url.password and not url.query and not url.fragment, 'provider must be an explicit auth-free non-memory loopback endpoint')
    require(settings.get('wire_api')=='responses' and settings.get('requires_openai_auth') is False,
        'actual Responses provider must explicitly disable OpenAI auth')
    require(30<=a.native_timeout_seconds<=900, 'native timeout must be between 30 and 900 seconds')
    # Version semantics are pinned; a different engine needs its own real fixture
    # contract rather than quietly applying this schema to an arbitrary runtime.
    version = call([binary,'--version'],base_env,'declared-codex-cli-engine-version').stdout
    if '0.158.' not in version: raise Blocked('native memory fixture requires declared Codex 0.158 engine')
    for memory_version, directory, dbname in (('v1','memories','memories_1.sqlite'),('v2','memories_v2','memories_v2_1.sqlite')):
        root = owned/('native-'+memory_version); root.mkdir()
        native_home, cwd = root/'codex', root/'project'; native_home.mkdir(); cwd.mkdir()
        native_env = dict(base_env,CODEX_HOME=str(native_home))
        native_config = native_home/'config.toml'
        native_config.write_text(provider_text+'\n[features]\nmemories = false\n[memories]\nversion = '+json.dumps(memory_version)
            +'\ndual_write = false\ngenerate_memories = false\nuse_memories = false\nextraction_model = '+json.dumps(model)
            +'\nconsolidation_model = '+json.dumps(model)+'\n')
        session = NativeSession(binary,native_env,cwd,'native-'+memory_version+'-bootstrap')
        try:
            session.initialize(); historical_thread = session.thread(cwd); session.infer(historical_thread)
        finally: session.close()
        state_db, memory_db = native_home/'state_5.sqlite', native_home/dbname
        require(state_db.is_file() and memory_db.is_file(), 'native application did not create pinned state/memory databases')
        now = int(time.time())
        with sqlite3.connect(state_db) as db:
            require(db.execute('SELECT count(*) FROM threads WHERE id=?',(historical_thread,)).fetchone()[0]==1, 'native persisted thread missing')
            db.execute("UPDATE threads SET memory_mode='enabled' WHERE id=?",(historical_thread,))
        # A durable historical output, deliberately eligible despite generation
        # and retrieval being disabled. No fake provider or summary writer exists.
        with sqlite3.connect(memory_db) as db:
            db.execute('INSERT INTO stage1_outputs(thread_id,source_updated_at,raw_memory,rollout_summary,rollout_slug,generated_at,last_usage) VALUES(?,?,?,?,?,?,?)',
                (historical_thread,now,'For this isolated project, persist decisions in project notes and inspect actual source before claiming a release. The user prefers explicit source versus installed runtime evidence.',
                 '# Completed project source inspection\nThe project owner requested durable source/evidence boundaries. Future work should preserve production files and report real integration outcomes.',
                 'scratch-source-inspection',now,now))
            db.execute("INSERT INTO jobs(kind,job_key,status,retry_remaining,input_watermark,last_success_watermark) VALUES('memory_consolidate_global','global','pending',3,?,0)",(now,))
        def job():
            with sqlite3.connect('file:'+str(memory_db)+'?mode=ro',uri=True) as db:
                db.row_factory=sqlite3.Row
                return dict(db.execute("SELECT * FROM jobs WHERE kind='memory_consolidate_global' AND job_key='global'").fetchone())
        text = native_config.read_text().replace('memories = false','memories = true',1); native_config.write_text(text)
        enabled_start = time.monotonic(); observed = []; claim_latency = None
        session = NativeSession(binary,native_env,cwd,'native-'+memory_version+'-enabled-generation-use-disabled')
        try:
            session.initialize(); session.thread(cwd)
            deadline = time.monotonic()+a.native_timeout_seconds
            while time.monotonic()<deadline:
                current=job()
                if not observed or current!=observed[-1]: observed.append(current)
                if claim_latency is None and current['status']!='pending': claim_latency=time.monotonic()-enabled_start
                if current['status']=='done': break
                if current['status']=='error': raise Blocked('actual native enabled control could not consolidate: '+str(current.get('last_error')))
                require(session.process.poll() is None, 'native application exited before consolidation')
                time.sleep(0.2)
            if job()['status']!='done': raise Blocked('actual enabled consolidation control did not reach durable success before declared deadline; status='+str(job()['status']))
            summary=native_home/directory/'memory_summary.md'
            # 0.158 uses the literal format header v1 in BOTH version roots;
            # the native successful job also validates v2's richer structure.
            require(summary.is_file() and summary.read_text().splitlines()[:1]==['v1'], 'real consolidation did not create a valid-format summary')
            require(job()['last_success_watermark']>=now, 'native job did not consume historical eligible input')
            shutil.copy2(summary,evidence/('native-'+memory_version+'-enabled-summary.md'))
        finally:
            session.close()
            enabled_duration=time.monotonic()-enabled_start
            (evidence/('native-'+memory_version+'-enabled-jobs.json')).write_text(json.dumps(dict(jobs=observed,
                claimLatencySeconds=claim_latency,completionSeconds=enabled_duration),indent=2))
        # Make the same retained historical data pending again before applying
        # policy. Restart twice: existing app-server configuration is not reloaded.
        with sqlite3.connect(memory_db) as db:
            db.execute("UPDATE jobs SET status='pending',worker_id=NULL,ownership_token=NULL,started_at=NULL,finished_at=NULL,lease_until=NULL,retry_at=NULL,last_error=NULL,input_watermark=input_watermark+1 WHERE kind='memory_consolidate_global' AND job_key='global'")
            db.execute('UPDATE stage1_outputs SET source_updated_at=source_updated_at+1,last_usage=?',(int(time.time()),))
        call([tools/'bash',helper],native_env,'native-'+memory_version+'-apply-policy')
        require(not summary.exists(), 'helper failed to archive enabled-control summary')
        baseline=job()
        with sqlite3.connect(memory_db) as db: retained_rows=db.execute('SELECT * FROM stage1_outputs ORDER BY thread_id').fetchall()
        for restart in range(2):
            session=NativeSession(binary,native_env,cwd,'native-'+memory_version+'-disabled-restart-'+str(restart))
            try:
                session.initialize(); thread_id=session.thread(cwd); session.infer(thread_id)
                observation_seconds=max(15,claim_latency+2)
                deadline=time.monotonic()+observation_seconds
                while time.monotonic()<deadline:
                    require(job()==baseline, 'disabled startup claimed/changed global consolidation job')
                    require(not summary.exists(), 'disabled native startup regenerated summary')
                    time.sleep(0.2)
            finally: session.close()
            with sqlite3.connect(memory_db) as db:
                require(db.execute('SELECT * FROM stage1_outputs ORDER BY thread_id').fetchall()==retained_rows, 'disabled startup changed retained historical outputs')
        cases.append(dict(acceptance=['02/AC-2','02/AC-3'],status='PASS',memoryVersion=memory_version,
            note='real auth-free provider inference; actual native enabled consolidation consumed eligible historical output; helper application plus two fresh disabled sessions left the same pending job and retained data unchanged',
            enabledJobs=str(evidence/('native-'+memory_version+'-enabled-jobs.json')),observationSeconds=observation_seconds,
            desktopEngine='unestablished; no desktop session or real settings inspected'))
try:
    full, source, scratch, doctor = map(absolute, (a.full,a.install_source,a.scratch,a.doctor_bin))
    for name,path in (('compiledDoctor',doctor),('installer',source/'scripts/install-system.js'),('scenario',full/'scripts/tests/test-codex-memory-integration.sh')):
        identities[name]=dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    if not source.is_relative_to(scratch) or source == scratch: raise Blocked('install-source must be a prepared scratch-contained candidate')
    if scratch == pathlib.Path.home().resolve(): raise Blocked('real HOME is not scratch')
    # Never let installer submodule initialization follow a real checkout pointer.
    for git in [source / '.git'] + list(source.rglob('.git')):
        if git.is_symlink(): raise Blocked('source contains symlinked Git metadata: ' + str(git))
        if git.is_file():
            line = git.read_text().strip()
            if not line.startswith('gitdir: '): raise Blocked('unrecognized Git pointer')
            target = (git.parent / line[8:]).resolve()
            if not target.is_relative_to(scratch): raise Blocked('Git pointer escapes scratch')
    if not evidence.is_absolute(): raise Blocked('evidence must be absolute')
    evidence.mkdir(parents=True, exist_ok=True)
    owned = pathlib.Path(tempfile.mkdtemp(prefix='codex-memory-', dir=scratch)); (owned / '.owned-integration').write_text('memory\n')
    tools = owned / 'tools'; tools.mkdir()
    for name in ('node','bash','python3','git','ssh-keygen','basename','awk','cat','head','mktemp','dirname','seq','sleep','rm','mkdir','shasum','sha256sum','uname','tr','cut','date','find','sed','wc','sort','ls','cp','mv','touch','pwd','printf','grep'):
        tool = shutil.which(name)
        if tool: (tools / name).symlink_to(pathlib.Path(tool).resolve())
    for name in ('node','bash','python3','git','ssh-keygen','basename'):
        if not (tools / name).exists(): raise Blocked('missing tool: ' + name)
    home, codex = owned / 'home', owned / "selected Codex's home"
    for path in (home, codex, owned / 'tmp', owned / 'unrelated-cwd'): path.mkdir()
    env = dict(PATH=str(tools), HOME=str(home), CODEX_HOME=str(codex),
               XDG_CONFIG_HOME=str(owned/'config'), XDG_DATA_HOME=str(owned/'data'), PROMETHEUS_DATA_DIR=str(owned/'data'),
               CLAUDE_CONFIG_DIR=str(owned/'claude'), CORTEX_DATA_DIR=str(owned/'cortex'), TMPDIR=str(owned/'tmp'),
               PROMETHEUS_LEARNING_QUEUE=str(owned/'queue'), PROMETHEUS_LEARNING_LOG_DIR=str(owned/'log'),
               PROMETHEUS_LEARNING_INDEX_DIR=str(owned/'index'), PROMETHEUS_TEAM_DIGEST_DIR=str(owned/'digest'),
               PROMETHEUS_LEARNING_CORTEX='0', GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=os.devnull,
               GIT_TERMINAL_PROMPT='0', LANG='C', LC_ALL='C')
    call([doctor,'--version'],env,'compiled-cli-version')
    call([tools/'node','--version'],env,'node-version')
    call([tools/'bash','--version'],env,'bash-version')
    call([tools/'python3','--version'],env,'python-version')
    original = '''# retained comment\nmodel = "scratch-fixture"\ninstructions = """\n[features]\nmemories = true\n[memories]\nuse_memories = true\n"""\n[features] # retained header\nmemories = true # retained setting comment\nunrelated = true\n[memories]\ngenerate_memories = true\nuse_memories = true\n[profiles.keep]\nmodel = "untouched"\n'''
    config = codex / 'config.toml'; config.write_text(original)
    retained = {}
    for version, directory in (('v1','memories'),('v2','memories_v2')):
        root = codex / directory; root.mkdir()
        (root / 'memory_summary.md').write_text('summary-' + version)
        for name in ('MEMORY.md','raw_memories.md','state.sqlite'):
            (root/name).write_text(directory + '/' + name); retained[str(root/name)] = (root/name).read_bytes()
    extensions = codex / 'memories_extensions'; extensions.mkdir(); (extensions/'memory_summary.md').write_text('extension-preserved')
    retained[str(extensions/'memory_summary.md')] = (extensions/'memory_summary.md').read_bytes()
    helper = source / 'shared/scripts/codex-memories-config.sh'
    before = snap(codex)
    unhealthy = json.loads(call([tools/'bash',helper,'--check'],env,'helper-read-only-unhealthy').stdout)
    require(not unhealthy['ok'] and unhealthy['summary_v1_present'] and unhealthy['summary_v2_present'], 'helper check did not expose both summary versions')
    require(before == snap(codex), '--check changed configuration/artifacts')
    install = [tools/'node',source/'scripts/install-system.js','--source-root',source,'--home',home,'--profile','skills','--targets','codex','--non-interactive','--yes']
    call(install,env,'actual-full-pack-installer-memory-dispatch',cwd=source)
    parsed = tomllib.loads(config.read_text())
    expected = tomllib.loads(original)
    for section, key in (('features','memories'),('memories','generate_memories'),('memories','use_memories')):
        require(parsed[section][key] is False, 'missing disabled setting ' + section + '.' + key)
        expected[section][key] = False
    require(parsed == expected, 'installer changed unrelated TOML data')
    require('# retained comment' in config.read_text() and '# retained setting comment' in config.read_text(), 'installer lost comments')
    backups = list(codex.glob('config.toml.bak-*'))
    require(backups and any(x.read_text() == original for x in backups), 'original config backup missing')
    for version in ('v1','v2'):
        archived = list((codex/'memories-archive').glob('memory_summary-' + version + '-*.md'))
        require(archived and any(x.read_text() == 'summary-' + version for x in archived), 'version-distinguishable archive missing')
    for path, data in retained.items(): require(pathlib.Path(path).read_bytes() == data, 'unrelated memory artifact changed')
    require(check_result(env,'compiled-doctor-healthy')['status'] == 'pass', 'compiled doctor disagrees with applied policy')
    policy_snapshot = snap(codex)
    call(install,env,'repeat-actual-install-idempotence',cwd=source)
    require(snap(codex) == policy_snapshot, 'repeat installation changed selected Codex contents')
    # Put summaries back with different bytes; same-second archives must retain
    # both versions and both rounds, regardless of timestamp collisions.
    for version,directory in (('v1','memories'),('v2','memories_v2')): (codex/directory/'memory_summary.md').write_text('repeat-' + version)
    call([tools/'bash',helper],env,'collision-safe-second-archive')
    for version in ('v1','v2'):
        contents = {x.read_text() for x in (codex/'memories-archive').glob('memory_summary-' + version + '-*.md')}
        require({'summary-' + version, 'repeat-' + version}.issubset(contents), 'archive overwrite or version collision')
    for index, text in enumerate(('# missing tables\nmodel = "keep"\n',
            '"features"."memories" = true # dotted\n"memories"."generate_memories" = true\n"memories"."use_memories" = true\n',
            "['features']\n'memories' = false\n['memories']\n'generate_memories' = false\n'use_memories' = false\n")):
        config.write_text(text)
        call([tools/'bash',helper],env,'supported-toml-' + str(index))
        require(json.loads(call([tools/'bash',helper,'--check'],env,'supported-check-' + str(index)).stdout)['ok'], 'supported TOML unhealthy')
    for index, text in enumerate(('features = { memories = true }\n', '[features\nmemories = true\n', 'features = "wrong table"\n')):
        config.write_text(text); before = snap(codex)
        require(not json.loads(call([tools/'bash',helper,'--check'],env,'unsupported-read-only-' + str(index)).stdout)['ok'], 'invalid/unsupported config falsely healthy')
        require(before == snap(codex), 'unhealthy read-only check mutated files')
        failed = call([tools/'bash',helper],env,'unsupported-apply-' + str(index),expected=None)
        require(failed.returncode != 0 and before == snap(codex), 'unsupported apply changed original or succeeded')
        require(check_result(env,'unsupported-doctor-' + str(index))['status'] != 'pass', 'doctor accepted invalid setting/table')
    config.write_text(original)
    store = home / '.prometheus/plugins/prometheus-skill-pack'
    env['PROMETHEUS_PLUGIN_ROOT'] = str(store)
    diagnosed = check_result(env,'installed-doctor-from-unrelated-cwd')
    action = next(row for row in diagnosed['actions'] if row['id'] == 'codex.disable-memories')
    hint = action['command_hint']; require(hint, 'verified installed helper remediation missing')
    words = shlex.split(hint)
    require(len(words) == 4 and words[0] == 'env' and words[1] == 'CODEX_HOME=' + str(codex) and words[2] == 'bash', 'unsafe/inaccurate repair command')
    installed_helper = pathlib.Path(words[3])
    require(installed_helper.is_absolute() and installed_helper.resolve().is_relative_to(store/'generations'), 'repair does not resolve absolute installed generation helper')
    # Execute exactly the diagnostic command in an unrelated cwd with scratch env.
    call([tools/'bash','-c',hint],env,'execute-actual-quoted-doctor-remediation',cwd=owned/'unrelated-cwd')
    require(check_result(env,'doctor-after-suggested-repair')['status'] == 'pass', 'suggested repair did not fix public doctor predicate')
    config.write_text(original)
    current_pointer = store/'pointers/current'
    for label, path, replacement in (('escaping-pointer',current_pointer,b'generations/../../outside\n'),
            ('corrupt-manifest',installed_helper.parents[2]/'manifest.json',b'{"invalid":true}\n'),
            ('corrupt-helper',installed_helper,b'#!/bin/bash\necho unverified\n'),
            ('missing-cache',home/'.prometheus/capabilities.json',None)):
        saved = path.read_bytes()
        if replacement is None: path.unlink()
        else: path.write_bytes(replacement)
        try:
            result = check_result(env,'doctor-fail-closed-' + label)
            require(result['status'] != 'pass', label + ' became healthy')
            require(all(not x.get('command_hint') for x in result['actions']), label + ' offered executable-looking remediation')
        finally: path.write_bytes(saved)
    cases.append(dict(acceptance=['02/AC-1','06/AC-1','06/AC-2'], status='PASS', note='actual full-pack skills installer, helper and compiled doctor; full service profile is not launched by this scenario'))
    # A new/absent config must pass through the same real installer dispatch.
    codex_new = owned/'new-codex'; fresh = dict(env,CODEX_HOME=str(codex_new))
    call(install,fresh,'installer-absent-selected-codex',cwd=source)
    require(json.loads(call([tools/'bash',helper,'--check'],fresh,'new-config-check').stdout)['ok'], 'installer did not create healthy selected config')
    if not a.codex_bin or not a.codex_provider_config:
        raise Blocked('02/AC-2 and AC-3 require explicit --codex-bin and --codex-provider-config; policy subcases alone do not establish native startup suppression')
    native_startup(absolute(a.codex_bin),absolute(a.codex_provider_config),helper,env)
    code = 0
except Blocked as error: code, reason = 2, str(error)
except BaseException as error: code, reason = 1, type(error).__name__ + ': ' + str(error)
finally:
    if evidence.is_absolute():
        evidence.mkdir(parents=True,exist_ok=True)
        (evidence/'codex-memory-result.json').write_text(json.dumps(dict(status=['PASS','FAIL','BLOCKED'][code],exitCode=code,reason=reason,
            start=started,end=datetime.datetime.now(datetime.timezone.utc).isoformat(),full=a.full,installSource=a.install_source,
            inputIdentities=identities,isolatedRoot=str(owned) if owned else None,cases=cases,commands=records),indent=2))
    if owned is not None and (owned/'.owned-integration').is_file(): shutil.rmtree(owned)
print(['PASS','FAIL','BLOCKED'][code] + ': Codex memory integration' + (': ' + reason if reason else '')); sys.exit(code)
PY
