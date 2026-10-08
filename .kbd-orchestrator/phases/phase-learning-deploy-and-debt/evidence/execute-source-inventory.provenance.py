#!/usr/bin/env python3
"""Read-only, time-bound source inventory; writes only its adjacent JSON artifact.

Never enters protected deploy-main. No builds, tests, fetch, or source mutation.
"""
import concurrent.futures
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

BASE = Path('/Users/gqadonis/Projects/prometheus')
HERE = Path(__file__).parent
FULL = BASE / 'prometheus-skill-pack'
MINI = BASE / 'prometheus-skills-mini'
CANDIDATE = BASE / 'worktrees/learning-deploy-and-debt'
PROTECTED = str(BASE / 'worktrees/deploy-main')
SELECTED = 'ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61'
START = datetime.datetime.now(datetime.timezone.utc).isoformat()
ENV = dict(os.environ, GIT_OPTIONAL_LOCKS='0')

def call(argv, timeout=30):
    try:
        p = subprocess.run(argv, capture_output=True, env=ENV, timeout=timeout)
        return p.returncode, p.stdout
    except (OSError, subprocess.TimeoutExpired) as e:
        return -1, str(type(e).__name__).encode()

def git(root, *args):
    if str(root) == PROTECTED:
        raise RuntimeError('Protected checkout is metadata-only')
    return call(['git', '-C', str(root), *args])

def out(root, *args):
    return git(root, *args)[1].decode(errors='replace').strip()

def disposition(value, reason):
    return {'disposition': value, 'reason': reason}

def ancestor(root, sha, target):
    return bool(target) and git(root, 'merge-base', '--is-ancestor', sha, target)[0] == 0

def secret(path):
    name = Path(path).name.lower()
    return (name in {'.env', '.mcp.json', 'secrets.env', 'credentials', 'id_rsa', 'id_ed25519'}
            or name.endswith(('.pem', '.p12', '.key'))
            or ('secret' in name and name not in {'secrets.example.env', 'secrets.env.example'}))

def kind(path):
    if secret(path): return 'sensitive-local-configuration'
    if path.startswith(('.kbd-orchestrator/', '.prometheus/', '.claude/worktrees/')):
        return 'bookkeeping-or-learning-data'
    if path.startswith(('dist/', 'out/', 'node_modules/', 'target/', '.agents/skills/', '.codex/skills/')):
        return 'generated-or-installation-output'
    if re.search(r'(^|/)(tests?|__tests__)(/|\.)|(^|/)test[-_]|\.test\.', path):
        return 'test-source'
    return 'production-or-authored-content'

def hash_file(root, path):
    p = root / path
    if secret(path): return {'hash_status': 'not-read-sensitive-path'}
    try:
        st = p.lstat()
        if p.is_symlink():
            data = os.readlink(p).encode()
            mode = 'symlink-target-bytes'
        elif p.is_file():
            data = p.read_bytes()
            mode = 'regular-file-bytes'
        else:
            return {'hash_status': 'directory-or-gitlink-not-file'}
        again = p.lstat()
        return {'sha256': hashlib.sha256(data).hexdigest(), 'byte_length': len(data),
                'hash_kind': mode, 'mode': oct(st.st_mode & 0o777),
                'mtime_ns': st.st_mtime_ns,
                'stable_during_read': (st.st_size, st.st_mtime_ns) == (again.st_size, again.st_mtime_ns)}
    except FileNotFoundError:
        return {'hash_status': 'deleted-or-moved-during-capture'}
    except OSError as e:
        return {'hash_status': type(e).__name__}

def path_disposition(repo, worktree, path, classification):
    if classification == 'bookkeeping-or-learning-data':
        if '/learning-deploy-and-debt' in str(worktree):
            return disposition('include', 'Current phase evidence and canonical artifacts; lead owns ledger changes.')
        return disposition('out-of-scope', 'Preserve historical learning/bookkeeping in the original owner checkout; not a product-source merge.')
    if classification == 'sensitive-local-configuration':
        return disposition('out-of-scope', 'Sensitive/local configuration was not read; preserve in place.')
    if classification == 'generated-or-installation-output':
        return disposition('superseded', 'Reconcile from authored source only at final change-12 generation boundary; preserve original copy.')
    if classification == 'test-source':
        return disposition('out-of-scope', 'Preserve existing test edits; production convergence does not authorize unit/test edits or early execution.')
    if repo == 'uar':
        return disposition('out-of-scope', 'UAR remains a consumer; inspect contract repairs without importing unrelated application source.')
    if repo == 'companion':
        return disposition('include', 'Source-only candidate for change 14; lead must select supported sync/control/workspace source and separate draft features.')
    if repo == 'surreal-memory':
        return disposition('include', 'Reconcile owner source against refreshed remote in isolated ldd-memory-final; no blind checkout copy.')
    if str(worktree) in {str(CANDIDATE), str(BASE / 'worktrees/ldd-mini-final'), str(BASE / 'worktrees/ldd-memory-final')}:
        return disposition('include', 'Current lead-owned isolated production candidate; retain current owner edits.')
    if repo == 'mini':
        return disposition('include', 'Candidate full/mini parity source requires exact hash and prior-owner coordination; do not overwrite source checkout.')
    if repo == 'full':
        return disposition('include', 'Authored source candidate; compare against selected merged baseline before capturing owner edits.')
    return disposition('include', 'Dependency source candidate; resolve relevance and compatibility at source freeze.')

