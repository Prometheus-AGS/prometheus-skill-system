import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { After, Then, When } from '@cucumber/cucumber';

const bootstrap = fileURLToPath(new URL('../../skills/process/prometheus-context-bootstrap/scripts/bootstrap.sh', import.meta.url));
const names=['AGENTS.md','CLAUDE.md'];
const axiom='Always think in terms of user goals and value—not in tests or assumptions.';
const gitEnv=Object.fromEntries(Object.entries(process.env).filter(([k])=>!k.startsWith('GIT_')));
const git=(r,...args)=>execFileSync('git',['-C',r,'-c','core.hooksPath=/dev/null',...args],{
  encoding:'utf8', env:gitEnv, stdio:['ignore','pipe','pipe'],
});
const write=(r,n,v)=>fs.writeFileSync(path.join(r,n),v);
const read=(r,n)=>fs.readFileSync(path.join(r,n));
const link=(r,n,t)=>fs.symlinkSync(t,path.join(r,n));
function newRepo(w, agents, claude) {
  const r=fs.mkdtempSync(path.join(os.tmpdir(),'full-resident-policy-'));
  (w.roots ??= []).push(r);
  if (agents !== undefined) write(r,'AGENTS.md',agents);
  if (claude !== undefined) write(r,'CLAUDE.md',claude);
  write(r,'not-instructions.md','unrelated content\n');
  git(r,'init','-q','--template=');
  return r;
}
function run(r, ...flags) {
  const result=spawnSync('bash',[bootstrap,'--path',r,...flags],{encoding:'utf8',timeout:25000,env:gitEnv});
  if(result.error) throw result.error;
  return result;
}
function ok(result) {
  assert.equal(result.status,0,result.stdout + result.stderr);
  return result.stdout;
}
function blocked(result, regex) {
  assert.equal(result.status,2,result.stdout + result.stderr);
  assert.match(result.stderr,/BLOCKED/);
  if(regex) assert.match(result.stderr, regex);
}
function snapshot(r) {
  const names=fs.readdirSync(r).filter(n=>n!=='.git').sort();
  return { index:git(r,'ls-files','--stage'), entries:names.map(n=>{
    const p=path.join(r,n); const s=fs.lstatSync(p);
    return [n,s.mode & 0o777,s.isSymbolicLink()?fs.readlinkSync(p):s.isFile()?fs.readFileSync(p).toString('hex'):'directory'];
  }) };
}
function preview(r) {
  const output=ok(run(r,'--preview'));
  const digest=output.match(/^PLAN_SHA256=([a-f0-9]{64})$/m)?.[1];
  assert.ok(digest,output);
  return {output,digest};
}
function expanded(r,n,before) {
  const after=read(r,n);
  assert.ok(after.subarray(0,before.length).equals(before),n+' prefix');
  assert.equal(after.toString().split(axiom).length,2,n+' unique policy');
}
After(function() {
  for (const root of this.roots ?? []) fs.rmSync(root,{recursive:true,force:true});
});
When('I work through a dirty two-document update',function(){
  const r=this.r=newRepo(this,'# A\r\ncustom baseline\r\n','# Claude\nclient workflow');
  git(r,'add','AGENTS.md','CLAUDE.md');
  fs.appendFileSync(path.join(r,'AGENTS.md'),'user owns this edit');
  fs.appendFileSync(path.join(r,'CLAUDE.md'),'\nindependent owner edit');
  fs.chmodSync(path.join(r,'AGENTS.md'),0o640);
  this.before=snapshot(r);
  this.beforeBytes=names.map(n=>read(r,n));
  const audit=ok(run(r,'--audit'));
  assert.doesNotMatch(audit,/user owns this edit/);
  assert.match(audit,/CHANGE AGENTS\.md/);
  assert.match(audit,/CHANGE CLAUDE\.md/);
  assert.deepEqual(snapshot(r),this.before);
  const p=preview(r);
  assert.match(p.output,/\+\+\+ b\/AGENTS\.md/);
  assert.match(p.output,/\+\+\+ b\/CLAUDE\.md/);
  assert.match(p.output,/\+## Goals-first development/);
  assert.deepEqual(snapshot(r),this.before);
  this.out=ok(run(r,'--apply',p.digest));
});
Then('audit and preview are read-only, and apply changes only approved policy',function(){
  assert.match(this.out,/Applied 2 named instruction change/);
  names.forEach((n,i)=>expanded(this.r,n,this.beforeBytes[i]));
  const after=snapshot(this.r);
  assert.equal(after.index,this.before.index);
  assert.deepEqual(after.entries.filter(([n])=>!names.includes(n)),this.before.entries.filter(([n])=>!names.includes(n)));
  assert.equal(fs.statSync(path.join(this.r,'AGENTS.md')).mode & 0o777,0o640);
});
Then('a second approved apply is a no-op',function(){
  const before=snapshot(this.r);
  const signal=preview(this.r);
  assert.doesNotMatch(signal.output,/^\+\+\+/m);
  assert.match(ok(run(this.r,'--apply',signal.digest)),/Applied 0 named instruction change/);
  assert.deepEqual(snapshot(this.r),before);
});
When('I change a named instruction or its Git staging after preview',function(){
  for(const change of ['bytes','index']) {
    const r=newRepo(this,'# A\n','# C\n');
    const p=preview(r);
    if(change==='bytes') fs.appendFileSync(path.join(r,'CLAUDE.md'),'later personal edit');
    else git(r,'add','CLAUDE.md');
    const before=snapshot(r);
    blocked(run(r,'--apply',p.digest),/stale/);
    assert.deepEqual(snapshot(r),before);
  }
  this.protected=true;
});
Then('the stale digest is refused without changing either document',function(){assert.equal(this.protected,true);});
When('I update relative symlinks and recognized existing instruction routes',function(){
  const routes=[
    ['AGENTS.md','CLAUDE.md',null],
    ['CLAUDE.md','AGENTS.md',null],
    ['CLAUDE.md','AGENTS.md','@./AGENTS.md\n\nClaude notes\n'],
    ['AGENTS.md','CLAUDE.md','**Canonical instructions live in [\x60CLAUDE.md\x60](./CLAUDE.md).** Read it in full.\n\nOperator notes\n'],
    ['AGENTS.md','CLAUDE.md','The full project instructions live in **[CLAUDE.md](CLAUDE.md)** — project notes.\n'],
  ];
  for(const [alias,target,body] of routes) {
    const r=newRepo(this);
    write(r,target,'# canonical document\nKeep preexisting text');
    if(body===null) link(r,alias,target); else write(r,alias,body);
    git(r,'add','AGENTS.md','CLAUDE.md');
    const before=snapshot(r); const prior=read(r,target); const p=preview(r);
    assert.deepEqual(snapshot(r),before);
    assert.match(p.output,new RegExp('\\+\\+\\+ b/'+target.replace('.','\\.')));
    assert.match(ok(run(r,'--apply',p.digest)),/Applied 1 named instruction change/);
    expanded(r,target,prior);
    const after=snapshot(r);
    assert.equal(after.index,before.index);
    assert.deepEqual(after.entries.filter(([n])=>n!==target),before.entries.filter(([n])=>n!==target));
  }
  // Well-formed examples without authority are carried byte-for-byte.
  const example = '# A\n\n\x60\x60\x60text\nordinary example\n\x60\x60\x60\x60\n<!-- harmless context -->\n';
  const e=newRepo(this,example,'# C\n');
  const old=read(e,'AGENTS.md');
  const ex=preview(e);
  assert.match(ok(run(e,'--apply',ex.digest)),/Applied 2 named instruction change/);
  expanded(e,'AGENTS.md',old);
  // Never-existing peers remain eligible in both unborn and established repositories.
  for (const committed of [false,true]) {
    const r=newRepo(this,'# A\n');
    if(committed){
      git(r,'add','AGENTS.md');
      git(r,'-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--no-gpg-sign','-qm','fixture baseline');
    }
    const p=preview(r);
    assert.match(ok(run(r,'--apply',p.digest)),/Applied 2 named instruction change/);
    expanded(r,'CLAUDE.md',Buffer.alloc(0));
  }
  this.preserved=true;
});
Then('each operation changes only the canonical regular destination',function(){assert.equal(this.preserved,true);});
When('I offer ignored, conflicting, malformed, circular, external or generated instruction entries',function(){
  const cases=[
    r=>{write(r,'.gitignore','AGENTS.md\n');git(r,'add','CLAUDE.md');},
    r=>write(r,'AGENTS.md','Never prioritize user goals.\n'),
    r=>write(r,'AGENTS.md','<!-- prometheus-base:start v1 -->\nMissing end\n'),
    r=>{for(const n of names)fs.unlinkSync(path.join(r,n));link(r,'AGENTS.md','CLAUDE.md');link(r,'CLAUDE.md','AGENTS.md');},
    r=>{fs.unlinkSync(path.join(r,'AGENTS.md'));link(r,'AGENTS.md','../private/CLAUDE.md');},
    r=>write(r,'CLAUDE.md','# Demo — agent constitution (Layer 0)\nEdit rules/src, never this file.\n'),
    r=>{git(r,'add','CLAUDE.md');fs.unlinkSync(path.join(r,'CLAUDE.md'));},
    r=>{
      git(r,'add','AGENTS.md','CLAUDE.md');
      git(r,'-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--no-gpg-sign','-qm','fixture baseline');
      git(r,'rm','--','CLAUDE.md');
    },
    r=>write(r,'AGENTS.md','# A\n<!--\n## Goals-first development\n'+axiom+'\naccepted specification, VLBV. Preserve established gates.\n-->\n'),
    r=>write(r,'AGENTS.md','# A\n~~~md\n## Goals-first development\n'+axiom+'\naccepted specification, VLBV. Preserve established gates.\n~~~~\n'),
    r=>write(r,'CLAUDE.md','# C\n\x60\x60\x60md\n@AGENTS.md\n\x60\x60\x60\x60\n'),
    r=>write(r,'CLAUDE.md','# C\n<!--\n@AGENTS.md\n-->\n'),
    r=>write(r,'AGENTS.md','# A\n> '+axiom+'\n'),
    r=>write(r,'AGENTS.md','# A\n\x60\x60\x60text\nunfinished notes'),
    r=>write(r,'CLAUDE.md','# C\n<!-- unfinished notes'),
  ];
  for(const setup of cases){
    const r=newRepo(this,'# A\n','# C\n');setup(r);
    const before=snapshot(r);
    blocked(run(r,'--audit'));
    blocked(run(r,'--preview'));
    assert.deepEqual(snapshot(r),before);
  }
  this.rejected=true;
});
Then('each operation is blocked without creating, replacing or mutating an instruction entry',function(){assert.equal(this.rejected,true);});
When('I request policy audit and the original bootstrap dry run',function(){
  const r=newRepo(this);
  const before=snapshot(r);
  blocked(run(r,'--audit'),/both root instruction files are missing/);
  assert.deepEqual(snapshot(r),before);
  const legacy=ok(run(r,'--stacks','go','--profile','mixed','--no-hooks','--dry-run'));
  assert.match(legacy,/Starting prometheus-context-bootstrap/);
  assert.match(legacy,/AGENTS\.md/);
  assert.doesNotMatch(legacy,/PLAN_SHA256=/);
  assert.deepEqual(snapshot(r),before);
  this.distinct=true;
});
Then('each remains a different operation and the existing checkout is untouched',function(){assert.equal(this.distinct,true);});
When('I combine policy preview with legacy force',function(){
  const r=newRepo(this);
  const before=snapshot(r);
  const result=run(r,'--preview','--force');
  assert.equal(result.status,1,result.stdout + result.stderr);
  assert.match(result.stderr,/USAGE/);
  assert.deepEqual(snapshot(r),before);
  this.noScaffold=true;
});
Then('the combined command is rejected before it can scaffold',function(){assert.equal(this.noScaffold,true);});
