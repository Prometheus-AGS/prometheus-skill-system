#!/usr/bin/env node
// Domain selection through the real installer entry point, in a scratch home.
// The packaged Claude payload is the source, as it is for a marketplace install.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'domain-selection-'));
const home = path.join(scratch, 'home');
const store = path.join(home, '.prometheus/plugins/prometheus-skill-pack');
const payload = path.join(scratch, 'payload');
const legacy = path.join(scratch, 'payload-without-domain-index');
fs.mkdirSync(home);
fs.cpSync(path.join(root, 'dist/plugins/claude/prometheus-skill-pack'), payload, {
  recursive: true,
  verbatimSymlinks: true,
});
const env = {
  PATH: process.env.PATH,
  HOME: home,
  CODEX_HOME: path.join(home, '.codex'),
  XDG_CONFIG_HOME: path.join(home, '.config'),
  XDG_DATA_HOME: path.join(home, '.local/share'),
  PROMETHEUS_PLUGIN_ROOT: store,
  GIT_CONFIG_NOSYSTEM: '1',
  GIT_CONFIG_GLOBAL: os.devNull,
};
const index = JSON.parse(fs.readFileSync(path.join(payload, 'shared/skill-domains.json'), 'utf8'));
const domain = id => index.domains.find(entry => entry.id === id);
const closure = ids => {
  const seen = new Set();
  const visit = id => {
    if (seen.has(id)) return;
    seen.add(id);
    domain(id).requires.forEach(visit);
  };
  ids.forEach(visit);
  return [...seen].flatMap(id => domain(id).skills).sort();
};
const everything = index.domains.flatMap(entry => entry.skills).sort();
const ROOTS = { claude: path.join(home, '.claude/skills'), agents: path.join(home, '.agents/skills') };
const FOREIGN = 'someone-elses-skill';

function argv(source, extra) {
  return [
    path.join(source, 'scripts/install-plugin-generation.js'),
    '--source-root', source, '--home', home, '--plugin-root', store,
    '--targets', 'claude,agents', ...extra,
  ];
}
function run(label, extra = [], { source = payload, status = 0 } = {}) {
  const result = spawnSync(process.execPath, argv(source, extra), {
    cwd: scratch, env, encoding: 'utf8', timeout: 240_000, maxBuffer: 16e6,
  });
  assert.equal(result.status, status, `${label}: ${result.stderr || result.error}`);
  return result;
}
/** Pack skills in a root: everything except the foreign entry and dot files. */
function projected(which) {
  return fs.readdirSync(ROOTS[which]).filter(name => !name.startsWith('.') && name !== FOREIGN).sort();
}
function assertProjection(label, expected) {
  for (const which of Object.keys(ROOTS)) {
    assert.deepEqual(projected(which), expected, `${label}: ${which} root`);
    assert(fs.existsSync(path.join(ROOTS[which], FOREIGN, 'SKILL.md')), `${label}: foreign skill kept in ${which}`);
  }
  run(`${label} verify`, ['--verify']);
}