def changed_files(root, repo):
    rc, raw = git(root, 'status', '--porcelain=v1', '-z', '--untracked-files=all')
    if rc: return {'status_capture_error': rc, 'changed_files': []}
    parts = raw.decode(errors='replace').split('\0')
    result = []
    i = 0
    while i < len(parts):
        entry = parts[i]; i += 1
        if not entry: continue
        state, path = entry[:2], entry[3:]
        item = {'path': path, 'status': state, 'classification': kind(path)}
        if 'R' in state or 'C' in state:
            item['old_path'] = parts[i]; i += 1
        item.update(path_disposition(repo, root, path, item['classification']))
        if item['classification'] == 'production-or-authored-content': item.update(hash_file(root, path))
        else: item['hash_status'] = 'nonproduction-source-candidate' if not secret(path) else 'not-read-sensitive-path'
        result.append(item)
    rc2, final = git(root, 'status', '--porcelain=v1', '-z', '--untracked-files=all')
    return {'changed_files': result, 'status_sha256': hashlib.sha256(raw).hexdigest(),
            'status_stable_during_capture': rc2 == 0 and raw == final}

def commits(root, shas, repo, remote):
    result = []
    for sha in shas:
        data = out(root, 'show', '-s', '--format=%H%x00%an%x00%aI%x00%s', sha).split('\0')
        item = {'sha': sha, 'author': data[1] if len(data)>1 else '',
                'authored_at': data[2] if len(data)>2 else '', 'subject': data[3] if len(data)>3 else ''}
        paths = out(root, 'diff-tree', '--no-commit-id', '--name-only', '-r', sha).splitlines()
        item['changed_paths'] = paths
        item['production_paths'] = [p for p in paths if kind(p) == 'production-or-authored-content']
        if ancestor(root, sha, remote):
            item.update(disposition('already-merged', 'Commit is an ancestor of current local origin/main.'))
        elif repo == 'uar':
            item.update(disposition('out-of-scope', 'Consumer implementation; pack uses repaired contract shapes without importing whole UAR branch.'))
        elif not item['production_paths']:
            item.update(disposition('out-of-scope', 'Historical metadata/test-only commit; preserve identity without treating it as unique production work.'))
        elif repo in {'mini', 'surreal-memory', 'companion'}:
            item.update(disposition('include', 'Reconcile exact production source in isolated candidate; source import is not publication or certification.'))
        else:
            item.update(disposition('out-of-scope', 'Historical local-only source identity retained for explicit lineage; not selected without current product-source relevance.'))
        result.append(item)
    return result

