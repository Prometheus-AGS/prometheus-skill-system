import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import test from 'node:test';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'../..');
const invoke=(name,args,opts={})=>spawnSync(process.execPath,[join(root,name),...args],{encoding:'utf8',...opts});
const fixture=fn=>{const work=mkdtempSync(join(tmpdir(),'hma-native ü-'));try{return fn(work);}finally{rmSync(work,{recursive:true,force:true});}};

test('runnable web and Tauri dependency locks are relocatable and match their manifests',()=>{
 for(const relative of ['assets/templates/baselines/web/web','assets/templates/baselines/tauri/desktop']){
  const directory=join(root,relative),manifest=JSON.parse(readFileSync(join(directory,'package.json'),'utf8')),lock=JSON.parse(readFileSync(join(directory,'package-lock.json'),'utf8'));
  assert.equal(lock.lockfileVersion,3);
  assert.deepEqual(lock.packages[''].dependencies,manifest.dependencies);
  assert.deepEqual(lock.packages[''].devDependencies,manifest.devDependencies);
  for(const key of Object.keys(lock.packages))assert.ok(key===''||key.startsWith('node_modules/'),`non-relocatable lock key in ${relative}: ${key}`);
 }
});

test('tray generation preserves edits, force is explicit and malformed identity is rejected',()=>fixture(work=>{
 const result=invoke('scripts/scaffold-tauri-tray.mjs',[work,'--crate-name','health-fixture','--tray-id','test-tray']);assert.equal(result.status,0,result.stderr);
 const tray=join(work,'src-tauri/src/tray.rs');assert.match(readFileSync(tray,'utf8'),/test-tray/);writeFileSync(tray,'user code\n');
 assert.equal(invoke('scripts/scaffold-tauri-tray.mjs',[work]).status,0);assert.equal(readFileSync(tray,'utf8'),'user code\n');
 assert.equal(invoke('scripts/scaffold-tauri-tray.mjs',[work,'--force']).status,0);assert.notEqual(readFileSync(tray,'utf8'),'user code\n');
 assert.notEqual(invoke('scripts/scaffold-tauri-tray.mjs',[work,'--tray-id','bad"id']).status,0);
}));
test('supervisor rendering is deterministic, escapes XML and rejects unsafe restart policy',()=>fixture(work=>{
 const args=['--label','dev.fixture','--program','/some path/app','--arg','a&b','--working-dir','/tmp/A&B','--format','launchd'];
 const first=invoke('scripts/render-supervisor-plist.mjs',args),second=invoke('scripts/render-supervisor-plist.mjs',args);assert.equal(first.status,0,first.stderr);assert.equal(first.stdout,second.stdout);assert.match(first.stdout,/a&amp;b/);assert.match(first.stdout,/\/tmp\/A&amp;B/);
 const file=join(work,'test.plist');assert.equal(invoke('scripts/render-supervisor-plist.mjs',[...args,'--out',file]).status,0);
 assert.notEqual(invoke('scripts/render-supervisor-plist.mjs',[...args,'--throttle','9']).status,0);
 const systemd=invoke('scripts/render-supervisor-plist.mjs',['--label','dev.fixture','--program','/some path/app','--arg','hello world','--format','systemd']);assert.equal(systemd.status,0,systemd.stderr);assert.match(systemd.stdout,/ExecStart="\/some path\/app" "hello world"/);
}));
test('one token source renders Dart and CSS and cleanup defaults to preserving duplicates',()=>fixture(work=>{
 mkdirSync(join(work,'desktop/src'),{recursive:true});mkdirSync(join(work,'mobile/lib/core/theme'),{recursive:true});
 const generated=invoke('scripts/gen-design-tokens.mjs',[work]);assert.equal(generated.status,0,generated.stderr);
 assert.match(readFileSync(join(work,'desktop/src/theme.css'),'utf8'),/--color-bgCanvas: #0D0D18/);assert.match(readFileSync(join(work,'mobile/lib/core/theme/tokens.dart'),'utf8'),/bgCanvas = Color\(0xFF0D0D18\)/);
 const extensions=join(work,'extensions');for(const version of ['1.0.0','2.0.0']){const path=join(extensions,`pub.ext-${version}`);mkdirSync(path,{recursive:true});writeFileSync(join(path,'package.json'),JSON.stringify({publisher:'pub',name:'ext',version}));}
 const dry=invoke('cleanup_antigravity_ide_exts.mjs',['--root',extensions]);assert.equal(dry.status,0,dry.stderr);assert.equal(readdirSync(extensions).length,2);
 assert.equal(invoke('cleanup_antigravity_ide_exts.mjs',['--root',extensions,'--apply']).status,0);assert.deepEqual(readdirSync(extensions),['pub.ext-2.0.0']);
}));
test('Git inventory preserves Unicode paths and wiki consolidation validates before changing content',()=>fixture(work=>{
 const git=(...args)=>{const result=spawnSync('git',args,{cwd:work,encoding:'utf8'});assert.equal(result.status,0,result.stderr);return result.stdout.trim();};
 git('init','--initial-branch=main');
 const scopes={root:'.prometheus',desktop:'apps/knowme-poc/desktop/.prometheus','src-tauri':'apps/knowme-poc/desktop/src-tauri/.prometheus',rust:'apps/knowme-poc/rust/.prometheus'};
 for(const [scope,path] of Object.entries(scopes)){const wiki=join(work,path,'knowledge/wiki');mkdirSync(wiki,{recursive:true});const minimum={root:238,desktop:20,'src-tauri':3,rust:34}[scope];for(let i=0;i<minimum-2;i++)writeFileSync(join(wiki,`page-${i}.md`),`---\ntype: Reference\nid: ${scope}-${i}\ntitle: Page ${i}\nrevision: 1\n---\n\nOriginal ${scope} ${i}\n`);}
 const originalBinary=Buffer.from([0,255,254,1,200]);mkdirSync(join(work,'.prometheus/assets'),{recursive:true});writeFileSync(join(work,'.prometheus/assets/vector.bin'),originalBinary);
 for(const name of ['a','b']){mkdirSync(join(work,'.prometheus/knowledge/wiki',name));writeFileSync(join(work,'.prometheus/knowledge/wiki',name,'shared.md'),`---\ntype: Reference\nid: shared-${name}\ntitle: Shared ${name}\n---\nDistinct body ${name}\n`);}
 git('add','.');const tree=git('write-tree');const snapshot=spawnSync('git',['commit-tree',tree],{cwd:work,input:'fixture\n',encoding:'utf8',env:{...process.env,GIT_AUTHOR_NAME:'Fixture',GIT_AUTHOR_EMAIL:'fixture@example.invalid',GIT_COMMITTER_NAME:'Fixture',GIT_COMMITTER_EMAIL:'fixture@example.invalid'}});assert.equal(snapshot.status,0,snapshot.stderr);git('update-ref','refs/heads/main',snapshot.stdout.trim());git('update-ref','refs/heads/codex/pre-consolidation-main-snapshot',snapshot.stdout.trim());
 const modifiedBinary=Buffer.from([0,128,254,1,201]);writeFileSync(join(work,'.prometheus/assets/vector.bin'),modifiedBinary);
 writeFileSync(join(work,'.prometheus/knowledge/wiki/page-0.md'),'---\ntype: Reference\nid: root-0\ntitle: Page zero\nrevision: 2\n---\nNew body Bearer abcdefghijklmnopqrstuv\n');
 writeFileSync(join(work,'Untracked ü file.txt'),'preserve me');
 const inventory=invoke('scripts/worktree-consolidation-inventory.mjs',['inventory.json'],{cwd:work});assert.equal(inventory.status,0,inventory.stderr);const inv=JSON.parse(readFileSync(join(work,'inventory.json'),'utf8'));assert.ok(inv.worktrees[0].dirty_files.some(file=>file.path==='Untracked ü file.txt'&&file.sha256));
 const merge=invoke('scripts/consolidate-prometheus-wikis.mjs',['--manifest','merge.json'],{cwd:work});assert.equal(merge.status,0,merge.stderr);const page=readFileSync(join(work,'.prometheus/knowledge/wiki/page-0.md'),'utf8');assert.match(page,/Original root 0/);assert.match(page,/New body Bearer \[REDACTED_SECRET\]/);assert.ok(!page.includes('abcdefghijklmnopqrstuv'));const manifest=JSON.parse(readFileSync(join(work,'merge.json'),'utf8'));assert.ok(manifest.scopes.root.wiki.mappings.some(item=>item.history_destination));
 assert.deepEqual(readFileSync(join(work,'.prometheus/assets/vector.bin')),originalBinary);assert.deepEqual(readFileSync(join(work,'.prometheus/consolidation/worktree-wiki-merge/files/integration/assets/vector.bin')),modifiedBinary);
 for(const name of ['a','b'])assert.match(readFileSync(join(work,'.prometheus/knowledge/wiki/history/worktree-consolidation/primary',name,'shared.md'),'utf8'),new RegExp(`Distinct body ${name}`));
 git('update-ref','-d','refs/heads/codex/pre-consolidation-main-snapshot');const before=readFileSync(join(work,'.prometheus/knowledge/wiki/page-0.md'),'utf8');assert.notEqual(invoke('scripts/consolidate-prometheus-wikis.mjs',[],{cwd:work}).status,0);assert.equal(readFileSync(join(work,'.prometheus/knowledge/wiki/page-0.md'),'utf8'),before);
 assert.ok(existsSync(join(work,'Untracked ü file.txt')));
}));

test('Tauri boot gate runs isolated process, requires readiness signals, preserves logs and stops the process',()=>fixture(work=>{
 const app=join(work,'app-fixture.mjs');
 writeFileSync(app,`import{mkdirSync,writeFileSync}from'node:fs';import{join}from'node:path';if(process.env.GEN_UI_APP_DATA_DIR){const root=process.env.GEN_UI_APP_DATA_DIR;writeFileSync(join(root,'pid'),String(process.pid));if(process.env.BOOT_FIXTURE_MODE==='exit')process.exit(3);for(const dir of ['diagnostics','config-db','memory-db','model-cache/fastembed'])mkdirSync(join(root,dir),{recursive:true});writeFileSync(join(root,'diagnostics/desktop.log'),'desktop migrations ready\\nseed load ready\\nsync ready in local-only mode\\n');setInterval(()=>{},1000);}`);
 const env={...process.env,NODE_OPTIONS:`--import=${pathToFileURL(app).href}`};
 const data=join(work,'data');const ready=invoke('scripts/verify-tauri-boot.mjs',[process.execPath,data,'10'],{env,timeout:25000});assert.equal(ready.status,0,ready.stderr);assert.match(ready.stdout,/Tauri boot proof passed/);assert.ok(existsSync(join(data,'tauri-process.log')));
 const pid=Number(readFileSync(join(data,'pid'),'utf8'));assert.throws(()=>process.kill(pid,0));
 const failed=invoke('scripts/verify-tauri-boot.mjs',[process.execPath,join(work,'failed'),'10'],{env:{...env,BOOT_FIXTURE_MODE:'exit'},timeout:25000});assert.equal(failed.status,1,failed.stderr);assert.match(failed.stderr,/exited before reaching ready/);
}));

test('Cargokit patch upgrades canonical pin idempotently and refuses unknown source before writing',()=>fixture(work=>{
 const dir=join(work,'rust_builder/cargokit/build_tool/lib/src');mkdirSync(dir,{recursive:true});
 const options="enum Toolchain {\n  stable,\n  beta,\n  nightly,\n}\n\nfinal Toolchain toolchain;\nToolchain toolchain = Toolchain.stable;\nstatic Toolchain _toolchainFromNode(YamlNode node) {\n  throw SourceSpanException('old parser', node.span);\n}\n";
 writeFileSync(join(dir,'options.dart'),options);writeFileSync(join(dir,'builder.dart'),'unknown upstream layout');writeFileSync(join(dir,'rustup.dart'),'Pattern nonCustom = RegExp(r"^(stable|beta|nightly)");');writeFileSync(join(dir,'build_pod.dart'),"import 'builder.dart';\n    if (staticLibs.isNotEmpty) {\n    }\n");
 assert.equal(invoke('scripts/patch-cargokit-ios.mjs',[work]).status,1);assert.equal(readFileSync(join(dir,'options.dart'),'utf8'),options);assert.ok(!existsSync(join(dir,'dedup_archive.dart')));
 writeFileSync(join(dir,'builder.dart'),"String get _toolchain => _buildOptions?.toolchain.name ?? 'stable';");
 const result=invoke('scripts/patch-cargokit-ios.mjs',[work]);assert.equal(result.status,0,result.stderr);const patchedOptions=readFileSync(join(dir,'options.dart'),'utf8');assert.match(patchedOptions,/String toolchain = '1\.97\.1'/);assert.equal((patchedOptions.match(/static String _toolchainFromNode/g)??[]).length,1);assert.ok(!patchedOptions.includes('old parser'));const patched=readFileSync(join(dir,'build_pod.dart'),'utf8');assert.match(patched,/dedupArchiveMembers/);
 assert.equal(invoke('scripts/patch-cargokit-ios.mjs',[work]).status,0);assert.equal(readFileSync(join(dir,'build_pod.dart'),'utf8'),patched);
}));
