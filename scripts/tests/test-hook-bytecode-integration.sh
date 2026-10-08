#!/usr/bin/env bash
# Local integration only. No build, generation, real-home or service discovery.
# 0 PASS, 1 FAIL, 2 BLOCKED. The coordinator owns the final execution boundary.
set -euo pipefail
command -v python3 >/dev/null 2>&1 || { echo 'BLOCKED: Python 3 is required' >&2; exit 2; }
exec python3 -B - "$@" <<'PY'
import argparse, datetime, hashlib, json, os, pathlib, signal, shutil, socket, stat, subprocess, sys, tempfile

p = argparse.ArgumentParser(description='Actual packaged and direct learning hooks in a scratch signed generation')
for name in ('full', 'evidence', 'scratch', 'hook-runtime-bin'):
    p.add_argument('--' + name, required=True)
p.add_argument('--legacy-payload', help='Read-only known-broken packaged payload for the historical control')
a = p.parse_args()
records, cases, owned, isolated_port, identities = [], [], None, None, {}
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
def snapshot(root):
    result = {}
    for path in sorted(root.rglob('*')):
        info = path.lstat()
        kind = 'link' if stat.S_ISLNK(info.st_mode) else 'file' if stat.S_ISREG(info.st_mode) else 'directory'
        content = os.readlink(path).encode() if kind == 'link' else path.read_bytes() if kind == 'file' else b''
        result[str(path.relative_to(root))] = [kind, stat.S_IMODE(info.st_mode), hashlib.sha256(content).hexdigest()]
    return result