def inventory_repo(name, root, remote_inspection=True):
    top = out(root, 'rev-parse', '--show-toplevel')
    if top != str(root): return None
    remote = out(root, 'rev-parse', '--verify', 'refs/remotes/origin/main')
    item = {'name': name, 'path': str(root), 'head': out(root, 'rev-parse', 'HEAD'),
            'branch': out(root, 'branch', '--show-current'), 'origin_main': remote,
            'origin': out(root, 'remote', 'get-url', 'origin'),
            'git_common_dir': out(root, 'rev-parse', '--git-common-dir')}
    blocks = out(root, 'worktree', 'list', '--porcelain').split('\n\n')
    worktrees = []
    for block in blocks:
        data = {}
        for line in block.splitlines():
            k, _, v = line.partition(' '); data[k] = v or True
        if 'worktree' not in data: continue
        w = Path(data['worktree'])
        protected = str(w) == PROTECTED or data.get('branch') == 'refs/heads/deploy/main'
        data['owner'] = ('lead/current phase' if str(w) in {str(CANDIDATE), str(BASE/'worktrees/ldd-mini-final'), str(BASE/'worktrees/ldd-memory-final')}
                         else 'prior session owner; no takeover authority')
        data['activity'] = 'Live owner/process association not established; mtime and cleanliness are not proof of inactivity.'
        if protected:
            data.update({'protected': True, 'inspection': 'registered Git metadata only; checkout never entered'})
            data.update(disposition('out-of-scope', 'Explicitly protected deploy-main/deploy/main; no checkout status/diff/hash or cleanup.'))
        elif name == 'uar' and w.name not in {'uar-skill-deployment-catalog', 'uar-c14-teams-work'}:
            data['inspection'] = 'registered Git metadata only; unrelated UAR checkout not entered'
            data.update(disposition('out-of-scope', 'Unrelated UAR application source not selected for pack convergence.'))
        elif 'prunable' in data:
            data.update(disposition('out-of-scope', 'Missing registered checkout; retain metadata, no prune/cleanup.'))
        else:
            data.update(changed_files(w, name))
            unique = out(root, 'rev-list', data['HEAD'], '--not', '--remotes').splitlines()
            vs_main = out(root, 'rev-list', data['HEAD'], '^'+remote).splitlines() if remote else [data['HEAD']]
            data['unique_vs_local_origin_main'] = vs_main
            data['local_only_vs_all_remote_refs'] = unique
            data['divergence_vs_local_origin_main'] = out(root, 'rev-list', '--left-right', '--count', remote+'...'+data['HEAD']) if remote else None
            if vs_main:
                data.update(disposition('include' if name in {'mini','surreal-memory','companion'} else 'out-of-scope',
                                       'Explicit source lineage candidate; production path disposition and owner coordination govern any import.'))
            else: data.update(disposition('already-merged', 'No unique commits versus current origin/main; preserve dirty files by their separate dispositions.'))
        worktrees.append(data)
    item['worktrees'] = worktrees
    refs = []
    if name != 'uar':
        for row in out(root, 'for-each-ref', '--format=%(refname)%00%(objectname)%00%(creatordate:iso-strict)', 'refs/heads', 'refs/tags', 'refs/archive', 'refs/stash').splitlines():
            ref, sha, date = row.split('\0')
            commit = out(root, 'rev-parse', '--verify', ref+'^{commit}')
            archived = any(x in ref for x in ['archive/', 'wip/', 'preserve'])
            record = {'ref': ref, 'object_sha': sha, 'commit': commit, 'created_at': date, 'archived_or_preserved': archived}
            if ref == 'refs/heads/deploy/main':
                record.update(disposition('out-of-scope', 'Protected branch identity only.'))
                record['local_only_vs_all_remote_refs'] = None
            else:
                unique = out(root, 'rev-list', commit, '--not', '--remotes').splitlines() if commit else []
                record['local_only_vs_all_remote_refs'] = unique
                if ancestor(root, commit, remote): record.update(disposition('already-merged', 'Ref commit is an ancestor of current origin/main; preserve archived ref.'))
                elif ref.startswith('refs/tags/v'):
                    record.update(disposition('superseded', 'Historical release tag; not latest-source authority; preserve unchanged.'))
                elif archived:
                    record.update(disposition('superseded', 'Preserved historical branch/ref; current candidate source governs shipping; retain any unique commits for explicit lineage review.'))
                elif name in {'mini','surreal-memory','companion'}:
                    record.update(disposition('include', 'Source lineage candidate; inspect exact commits and paths before any isolated import.'))
                else: record.update(disposition('out-of-scope', 'Unselected historical local branch; explicit source lineage retained.'))
            refs.append(record)
    item['refs'] = refs
    relevant_unique = sorted(set(s for r in refs for s in (r.get('local_only_vs_all_remote_refs') or [])))
    if name == 'uar':
        relevant_unique = sorted(set(s for w in worktrees for s in w.get('unique_vs_local_origin_main', [])))
    item['local_only_commit_details'] = commits(root, relevant_unique, name, remote)
    return item

def query_remote(url):
    rc, raw = call(['git', 'ls-remote', url, 'refs/heads/main'], timeout=25)
    return {'url': url, 'exit': rc, 'observed_main': raw.decode().split()[0] if rc == 0 and raw.split() else None,
            'observation': 'read-only ls-remote; no fetch or remote-ref mutation',
            'observed_at': datetime.datetime.now(datetime.timezone.utc).isoformat()}

