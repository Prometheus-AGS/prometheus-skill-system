import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { dirname, join, resolve, delimiter } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { copyPathExact } from './support.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
test('harness installer persists receipt, reinstalls idempotently and uninstalls only owned files', async t => {
  const work = mkdtempSync(join(tmpdir(), 'builder-install ü-')); t.after(() => rmSync(work, { recursive: true, force: true }));
  const bin = join(work, 'bin'), project = join(work, 'project'); mkdirSync(join(bin, 'node_modules/fixture'), { recursive: true }); mkdirSync(project);
  const code = `import { appendFileSync } from 'node:fs'; appendFileSync(process.env.BUILDER_TEST_LOG, JSON.stringify(process.argv.slice(2))+'\\n');`;
  writeFileSync(join(bin, 'node_modules/fixture/package.json'), JSON.stringify({ name: 'fixture', bin: { npx: 'cli.mjs' } }));
  writeFileSync(join(bin, 'node_modules/fixture/cli.mjs'), code);
  if (process.platform === 'win32') writeFileSync(join(bin, 'npx.cmd'), '@echo off');
  else writeFileSync(join(bin, 'npx'), `#!/usr/bin/env node\n${code}`, { mode: 0o755 });
  const env = { ...process.env, PATH: `${bin}${delimiter}${process.env.PATH}`, BUILDER_TEST_LOG: join(work, 'commands.jsonl') };
  const run = (...extra) => spawnSync(process.execPath, [join(root, 'scripts/install-harness-package.mjs'), '--harness', 'opencode', '--scope', 'project', ...extra], { encoding: 'utf8', shell: false, cwd: project, env });
  // A preexisting sibling skill must remain unowned and survive uninstall.
  const preexisting = join(project, '.agents/skills/a11y-gate');
  mkdirSync(preexisting, { recursive: true });
  copyPathExact(join(root, 'skills/a11y-gate'), preexisting);
  let result = run(); assert.equal(result.status, 0, result.stderr);
  const receiptFile = join(project, '.knowme-builder/harness-install.json'), plugin = join(project, '.opencode/plugins/knowme-builder.mjs');
  const receipt = readFileSync(receiptFile, 'utf8'); assert.ok(existsSync(plugin));
  writeFileSync(join(project, '.opencode/plugins/unrelated.mjs'), 'user-owned');
  result = run(); assert.equal(result.status, 0, result.stderr); assert.equal(readFileSync(receiptFile, 'utf8'), receipt);
  writeFileSync(plugin, 'user modified'); result = run('--uninstall'); assert.equal(result.status, 1); assert.match(result.stderr, /preserving/); assert.ok(existsSync(receiptFile));
  writeFileSync(plugin, readFileSync(join(root, '.opencode/plugins/knowme-builder.mjs')));
  result = run('--uninstall'); assert.equal(result.status, 0, result.stderr); assert.equal(existsSync(receiptFile), false); assert.equal(existsSync(plugin), false);
  assert.equal(readFileSync(join(project, '.opencode/plugins/unrelated.mjs'), 'utf8'), 'user-owned');
  assert.ok(existsSync(join(preexisting, 'SKILL.md')));
  assert.equal(existsSync(join(work, 'commands.jsonl')), false, 'copy installation must not delegate broad npm skill removal');
});

test('Kimi Code, MiniMax Code and Zed receive portable skills in native discovery roots', t => {
  const work = mkdtempSync(join(tmpdir(), 'builder-extra-harnesses-'));
  t.after(() => rmSync(work, { recursive: true, force: true }));
  const user = join(work, 'user');
  const minimax = join(work, 'minimax-data');
  mkdirSync(user, { recursive: true });
  const env = {
    ...process.env,
    HOME: user,
    USERPROFILE: user,
    MINIMAX_DATA_DIR: minimax,
    XDG_STATE_HOME: join(work, 'state'),
  };
  const install = harness => spawnSync(process.execPath, [
    join(root, 'scripts/install-harness-package.mjs'),
    '--harness',
    harness,
    '--scope',
    'user',
  ], { encoding: 'utf8', shell: false, env });
  for (const harness of ['kimi-code', 'zed', 'minimax-code']) {
    const result = install(harness);
    assert.equal(result.status, 0, result.stderr);
  }
  assert.ok(existsSync(join(user, '.agents/skills/hybrid-mobile-architecture/SKILL.md')));
  assert.ok(existsSync(join(user, '.kimi-code/skills/hybrid-mobile-architecture/SKILL.md')));
  assert.ok(existsSync(join(minimax, 'skills/hybrid-mobile-architecture/SKILL.md')));
  const receipt = JSON.parse(readFileSync(join(work, 'state/knowme-builder/install.json'), 'utf8'));
  assert.deepEqual(receipt.harnesses.sort(), ['kimi-code', 'minimax-code', 'zed']);
});