def call(argv, env, label, payload=None, expected=0, cwd=None, timeout=120):
    index = len(records)
    begin = datetime.datetime.now(datetime.timezone.utc).isoformat()
    process = subprocess.Popen([str(x) for x in argv], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, text=True, env=env, cwd=cwd or owned, start_new_session=True)
    timed_out = False
    try:
        stdout_text, stderr_text = process.communicate(payload, timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(process.pid, signal.SIGKILL)
        stdout_text, stderr_text = process.communicate()
    finally:
        try: os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError: pass
        if process.poll() is None: process.wait()
    run = subprocess.CompletedProcess(argv, process.returncode, stdout_text, stderr_text)
    stdout, stderr = evidence / ('hook-%03d.stdout' % index), evidence / ('hook-%03d.stderr' % index)
    stdout.write_text(run.stdout); stderr.write_text(run.stderr)
    records.append(dict(label=label, argv=[str(x) for x in argv], start=begin,
                        end=datetime.datetime.now(datetime.timezone.utc).isoformat(), exitCode=run.returncode,
                        stdout=str(stdout), stderr=str(stderr), cwd=str(cwd or owned)))
    require(not timed_out, label + ': process deadline exceeded')
    if expected is not None: require(run.returncode == expected, label + ': unexpected exit ' + str(run.returncode))
    return run

code, reason = 1, None
try:
    full, scratch, binary = absolute(a.full), absolute(a.scratch), absolute(a.hook_runtime_bin)
    for name,path in (('compiledRuntime',binary),('scenario',full/'scripts/tests/test-hook-bytecode-integration.sh')):
        identities[name]=dict(path=str(path),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    if not scratch.is_dir() or scratch == pathlib.Path.home().resolve(): raise Blocked('scratch must be an explicit isolated directory')
    if not evidence.is_absolute(): raise Blocked('evidence must be absolute')
    evidence.mkdir(parents=True, exist_ok=True)
    owned = pathlib.Path(tempfile.mkdtemp(prefix='hook-bytecode-', dir=scratch))
    (owned / '.owned-integration').write_text('hook-bytecode\n')
    tools = owned / 'tools'; tools.mkdir()
    for name in ('node','bash','python3','git','ssh-keygen','awk','cat','head','mktemp','dirname','seq','sleep','rm','mkdir','shasum','sha256sum','uname','tr','cut','date','find','sed','wc','sort'):
        tool = shutil.which(name)
        if tool: (tools / name).symlink_to(pathlib.Path(tool).resolve())
    for name in ('node','bash','python3','git','ssh-keygen'):
        if not (tools / name).exists(): raise Blocked('missing tool: ' + name)
    home, project = owned / 'home', owned / 'project'
    for folder in (home, project, owned / 'tmp'): folder.mkdir()
    # Own a non-listening port for real file-tier fallback, so even the absence
    # path cannot discover or contact the operator's memory service on :23001.
    isolated_port = socket.socket(); isolated_port.bind(('127.0.0.1', 0))
    require(isolated_port.getsockname()[1] != 23001, 'allocated reserved operator memory port')
    env = dict(PATH=str(tools), HOME=str(home), CODEX_HOME=str(owned / 'codex'),
        XDG_CONFIG_HOME=str(owned / 'config'), XDG_DATA_HOME=str(owned / 'data'), TMPDIR=str(owned / 'tmp'),
        CORTEX_DATA_DIR=str(owned / 'cortex'), PROMETHEUS_DATA_DIR=str(owned / 'data'),
        CLAUDE_CONFIG_DIR=str(owned / 'claude'), PROMETHEUS_LEARNING_QUEUE=str(owned / 'queue'),
        PROMETHEUS_LEARNING_LOG_DIR=str(owned / 'log'), PROMETHEUS_LEARNING_INDEX_DIR=str(owned / 'index'),
        PROMETHEUS_TEAM_DIGEST_DIR=str(owned / 'digest'), PROMETHEUS_LEARNING_CORTEX='0',
        PROMETHEUS_PROJECT_ID='ldd-hook-bytecode', PROMETHEUS_USER_ID='ldd-integration',
        PROMETHEUS_PROJECT_ID_SKIP_RUNTIME='1', GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=os.devnull,
        SURREAL_MEMORY_URL='http://127.0.0.1:' + str(isolated_port.getsockname()[1]), LANG='C', LC_ALL='C')
    def harness_environment(client, cache, store):
        # Each signed store owns its own projected canonical skills. Sharing a
        # target root would correctly make the next store refuse foreign copies.
        context = owned / ('context-' + client); context.mkdir()
        roots = dict(HOME=context/'home', CODEX_HOME=context/'codex',
            CLAUDE_CONFIG_DIR=context/'claude', XDG_CONFIG_HOME=context/'config', XDG_DATA_HOME=context/'data')
        for root in roots.values(): root.mkdir()
        child = dict(env, **{key:str(root) for key,root in roots.items()},
            PROMETHEUS_PLUGIN_ROOT=str(store), CLAUDE_PLUGIN_ROOT=str(cache), PLUGIN_ROOT=str(cache))
        return roots['HOME'], child
    team = project / '.agent-team' / 'integration'; team.mkdir(parents=True)
    (team / 'team.json').write_text(json.dumps(dict(id='integration', roles=[dict(id='implementer', owns=['src/**'])])))
    (project / '.prometheus').mkdir()
    (project / '.prometheus/project.json').write_text(json.dumps(dict(schemaVersion='1', projectId='ldd-hook-bytecode')))
    call([binary, '--version'], env, 'compiled-runtime-version')
    call([tools / 'node', '--version'], env, 'node-version')
    call([tools / 'bash', '--version'], env, 'bash-version')
    call([tools / 'python3', '--version'], env, 'python-version')
    all_trees = []
    for harness, client in (('claude-code','claude'), ('codex','codex')):
        source = full / 'dist/plugins' / client / 'prometheus-skill-pack'
        if not source.is_dir(): raise Blocked('missing generated payload: ' + str(source))
        cache = owned / ('payload-' + client)
        shutil.copytree(source, cache, symlinks=True)
        candidate_relative = pathlib.Path('bin') / (sys.platform + '-' + os.uname().machine) / 'prometheus-hook'
        candidate = cache / candidate_relative
        candidate.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(binary, candidate); candidate.chmod(0o755)
        store = owned / ('store-' + client)
        client_home, child = harness_environment(client, cache, store)
        installer = cache / 'scripts/install-plugin-generation.js'
        command = [tools / 'node', installer, '--source-root', cache, '--home', client_home, '--plugin-root', store, '--targets', 'codex']
        installed = call(command, child, client + '-actual-generation-install')
        generation_id = installed.stdout.strip().splitlines()[-1]
        generation = store / 'generations' / generation_id
        require(generation.is_dir(), 'installer did not return an installed generation')
        bundle = json.loads((generation / 'manifest.json').read_text())['bundleId']
        before = snapshot(generation)
        require(not any('__pycache__' in n or n.endswith('.pyc') for n in before), 'input generation already contains bytecode')
        manifest = json.loads((cache / 'hooks/hooks.json').read_text())['hooks']
        hooks = {}
        for event, groups in manifest.items():
            for group in groups:
                for hook in group.get('hooks', []):
                    words = hook.get('args', [])
                    joined = ' '.join(words) if words else hook.get('command', '')
                    for name in ('sessionstart-learning','subagentstart-learning','subagentstop-learning'):
                        if '--hook ' + name in joined: hooks[name] = hook
        require(len(hooks) == 3, 'packaged hooks must declare all three learning entrypoints')
        for dispatch in ('shell', 'compiled'):
            compiled = store / 'runtime/v1/prometheus-hook'
            if dispatch == 'compiled': shutil.copy2(generation / candidate_relative, compiled); compiled.chmod(0o755)
            elif compiled.exists(): compiled.unlink()
            for value in (None, '0'):
                run_env = dict(child, PROMETHEUS_HARNESS=harness)
                if value is not None: run_env['PYTHONDONTWRITEBYTECODE'] = value
                suffix = client + '-' + dispatch + '-' + (value or 'unset')
                stop = dict(cwd=str(project), session_id=suffix, agent_type='implementer', agent_id=suffix,
                            last_assistant_message='LESSON: integration hook ' + suffix + ' paths: src/example.py',
                            hook_event_name='SubagentStop', turn_id=suffix)
                def declared(name, payload):
                    hook = hooks[name]
                    replace = lambda s: s.replace('${CLAUDE_PLUGIN_ROOT}', str(cache)).replace('${PLUGIN_ROOT}', str(cache))
                    argv = [hook['command']] + [replace(x) for x in hook['args']] if isinstance(hook.get('args'), list) else [tools / 'bash', '-c', replace(hook['command'])]
                    return call(argv, run_env, suffix + '-' + name, json.dumps(payload), cwd=project)
                declared('subagentstop-learning', stop)
                log = owned / 'log/lessons.jsonl'
                require(log.is_file() and ('integration hook ' + suffix) in log.read_text(), 'SubagentStop did not persist its real lesson')
                main = dict(cwd=str(project), session_id=suffix, hook_event_name='SessionStart', turn_id=suffix)
                require('recorded a lesson' in declared('sessionstart-learning', main).stdout, 'SessionStart did not deliver persisted digest')
                start_payload = dict(stop, hook_event_name='SubagentStart')
                start_payload.pop('last_assistant_message')
                delivered = declared('subagentstart-learning', start_payload)
                require('additionalContext' in delivered.stdout, 'SubagentStart did not deliver persisted role content')
        for value in (None, '0'):
            direct_env = dict(child, PROMETHEUS_HARNESS=harness)
            if value is not None: direct_env['PYTHONDONTWRITEBYTECODE'] = value
            payload = dict(cwd=str(project), session_id=client + '-direct-' + (value or 'unset'), agent_type='implementer',
                           last_assistant_message='LESSON: direct wrapper ' + client + ' paths: src/direct.py')
            for name in ('subagentstop-learning', 'subagentstart-learning'):
                result = call([tools / 'bash', generation / ('shared/scripts/' + name + '.sh'), harness], direct_env,
                              client + '-direct-' + name, json.dumps(payload), cwd=project)
                if name == 'subagentstart-learning': require('additionalContext' in result.stdout, 'direct role recall was silent')
            main = dict(cwd=str(project), session_id='direct-main', turn_id='direct-main')
            require(call([tools / 'bash', generation / 'shared/scripts/sessionstart-learning.sh', harness], direct_env,
                         client + '-direct-sessionstart', json.dumps(main), cwd=project).stdout, 'direct main recall was silent')
            result = call([tools / 'python3', generation / 'shared/scripts/lib/learning_write.py', '--text', 'direct writer ' + client + (value or 'unset'),
                           '--cwd', project, '--payload-stdin', '--visibility', 'agent'], direct_env, client + '-direct-writer', json.dumps(payload), cwd=project)
            require(json.loads(result.stdout)['operations'], 'writer did not enqueue real memory operations')
        require(list((owned / 'queue/memory/pending').glob('*.json')), 'real durable enqueue child was not exercised')
        after = snapshot(generation)
        require(before == after, 'immutable generation bytes/modes changed')
        call(command + ['--verify'], child, client + '-actual-verification-after-hooks')
        coverage = json.loads(call(command + ['--reviewed-coverage'], child,
            client + '-actual-reviewed-coverage-after-hooks').stdout)
        require(coverage['generation'] == generation_id and coverage['generationDigest'] == 'sha256:' + generation_id,
            'reviewed coverage is not bound to the installed generation')
        require(coverage['manifestDigest'] == 'sha256:' + hashlib.sha256((generation / 'manifest.json').read_bytes()).hexdigest(),
            'reviewed coverage is not bound to the signed manifest')
        require(coverage['inventory'] == json.loads((generation / 'reviewed-skill-closures.json').read_text()),
            'reviewed coverage differs from the installed closure inventory')
        require(coverage['targetReceipts'], 'reviewed coverage omitted verified target receipts')
        all_trees.append(dict(client=client, generation=str(generation), before=before, after=after))
        # A real unprotected local import demonstrates that the immutable verifier
        # rejects cache contamination. This is separate from the historical hook control.
        contamination = call([tools / 'python3', '-c', 'import os,sys; os.environ.pop("PYTHONDONTWRITEBYTECODE",None); sys.dont_write_bytecode=False; sys.path.insert(0,sys.argv[1]); import agent_identity', generation / 'shared/scripts/lib'],
                             child, client + '-actual-bytecode-contamination')
        require(any('__pycache__' in n for n in snapshot(generation)), 'negative import did not create bytecode')
        rejected = call(command + ['--verify'], child, client + '-contamination-rejected', expected=None)
        require(rejected.returncode != 0, 'strict generation verification accepted bytecode contamination')
    (evidence / 'hook-generation-snapshots.json').write_text(json.dumps(all_trees, indent=2))
    cases.append(dict(acceptance=['01/AC-1','01/AC-2'], status='PASS', note='packaged shell/compiled dispatch and direct wrapper/writer/enqueue boundaries; optional real Cortex feeder is a coordinator-owned scenario'))
    if not a.legacy_payload:
        raise Blocked('01/AC-3 historical packaged-hook payload is unavailable; contamination rejection passed separately')
    legacy = absolute(a.legacy_payload)
    cache = owned / 'legacy-payload'; shutil.copytree(legacy, cache, symlinks=True)
    store = owned / 'legacy-store'
    legacy_home, child = harness_environment('legacy', cache, store)
    command = [tools / 'node', cache / 'scripts/install-plugin-generation.js', '--source-root', cache, '--home', legacy_home, '--plugin-root', store, '--targets', 'codex']
    installed = call(command, child, 'historical-generation-install')
    generation = store / 'generations' / installed.stdout.strip().splitlines()[-1]
    bundle = json.loads((generation / 'manifest.json').read_text())['bundleId']
    retained_before = snapshot(generation)
    require(not any('__pycache__' in n or n.endswith('.pyc') for n in retained_before), 'historical input already contains bytecode')
    # Run the unchanged historical dispatcher directly through each current
    # stable runner. Bypass hook-entry so its own protection cannot mask a
    # missing runner-level guarantee, and retain the original runtime for the
    # subsequent unprotected historical control.
    for dispatch in ('shell', 'compiled'):
        for harness in ('claude-code', 'codex'):
            for value in (None, '0'):
                suffix = 'retained-' + dispatch + '-' + harness + '-' + (value or 'unset')
                run_env = dict(child, PROMETHEUS_HARNESS=harness)
                if value is not None: run_env['PYTHONDONTWRITEBYTECODE'] = value
                payload = dict(cwd=str(project), agent_type='implementer', agent_id=suffix,
                    session_id=suffix, turn_id=suffix,
                    last_assistant_message='LESSON: ' + suffix + ' paths: src/retained.py')
                runner = [tools / 'bash', full / 'shared/scripts/hook-runtime-v1.sh'] if dispatch == 'shell' else [binary, 'run']
                for hook in ('subagentstop-learning', 'subagentstart-learning'):
                    result = call(runner + ['--bundle', bundle, '--hook', hook, '--harness', harness],
                        run_env, suffix + '-' + hook, json.dumps(payload), cwd=project)
                    if hook == 'subagentstart-learning':
                        require('additionalContext' in result.stdout, suffix + ': real role recall was silent')
                require(suffix in (owned / 'log/lessons.jsonl').read_text(), suffix + ': real lesson was not persisted')
    require(retained_before == snapshot(generation), 'current stable runner mutated retained historical generation')
    call(command + ['--verify'], child, 'historical-verification-after-current-runners')
    cases.append(dict(acceptance=['issue-160/retained-dispatcher'], status='PASS',
        note='unchanged historical dispatcher and wrappers through current shell/compiled runners, both clients, unset/conflicting bytecode settings, real write/recall'))
    payload = json.dumps(dict(cwd=str(project), agent_type='implementer', session_id='historical', last_assistant_message='LESSON: historical control'))
    call([tools / 'node', cache / 'scripts/hook-entry.mjs', '--bundle', bundle, '--hook', 'subagentstop-learning', '--harness', 'codex'], child, 'historical-actual-hook', payload, cwd=project)
    require(any('__pycache__' in n for n in snapshot(generation)), 'historical hook failed to reproduce bytecode contamination')
    require(call(command + ['--verify'], child, 'historical-verifier-rejection', expected=None).returncode != 0, 'historical contaminated generation verified')
    cases.append(dict(acceptance=['01/AC-3'], status='PASS', note='actual historical packaged hook contaminated its disposable generation; real verifier rejected it'))
    code = 0
except Blocked as error:
    code, reason = 2, str(error)
except BaseException as error:
    code, reason = 1, type(error).__name__ + ': ' + str(error)
finally:
    if evidence.is_absolute():
        evidence.mkdir(parents=True, exist_ok=True)
        (evidence / 'hook-bytecode-result.json').write_text(json.dumps(dict(status=['PASS','FAIL','BLOCKED'][code], exitCode=code,
            reason=reason, start=started, end=datetime.datetime.now(datetime.timezone.utc).isoformat(),
            full=a.full, inputIdentities=identities, isolatedRoot=str(owned) if owned else None, cases=cases, commands=records), indent=2))
    if isolated_port is not None: isolated_port.close()
    if owned is not None and (owned / '.owned-integration').is_file(): shutil.rmtree(owned)
print(['PASS','FAIL','BLOCKED'][code] + ': hook bytecode integration' + (': ' + reason if reason else ''))
sys.exit(code)
PY