result = {'schema_version': 1, 'captured_at_start': START,
          'task': {'phase': 'phase-learning-deploy-and-debt', 'change': 'change-ldd-13-source-convergence', 'backend_task_id': '1'},
          'route': {'provider': 'OpenAI', 'model': 'gpt-6.1-sol', 'reasoning_effort': 'xhigh',
                    'harness': 'Codex desktop', 'dispatch': 'native collaboration.spawn_agent fresh-context implementation worker'},
          'selected_full_skill_system_pin': SELECTED,
          'restrictions': ['No builds/tests/reviewers/inference', 'No checkout/source mutation', 'Protected deploy-main metadata only',
                           'No home/config/cache/version/tag changes', 'No cleanup', 'No certification or KBD completion claim'],
          'starting_pointer_hashes': {}, 'repositories': [], 'gitlinks': [], 'remote_main_observations': [], 'github': []}
for p in [HERE/'repository-inventory.json', Path('/tmp/ldd-inventory.py'), Path('/tmp/ldd-pin-inventory.json')]:
    result['starting_pointer_hashes'][str(p)] = hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
roots = [('full', FULL), ('mini', MINI), ('surreal-memory', BASE/'surreal-memory-server'),
         ('knowledge', BASE/'prometheus-knowledge'), ('companion', BASE/'prometheus-companion'), ('uar', BASE/'universal-agent-runtime')]
for name, root in roots:
    item = inventory_repo(name, root)
    if item: result['repositories'].append(item)
    print('Captured',name,flush=True)
dependencies = {}
for repo, root in [('full',CANDIDATE), ('mini',MINI)]:
    config = out(root,'config','-f',str(root/'.gitmodules'),'--get-regexp',r'submodule\..*\.(path|url)$')
    modules = {}
    for line in config.splitlines():
        key, _, val = line.partition(' ')
        match = re.match(r'submodule\.(.*)\.(path|url)$',key)
        if match: modules.setdefault(match[1],{})[match[2]]=val
    by_path = {m['path']:m.get('url') for m in modules.values() if 'path' in m}
    for row in out(root,'ls-tree','-r','HEAD').splitlines():
        if not row.startswith('160000 '): continue
        meta, path = row.split('\t',1); sha = meta.split()[2]; url = by_path.get(path)
        checkout = (FULL if repo == 'full' else MINI)/path
        top = out(checkout,'rev-parse','--show-toplevel') if checkout.exists() else ''
        item={'repository':repo,'source_head':out(root,'rev-parse','HEAD'),'path':path,'committed_pin':sha,'url':url,
              'checkout_path':str(checkout),'initialized':top==str(checkout)}
        if item['initialized']:
            item['checkout_head']=out(checkout,'rev-parse','HEAD')
            item['local_origin_main']=out(checkout,'rev-parse','--verify','refs/remotes/origin/main')
            item['pinned_commit_available_locally']=git(checkout,'cat-file','-e',sha+'^{commit}')[0]==0
            dependencies.setdefault(url,checkout)
        item.update(disposition('include','Account for dependency source; exact compatible pin decision and final remote refresh belong to change 03.'))
        result['gitlinks'].append(item)
for url, checkout in dependencies.items():
    if not any(r['path']==str(checkout) for r in result['repositories']):
        item=inventory_repo('gitlink:'+checkout.name,checkout)
        if item: result['repositories'].append(item)
urls=sorted(set(r['origin'] for r in result['repositories'] if r['origin'])|set(g['url'] for g in result['gitlinks'] if g['url']))
with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
    result['remote_main_observations']=list(pool.map(query_remote,urls))
for repo in ['Prometheus-AGS/prometheus-skill-system','Prometheus-AGS/prometheus-skills-mini','Prometheus-AGS/surreal-memory-server']:
    rc,raw=call(['gh','pr','list','--repo',repo,'--state','open','--limit','100','--json','number,title,url,headRefName,headRefOid,baseRefName,updatedAt'],timeout=25)
    result['github'].append({'repo':repo,'query':'all open PRs','exit':rc,'items':json.loads(raw) if rc==0 else [],'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat()})
for repo,number in [('Prometheus-AGS/prometheus-skill-system',157),('Prometheus-AGS/prometheus-skill-system',158),('Prometheus-AGS/prometheus-skills-mini',41)]:
    rc,raw=call(['gh','pr','view',str(number),'--repo',repo,'--json','number,title,url,state,headRefName,headRefOid,mergeCommit,mergedAt,updatedAt'],timeout=25)
    result['github'].append({'repo':repo,'query':'PR '+str(number),'exit':rc,'item':json.loads(raw) if rc==0 else None,'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat()})
result['captured_at_end']=datetime.datetime.now(datetime.timezone.utc).isoformat()
result['provenance_script_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
(HERE/'execute-source-inventory.json').write_text(json.dumps(result,indent=2)+'\n')
print('Wrote',HERE/'execute-source-inventory.json',flush=True)