function cliFixture(t, code) {
  const work = mkdtempSync(join(tmpdir(), 'builder-mcp ü-')); t.after(() => rmSync(work, { recursive: true, force: true }));
  const bin = join(work, 'bin'), project = join(work, 'project'); mkdirSync(join(bin, 'node_modules/fixture'), { recursive: true }); mkdirSync(project);
  writeFileSync(join(bin, 'node_modules/fixture/package.json'), JSON.stringify({ name: 'fixture', bin: { claude: 'cli.mjs' } }));
  writeFileSync(join(bin, 'node_modules/fixture/cli.mjs'), code);
  if (process.platform === 'win32') writeFileSync(join(bin, 'claude.cmd'), '@echo off'); else writeFileSync(join(bin, 'claude'), `#!/usr/bin/env node\n${code}`, { mode: 0o755 });
  const env = { ...process.env, PATH: `${bin}${delimiter}${process.env.PATH}`, BUILDER_TEST_STATE: work };
  const run = (...extra) => spawnSync(process.execPath, [join(root, 'scripts/install-harness-package.mjs'), '--harness', 'claude-code', '--scope', 'project', ...extra], { encoding: 'utf8', shell: false, cwd: project, env });
  return { work, project, run };
}

test('unknown inventory and unsupported Claude revision never become owned installs', t => {
  const { work, project, run } = cliFixture(t, `import { appendFileSync } from 'node:fs';import { join } from 'node:path';appendFileSync(join(process.env.BUILDER_TEST_STATE,'calls'),JSON.stringify(process.argv.slice(2))+'\\n');process.stdout.write('not JSON');`);
  let result = run('--ref', 'pinned-revision'); assert.equal(result.status, 1); assert.match(result.stderr, /--ref is not supported/); assert.equal(existsSync(join(work, 'calls')), false);
  result = run(); assert.equal(result.status, 1); assert.match(result.stderr, /ownership/); assert.equal(existsSync(join(project, '.knowme-builder')), false);
  const calls = readFileSync(join(work, 'calls'), 'utf8').trim().split('\n').map(JSON.parse); assert.ok(calls.every(args => args.includes('list')));
});

test('MCP uninstall preserves a user replacement instead of trusting its old name', t => {
  const { work, project, run } = cliFixture(t, `import { appendFileSync,existsSync,readFileSync,writeFileSync } from 'node:fs';import { join } from 'node:path';const args=process.argv.slice(2),root=process.env.BUILDER_TEST_STATE;appendFileSync(join(root,'calls'),JSON.stringify(args)+'\\n');if(args[0]==='plugin'&&args.includes('list'))process.stdout.write('[]');if(args[0]==='mcp'&&args[1]==='get'){const file=join(root,args[2]+'.json');if(existsSync(file))process.stdout.write(readFileSync(file,'utf8'));else process.exitCode=1;}if(args[0]==='mcp'&&args[1]==='add')writeFileSync(join(root,args[4]+'.json'),JSON.stringify({name:args[4],command:args.slice(6)}));`);
  let result = run('--with-mcp'); assert.equal(result.status, 0, result.stderr);
  writeFileSync(join(work, 'dart.json'), JSON.stringify({ name: 'dart', command: ['user-replacement'] }));
  const before = readFileSync(join(work, 'calls'), 'utf8'); result = run('--uninstall'); assert.equal(result.status, 1); assert.match(result.stderr, /MCP entry changed/);
  assert.ok(existsSync(join(project, '.knowme-builder/harness-install.json')));
  const afterCalls = readFileSync(join(work, 'calls'), 'utf8').slice(before.length).trim().split('\n').filter(Boolean).map(JSON.parse); assert.ok(afterCalls.every(args => args[0] === 'mcp' && args[1] === 'get'));
  assert.match(readFileSync(join(work, 'dart.json'), 'utf8'), /user-replacement/);
});
