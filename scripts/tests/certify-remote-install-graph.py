#!/usr/bin/env python3
"""Clone exact published refs, verify their graph, then install in owned homes.

An optional explicitly named source-read token is passed only to Git through
environment configuration. No host credential file, Git config or auth home is
copied or consulted. Manifest values must be approved, remotely existing commits.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from urllib.parse import urlparse


def main():
    ap=argparse.ArgumentParser()
    for key in ('manifest','scratch','evidence','doctor-bin'):
        ap.add_argument('--'+key,required=True,type=Path)
    args=ap.parse_args()
    if any(not value.is_absolute() for value in vars(args).values()):
        ap.error('absolute inputs required')
    work=Path(tempfile.mkdtemp(prefix='remote-release-',dir=args.scratch))
    evidence=args.evidence; evidence.mkdir(parents=True,exist_ok=True)
    env={key:os.environ[key] for key in ('PATH',) if key in os.environ}
    env.update(HOME=str(work/'home'),CODEX_HOME=str(work/'codex'),CORTEX_DATA_DIR=str(work/'cortex'),
        PROMETHEUS_PLUGIN_ROOT=str(work/'plugins'),PROMETHEUS_LEARNING_QUEUE=str(work/'queue'),
        PROMETHEUS_LEARNING_LOG_DIR=str(work/'log'),PROMETHEUS_LEARNING_INDEX_DIR=str(work/'index'),
        CLAUDE_CONFIG_DIR=str(work/'claude'),PROMETHEUS_LEARNING_CORTEX='0',
        PYTHONDONTWRITEBYTECODE='1',GIT_CONFIG_GLOBAL=os.devnull,GIT_CONFIG_NOSYSTEM='1',
        GIT_TERMINAL_PROMPT='0',GIT_SSH_COMMAND='false',GIT_ALLOW_PROTOCOL='https',TMPDIR=str(work/'tmp'))
    for key in ('HOME','CODEX_HOME','CORTEX_DATA_DIR','CLAUDE_CONFIG_DIR','TMPDIR'):
        Path(env[key]).mkdir(parents=True)
    records=[]; clones={}; code=0
    class Blocked(Exception): pass
    def run(label,argv,cwd=None,source=False):
        command=list(map(str,argv)); child_env=dict(env)
        # Credentials are explicit inputs, never read from ~/.config/gh or ~/.gitconfig.
        if source and token:
            child_env.update(GIT_CONFIG_COUNT='1',GIT_CONFIG_KEY_0='http.https://github.com/.extraHeader',
                             GIT_CONFIG_VALUE_0='Authorization: Bearer '+token)
        result=subprocess.run(command,cwd=cwd or work,env=child_env,capture_output=True,text=True,timeout=900)
        stdout=evidence/(label+'.stdout'); stderr=evidence/(label+'.stderr')
        stdout.write_text(result.stdout.replace(token,'[redacted]') if token else result.stdout)
        stderr.write_text(result.stderr.replace(token,'[redacted]') if token else result.stderr)
        records.append({'name':label,'argv':command,'exitCode':result.returncode,'stdout':str(stdout),'stderr':str(stderr)})
        if result.returncode:
            if source: raise Blocked('exact remote source unavailable: '+label)
            raise AssertionError('production entrypoint failed: '+label)
        return result.stdout
    try:
        manifest=json.loads(args.manifest.read_text()); token=''
        token_name=manifest.get('sourceReadTokenEnv')
        if token_name:
            if not token_name.startswith('LDD_'): raise Blocked('explicit LDD source token variable required')
            token=os.environ.get(token_name,'')
        sources=manifest.get('repositories',[])
        if {row.get('name') for row in sources}!={'full','mini','surreal','companion'} or len(sources)!=4:
            raise Blocked('four exact approved remote repository identities required')
        for row in sources:
            name,url,commit=row['name'],row['url'],row['commit']; parsed=urlparse(url)
            if parsed.scheme!='https' or parsed.hostname!='github.com' or parsed.username or parsed.password or parsed.query or parsed.fragment:
                raise Blocked('approved credential-free GitHub HTTPS source required')
            if len(commit)!=40 or any(c not in '0123456789abcdef' for c in commit):
                raise Blocked('exact final commit required')
            root=work/name
            run(name+'-remote-clone',['git','clone','--no-checkout','--',url,root],source=True)
            run(name+'-exact-checkout',['git','checkout','--detach',commit],root,source=True)
            run(name+'-recursive-gitlinks',['git','submodule','update','--init','--recursive','--jobs','1'],root,source=True)
            assert run(name+'-head',['git','rev-parse','HEAD'],root).strip()==commit
            statuses=run(name+'-submodule-status',['git','submodule','status','--recursive'],root)
            assert all(line.startswith(' ') for line in statuses.splitlines()), 'unresolved or divergent gitlink'
            assert not run(name+'-clean-source',['git','status','--porcelain'],root).strip(), 'fresh source dirty'
            clones[name]=root
        full,mini=clones['full'],clones['mini']
        for name in ('full','mini','surreal','companion'):
            row=next(row for row in sources if row['name']==name)
            run(name+'-committed-protected-integrity',['node',full/'scripts/verify-protected-tests.mjs',
                 '--base',row['baseCommit'],'--candidate',row['commit']],clones[name])
        for name in ('full','mini'):
            contract=json.loads((clones[name]/'skill-system.json').read_text())
            assert contract['releaseVersion']==manifest[name+'ReleaseVersion'], name+' release mismatch'
            for item in contract.get('imports',[]):
                child=clones[name]/item['path']
                assert run(name+'-pin-'+item.get('id',item['path'].replace('/','-')),['git','rev-parse','HEAD'],child).strip()==item['commit']
        # Parent memory identity must be the actual certified published source.
        memory=next(row['commit'] for row in sources if row['name']=='surreal')
        for name in ('full','mini'):
            assert run(name+'-memory-gitlink',['git','rev-parse','HEAD'],clones[name]/'tools/surreal-memory-server').strip()==memory
        for name in ('full','mini'):
            home=work/(name+'-home'); home.mkdir()
            selected=work/(name+'-codex'); selected.mkdir()
            env.update(HOME=str(home),CODEX_HOME=str(selected),PROMETHEUS_PLUGIN_ROOT=str(home/'.prometheus/plugins'))
            if name=='full':
                install=['node',full/'scripts/install-system.js','--source-root',full,'--home',home,
                         '--profile','skills','--targets','codex','--non-interactive','--yes']
                run(name+'-actual-fresh-install',install,full)
                run(name+'-actual-fresh-verify',install+['--verify'],full)
                run(name+'-selected-home-compiled-doctor',[args.doctor_bin,'doctor','--check','codex.memories','--json'],full)
            else:
                run(name+'-actual-fresh-install',['node',mini/'scripts/doctor.mjs','--fix','copy-skills'],mini)
                run(name+'-actual-fresh-update',['node',mini/'scripts/doctor.mjs','--fix','copy-skills'],mini)
            assert list((selected/'skills').glob('*/SKILL.md')), name+' installed no selected skills'
        records.append({'name':'exact-remote-install-graph','exitCode':0,'commits':{row['name']:row['commit'] for row in sources}})
    except Blocked as error:
        code=2; records.append({'exitCode':2,'reason':str(error)})
    except Exception as error:
        code=1; records.append({'exitCode':1,'reason':str(error)})
    finally:
        (evidence/'final-remote-install-graph.json').write_text(json.dumps({'exitCode':code,'records':records},indent=2)+'\n')
        shutil.rmtree(work)
    return code


if __name__=='__main__':
    raise SystemExit(main())