try {
  for (const directory of Object.values(ROOTS)) {
    fs.mkdirSync(path.join(directory, FOREIGN), { recursive: true });
    fs.writeFileSync(path.join(directory, FOREIGN, 'SKILL.md'), '---\nname: someone-elses-skill\ndescription: not the pack\'s\n---\n');
  }

  // 1. No flag and no stored selection: every skill, as before selection existed.
  run('install all');
  assertProjection('all', everything);
  assert(!fs.existsSync(path.join(store, 'selection.json')), 'no selection is stored for an unfiltered install');
  const receiptAll = JSON.parse(fs.readFileSync(
    path.join(store, 'receipts', fs.readFileSync(path.join(store, 'pointers/current'), 'utf8').trim().split('/').pop(), 'claude__skills.json'), 'utf8'));
  assert.equal(receiptAll.body.schemaVersion, 1, 'an unfiltered target keeps the schema 1 receipt');

  // 2. One domain: only its skills, and hooks still dispatch from the active store.
  run('select rust', ['--domains', 'rust']);
  assertProjection('rust', closure(['rust']));
  const hook = spawnSync(process.execPath, [path.join(payload, 'scripts/hook-entry.mjs'),
    '--bundle', JSON.parse(fs.readFileSync(path.join(payload, 'shared/harnesses/generated/release-manifest.json'), 'utf8')).bundleId,
    '--hook', 'sessionstart-vlbv', '--harness', 'claude-code'], {
    cwd: scratch, env: { ...env, CLAUDE_PLUGIN_ROOT: payload }, encoding: 'utf8', timeout: 90_000,
    input: JSON.stringify({ session_id: 'domain-selection', cwd: scratch, hook_event_name: 'SessionStart', source: 'startup' }),
  });
  assert.equal(hook.status, 0, `hook dispatch with a narrow selection: ${hook.stderr}`);
  assert.doesNotMatch(`${hook.stdout}${hook.stderr}`, /NOT_ACTIVATED|HOOK_RUNTIME_ERROR/);

  // 3. Adding a domain brings the domains it requires.
  run('add process', ['--add-domains', 'process']);
  assert(domain('process').requires.length > 0, 'the process domain declares requirements');
  assertProjection('rust+process', closure(['rust', 'process']));

  // 4. Removing a domain removes its skills and nothing else.
  run('remove rust', ['--remove-domains', 'rust']);
  assertProjection('process', closure(['process']));

  // 5. An install with no flag keeps the stored selection.
  run('plain reinstall');
  assertProjection('process kept', closure(['process']));

  // An unknown domain is refused and changes nothing.
  const refused = run('unknown domain', ['--domains', 'no-such-domain'], { status: 1 });
  assert.match(refused.stderr, /unknown domain/);
  assertProjection('after refusal', closure(['process']));

  // --list-domains reports the selection without changing it.
  const listed = JSON.parse(run('list', ['--list-domains']).stdout);
  assert.deepEqual(listed.targets['.claude/skills'].requested, ['process']);
  assert.equal(listed.targets['.claude/skills'].skills, closure(['process']).length);
  assert.equal(listed.domains.length, index.domains.length);

  // 6. An install interrupted after its journal is published recovers to the
  //    journaled selection, not to the one stored before it.
  {
    const journal = path.join(store, 'pointers/activation.pending.json');
    const child = spawn(process.execPath, argv(payload, ['--domains', 'learn']), { cwd: scratch, env, stdio: 'ignore' });
    let killed = false;
    const poll = setInterval(() => {
      if (!killed && fs.existsSync(journal)) {
        killed = true;
        child.kill('SIGKILL');
      }
    }, 1);
    await new Promise((resolve, reject) => {
      child.on('error', reject);
      child.on('exit', resolve);
    });
    clearInterval(poll);
    if (killed && fs.existsSync(journal)) {
      run('recover interrupted install');
      assertProjection('recovered to journaled selection', closure(['learn']));
    } else {
      // The install won the race; the selection must simply have been applied.
      assertProjection('uninterrupted learn', closure(['learn']));
      console.log('note: the interrupted-install case did not interrupt on this run');
    }
  }

  // 7. A generation that predates the domain index projects everything under
  //    schema 1 receipts, and rolling forward again restores the selection.
  fs.cpSync(payload, legacy, { recursive: true, verbatimSymlinks: true });
  fs.rmSync(path.join(legacy, 'shared/skill-domains.json'));
  const releaseFile = path.join(legacy, 'shared/harnesses/generated/release-manifest.json');
  if (JSON.stringify(JSON.parse(fs.readFileSync(releaseFile, 'utf8'))).includes('skill-domains.json')) {
    console.log('note: the release manifest pins shared/skill-domains.json; skipping the pre-index generation case');
  } else {
    run('install pre-index generation', [], { source: legacy });
    assertProjection('pre-index generation', everything);
    run('install indexed generation', []);
    assertProjection('selection restored', closure(['learn']));
    run('rollback to pre-index generation', ['--rollback']);
    assertProjection('rolled back', everything);
    run('roll forward', ['--rollback']);
    assertProjection('rolled forward', closure(['learn']));
  }

  // Back to everything, then a clean uninstall that leaves the foreign skill.
  run('select all', ['--domains', 'all']);
  assertProjection('all again', everything);
  assert(!fs.existsSync(path.join(store, 'selection.json')), 'selecting all clears the stored selection');
  run('uninstall', ['--uninstall']);
  for (const which of Object.keys(ROOTS)) {
    assert.deepEqual(projected(which), [], `uninstall: ${which} root`);
    assert(fs.existsSync(path.join(ROOTS[which], FOREIGN, 'SKILL.md')));
  }
  console.log('PASS: domain selection installs, adds, removes, persists, recovers and rolls back through the real installer');
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}
