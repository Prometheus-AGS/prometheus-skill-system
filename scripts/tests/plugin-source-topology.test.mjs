/**
 * The plugin source topology check: where each client's marketplace entry for this pack
 * points, and whether that location is safe to depend on.
 *
 * Incident (2026-10-05): a topic-branch git worktree was registered as the marketplace
 * `directory` source and deleted after its branch merged. Every running session then
 * failed every Stop hook with "Plugin directory does not exist", and nothing had warned.
 * The hazard is a topic-branch checkout, not a linked worktree as such: a dedicated
 * release-line worktree is a legitimate, stable source.
 */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import {
  evaluateSources,
  gitFacts,
  isReleaseLineBranch,
  nativeCacheSkew,
  readRegisteredSources,
} from '../lib/plugin-source-topology.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const NAME = 'prometheus-skill-pack';
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'plugin-source-topology-'));
process.on('exit', () => fs.rmSync(tmp, { recursive: true, force: true }));

const git = (cwd, ...args) => {
  const run = spawnSync('git', ['-c', 'user.email=t@t', '-c', 'user.name=t', ...args], { cwd, encoding: 'utf8' });
  assert.equal(run.status, 0, `git ${args.join(' ')}: ${run.stderr}`);
  return run.stdout.trim();
};

function makeCheckout(dir, { branch = 'main', dirty = false } = {}) {
  fs.mkdirSync(dir, { recursive: true });
  git(dir, 'init', '-q', '-b', 'main');
  fs.writeFileSync(path.join(dir, 'a.txt'), 'a');
  git(dir, 'add', '.');
  git(dir, 'commit', '-q', '-m', 'init');
  if (branch !== 'main') git(dir, 'checkout', '-q', '-b', branch);
  if (dirty) fs.writeFileSync(path.join(dir, 'a.txt'), 'changed');
  return dir;
}

function makeHome(entries) {
  const home = fs.mkdtempSync(path.join(tmp, 'home-'));
  fs.mkdirSync(path.join(home, '.claude/plugins'), { recursive: true });
  fs.mkdirSync(path.join(home, '.codex'), { recursive: true });
  if (entries.claudeKnown) {
    fs.writeFileSync(
      path.join(home, '.claude/plugins/known_marketplaces.json'),
      JSON.stringify({ [NAME]: { source: { source: 'directory', path: entries.claudeKnown }, installLocation: entries.claudeKnown } })
    );
  }
  if (entries.claudeSettings) {
    fs.writeFileSync(
      path.join(home, '.claude/settings.json'),
      JSON.stringify({ extraKnownMarketplaces: { [NAME]: { source: { source: 'directory', path: entries.claudeSettings } } } })
    );
  }
  if (entries.codex) {
    fs.writeFileSync(
      path.join(home, '.codex/config.toml'),
      `model = "x"\n\n[marketplaces.${NAME}]\nsource_type = "local"\nsource = "${entries.codex}"\n\n[tui]\nx = 1\n`
    );
  }
  return home;
}

const codes = result => result.findings.map(finding => finding.code).sort();

// --- release-line classification ------------------------------------------------------
assert.equal(isReleaseLineBranch('main'), true);
assert.equal(isReleaseLineBranch('master'), true);
assert.equal(isReleaseLineBranch('deploy/main'), true);
assert.equal(isReleaseLineBranch('release/1.11'), true);
assert.equal(isReleaseLineBranch('fix/dist-ship-script-lib'), false);
assert.equal(isReleaseLineBranch('codex/kbd-task-model-planning'), false);
assert.equal(isReleaseLineBranch(null), true, 'a detached checkout (for example a tag) is a legitimate source');

// --- reading what each client registered ----------------------------------------------
{
  const checkout = makeCheckout(path.join(tmp, 'pack-main'));
  const home = makeHome({ claudeKnown: checkout, claudeSettings: checkout, codex: checkout });
  const sources = readRegisteredSources({ home });
  assert.deepEqual(sources.map(source => source.client).sort(), ['claude', 'claude', 'codex']);
  assert(sources.every(source => source.path === checkout));
}

// --- evaluation -----------------------------------------------------------------------
{
  const good = makeCheckout(path.join(tmp, 'good'));
  const home = makeHome({ claudeKnown: good, codex: good });
  const result = evaluateSources(readRegisteredSources({ home }));
  assert.equal(result.status, 'ok', JSON.stringify(result.findings));
}
{
  const gone = path.join(tmp, 'removed-worktree');
  const home = makeHome({ claudeKnown: gone, codex: gone });
  const result = evaluateSources(readRegisteredSources({ home }));
  assert.equal(result.status, 'fail');
  assert(codes(result).includes('SOURCE_MISSING'));
  assert(result.findings.find(f => f.code === 'SOURCE_MISSING').message.includes(gone));
}
{
  const topic = makeCheckout(path.join(tmp, 'topic'), { branch: 'fix/something' });
  const home = makeHome({ claudeKnown: topic });
  const result = evaluateSources(readRegisteredSources({ home }));
  assert.equal(result.status, 'warn');
  assert(codes(result).includes('TOPIC_BRANCH'));
  assert.equal(result.findings.find(f => f.code === 'TOPIC_BRANCH').branch, 'fix/something');
}
{
  const dirty = makeCheckout(path.join(tmp, 'dirty'), { dirty: true });
  const result = evaluateSources(readRegisteredSources({ home: makeHome({ codex: dirty }) }));
  assert.equal(result.status, 'warn');
  assert(codes(result).includes('SOURCE_DIRTY'));
}
{
  const a = makeCheckout(path.join(tmp, 'a'));
  const b = makeCheckout(path.join(tmp, 'b'));
  const result = evaluateSources(readRegisteredSources({ home: makeHome({ claudeKnown: a, codex: b }) }));
  assert(codes(result).includes('SOURCES_DISAGREE'));
  assert.equal(result.status, 'warn');
}
{
  // A release-line linked worktree is a legitimate source: noted, not a warning.
  const main = makeCheckout(path.join(tmp, 'main-repo'));
  const wt = path.join(tmp, 'deploy-main');
  git(main, 'worktree', 'add', '-q', '-b', 'deploy/main', wt);
  const result = evaluateSources(readRegisteredSources({ home: makeHome({ claudeKnown: wt, codex: wt }) }));
  assert.equal(result.status, 'ok', JSON.stringify(result.findings));
  assert(codes(result).includes('LINKED_WORKTREE'));
  assert(result.findings.every(f => f.severity === 'info'));
}
{
  const nothing = evaluateSources(readRegisteredSources({ home: makeHome({}) }));
  assert.equal(nothing.status, 'skip');
}

// --- native cache vs active generation ------------------------------------------------
{
  const home = makeHome({});
  const cache = path.join(home, '.claude/plugins/cache/prometheus-skill-pack/prometheus-skill-pack');
  fs.mkdirSync(path.join(cache, '1.11.1'), { recursive: true });
  fs.mkdirSync(path.join(cache, '1.10.0/.in_use'), { recursive: true });
  fs.writeFileSync(path.join(cache, '1.10.0/.in_use', String(process.pid)), '1');
  fs.writeFileSync(path.join(cache, '1.10.0/.in_use', '999999999'), '1');
  fs.writeFileSync(
    path.join(home, '.claude/plugins/installed_plugins.json'),
    JSON.stringify({ plugins: { [`${NAME}@${NAME}`]: [{ version: '1.11.1', installPath: path.join(cache, '1.11.1') }] } })
  );
  const gen = path.join(home, '.prometheus/plugins/prometheus-skill-pack/current');
  fs.mkdirSync(gen, { recursive: true });
  fs.writeFileSync(path.join(gen, 'skill-system.json'), JSON.stringify({ releaseVersion: '1.11.2' }));
  const skew = nativeCacheSkew({ home });
  assert.equal(skew.installedVersion, '1.11.1');
  assert.equal(skew.activeGeneration, '1.11.2');
  assert(skew.findings.some(f => f.code === 'CACHE_BEHIND_GENERATION'));
  const live = skew.findings.find(f => f.code === 'LIVE_SESSIONS_ON_SUPERSEDED');
  assert.deepEqual(live.pids, [process.pid], 'only live processes are reported');
  assert.equal(live.version, '1.10.0');
  assert(skew.findings.every(f => f.severity === 'info' || f.severity === 'warn'));
}
{
  // Nothing installed: nothing to compare, nothing reported.
  const skew = nativeCacheSkew({ home: makeHome({}) });
  assert.deepEqual(skew.findings, []);
}

// --- CLI contract ---------------------------------------------------------------------
{
  const topic = makeCheckout(path.join(tmp, 'cli-topic'), { branch: 'feat/x' });
  const home = makeHome({ claudeKnown: topic });
  const run = (...args) =>
    spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home, ...args], { encoding: 'utf8' });
  const json = run('--json');
  assert.equal(json.status, 0, 'warnings do not fail the check');
  assert.equal(JSON.parse(json.stdout).topology.status, 'warn');
  assert.equal(run('--enforce', topic).status, 3, 'a registered source on a topic branch is refused');
  assert.equal(run('--enforce', topic, '--allow-topic-branch').status, 0, 'the override is explicit');
  const elsewhere = makeCheckout(path.join(tmp, 'cli-unregistered'), { branch: 'feat/y' });
  assert.equal(run('--enforce', elsewhere).status, 0, 'a topic-branch checkout that is not a registered source is not blocked');
  const missing = makeHome({ claudeKnown: path.join(tmp, 'nope') });
  const failing = spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', missing, '--json'], { encoding: 'utf8' });
  assert.equal(failing.status, 1, 'a missing registered source fails the check');
}

// --- the installer scripts honour the guard ------------------------------------------
{
  // Run the REAL scripts from a fixture checkout that is a registered source on a topic branch.
  const checkout = makeCheckout(path.join(tmp, 'installer-topic'), { branch: 'fix/installer' });
  fs.mkdirSync(path.join(checkout, 'scripts/lib'), { recursive: true });
  for (const file of ['update-skill-pack.sh', 'refresh-native-plugin-installs.sh', 'check-plugin-source.js', 'lib/plugin-source-topology.js']) {
    fs.copyFileSync(path.join(root, 'scripts', file), path.join(checkout, 'scripts', file));
  }
  git(checkout, 'add', '-A');
  git(checkout, 'commit', '-q', '-m', 'scripts');
  const home = makeHome({ claudeKnown: checkout });
  const env = { ...process.env, HOME: home };
  const update = (...args) => spawnSync('bash', [path.join(checkout, 'scripts/update-skill-pack.sh'), ...args], { encoding: 'utf8', env });
  const refresh = (...args) =>
    spawnSync('bash', [path.join(checkout, 'scripts/refresh-native-plugin-installs.sh'), '--source-root', checkout, '--generation', 'g', ...args], { encoding: 'utf8', env });

  const blockedUpdate = update();
  assert.equal(blockedUpdate.status, 3, 'update-skill-pack.sh propagates the guard exit status');
  assert(/topic branch 'fix\/installer'/.test(blockedUpdate.stderr), blockedUpdate.stderr);
  assert(!blockedUpdate.stdout.includes('Step 1'), 'the guard runs before any update step');
  const blockedRefresh = refresh();
  assert.equal(blockedRefresh.status, 3);
  assert(/refusing/.test(blockedRefresh.stderr));

  // With the explicit override the guard steps aside (the run then proceeds to its own, later checks).
  assert(!/refusing/.test(update('--allow-topic-branch').stderr), 'the override is honoured by update-skill-pack.sh');
  assert.notEqual(refresh('--allow-topic-branch').status, 3, 'the override is honoured by refresh-native-plugin-installs.sh');
}

// --- symlinked aliases of a registered source are still the registered source --------
{
  const real = makeCheckout(path.join(tmp, 'alias-real'), { branch: 'feat/alias' });
  const alias = path.join(tmp, 'alias-link');
  fs.symlinkSync(real, alias);
  const home = makeHome({ claudeKnown: alias });
  const run = (...args) =>
    spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home, ...args], { encoding: 'utf8' });
  assert.equal(run('--enforce', real).status, 3, 'registered through a symlink, enforced by its physical path');
  const home2 = makeHome({ claudeKnown: real });
  const run2 = (...args) =>
    spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home2, ...args], { encoding: 'utf8' });
  assert.equal(run2('--enforce', alias).status, 3, 'registered by its physical path, enforced through a symlink');
}

// --- a corrupt native cache never hides the topology report ---------------------------
{
  const missing = path.join(tmp, 'gone-again');
  const home = makeHome({ claudeKnown: missing });
  const cache = path.join(home, '.claude/plugins/cache/prometheus-skill-pack');
  fs.mkdirSync(cache, { recursive: true });
  fs.writeFileSync(path.join(cache, 'prometheus-skill-pack'), 'not a directory');
  fs.writeFileSync(
    path.join(home, '.claude/plugins/installed_plugins.json'),
    JSON.stringify({ plugins: { [`${NAME}@${NAME}`]: [{ version: '1.11.1' }] } })
  );
  assert.doesNotThrow(() => nativeCacheSkew({ home }));
  const run = spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home, '--json'], { encoding: 'utf8' });
  assert.equal(run.status, 1, run.stderr);
  const report = JSON.parse(run.stdout);
  assert(report.topology.findings.some(f => f.code === 'SOURCE_MISSING'), 'the missing source is still reported');
}

// --- Codex TOML: every accepted spelling is read, and an unreadable one is never ignored
{
  const checkout = makeCheckout(path.join(tmp, 'toml-main'));
  const spellings = {
    'quoted table key': `[marketplaces."${NAME}"]\nsource_type = "local"\nsource = "${checkout}"\n`,
    'single-quoted key and literal strings': `[marketplaces.'${NAME}']\nsource_type = 'local'\nsource = '${checkout}'\n`,
    'spaces around the dots': `[ marketplaces . ${NAME} ]\nsource_type = "local"\nsource = "${checkout}"\n`,
    'dotted keys at the root': `marketplaces.${NAME}.source_type = "local"\nmarketplaces.${NAME}.source = "${checkout}"\n`,
    'keys in either order with a comment': `[marketplaces.${NAME}] # pack\nsource = "${checkout}"\nsource_type = "local"\n`,
  };
  for (const [label, toml] of Object.entries(spellings)) {
    const home = makeHome({});
    fs.writeFileSync(path.join(home, '.codex/config.toml'), `model = "x"\n${toml}\n[tui]\nx = 1\n`);
    const sources = readRegisteredSources({ home });
    assert.deepEqual(sources.map(source => source.path), [checkout], `Codex registration read from: ${label}`);
  }
  // A registration the reader cannot interpret is reported, never silently dropped.
  const home = makeHome({});
  fs.writeFileSync(path.join(home, '.codex/config.toml'), `[marketplaces.${NAME}]\nsource_type = "git"\nurl = "https://example.test/x.git"\n`);
  const sources = readRegisteredSources({ home });
  assert.equal(sources.length, 1);
  assert.equal(sources[0].unreadable, true);
  const result = evaluateSources(sources);
  assert(codes(result).includes('REGISTRATION_UNREADABLE'));
  assert.equal(result.status, 'warn');
  // And a quoted-key registration of a removed directory still fails the run.
  const gone = path.join(tmp, 'toml-gone');
  const failHome = makeHome({});
  fs.writeFileSync(path.join(failHome, '.codex/config.toml'), `[marketplaces."${NAME}"]\nsource_type = "local"\nsource = "${gone}"\n`);
  assert.equal(evaluateSources(readRegisteredSources({ home: failHome })).status, 'fail');
}

// --- an unreadable registration never breaks the installer guard -----------------------
{
  const gitRegistration = `[marketplaces.${NAME}]\nsource_type = "git"\nurl = "https://example.test/x.git"\n`;
  const topic = makeCheckout(path.join(tmp, 'unreadable-topic'), { branch: 'feat/unreadable' });
  const other = makeCheckout(path.join(tmp, 'unreadable-other'), { branch: 'feat/other' });
  const enforce = (home, target, ...extra) =>
    spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home, '--enforce', target, ...extra], { encoding: 'utf8' });

  // Unreadable Codex registration only: an unregistered checkout is never blocked, and nothing crashes.
  const onlyUnreadable = makeHome({});
  fs.writeFileSync(path.join(onlyUnreadable, '.codex/config.toml'), gitRegistration);
  const unregistered = enforce(onlyUnreadable, other);
  assert.equal(unregistered.status, 0, unregistered.stderr);
  assert(!/TypeError|ERR_INVALID_ARG/.test(unregistered.stderr), 'no crash on a null path');

  // The same unreadable registration beside a registered topic-branch source: that source is still refused.
  const mixed = makeHome({ claudeKnown: topic });
  fs.writeFileSync(path.join(mixed, '.codex/config.toml'), gitRegistration);
  assert.equal(enforce(mixed, topic).status, 3, 'the registered topic-branch checkout is still refused');
  assert.equal(enforce(mixed, other).status, 0, 'an unregistered checkout is still not blocked');

  // The advisory finding survives in the report.
  const report = spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', mixed, '--json'], { encoding: 'utf8' });
  assert.equal(report.status, 0, report.stderr);
  assert(JSON.parse(report.stdout).topology.findings.some(f => f.code === 'REGISTRATION_UNREADABLE'));
}

// --- TOML escapes in a Codex path are decoded; Claude registry read failures are reported
{
  const real = makeCheckout(path.join(tmp, 'esc-pack'), { branch: 'feat/escaped' });
  // 'p' as \u0070 and the rest of the name as plain text: valid TOML for the same path.
  const escaped = real.replace('esc-pack', 'esc-\\u0070ack').replace(/\\(?!u)/g, '\\\\');
  const home = makeHome({});
  fs.writeFileSync(path.join(home, '.codex/config.toml'), `[marketplaces.${NAME}]\nsource_type = "local"\nsource = "${escaped}"\n`);
  assert.deepEqual(readRegisteredSources({ home }).map(s => s.path), [real], 'a \\uXXXX escape decodes to the real path');
  assert.equal(evaluateSources(readRegisteredSources({ home })).status, 'warn', 'the existing source is not reported missing');
  assert(!codes(evaluateSources(readRegisteredSources({ home }))).includes('SOURCE_MISSING'));
  const enforce = spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home, '--enforce', real], { encoding: 'utf8' });
  assert.equal(enforce.status, 3, 'the escaped registration is recognised as the registered topic-branch source');
  // Other escapes: \t, \n, \U, and an invalid code point is unreadable, not a path.
  const odd = makeHome({});
  fs.writeFileSync(path.join(odd, '.codex/config.toml'), `[marketplaces.${NAME}]\nsource_type = "local"\nsource = "/tmp/a\\U00000062c\\tx"\n`);
  assert.equal(readRegisteredSources({ home: odd })[0].path, '/tmp/abc\tx');
  const bad = makeHome({});
  fs.writeFileSync(path.join(bad, '.codex/config.toml'), `[marketplaces.${NAME}]\nsource_type = "local"\nsource = "/tmp/\\UFFFFFFFF"\n`);
  assert.equal(readRegisteredSources({ home: bad })[0].unreadable, true);
}
{
  for (const [file, label] of [['plugins/known_marketplaces.json', 'known_marketplaces.json'], ['settings.json', 'settings.json']]) {
    const home = makeHome({});
    fs.writeFileSync(path.join(home, '.claude', file), '{ not json');
    const result = evaluateSources(readRegisteredSources({ home }));
    assert.equal(result.status, 'warn', `${label} malformed`);
    assert(codes(result).includes('REGISTRATION_UNREADABLE'), `${label} failure is reported, not read as "nothing registered"`);
  }
  // Absent files are simply "nothing registered".
  const absent = makeHome({});
  fs.rmSync(path.join(absent, '.claude/plugins'), { recursive: true, force: true });
  assert.equal(evaluateSources(readRegisteredSources({ home: absent })).status, 'skip');
}

// --- malformed registrations never crash evaluation or hide another client's source ------
{
  const gone = path.join(tmp, 'mixed-missing');
  const run = home =>
    spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home, '--json'], { encoding: 'utf8' });
  for (const badPath of [{}, 5, null, ['x'], true, '', '   ']) {
    const home = makeHome({});
    fs.writeFileSync(
      path.join(home, '.claude/plugins/known_marketplaces.json'),
      JSON.stringify({ [NAME]: { source: { source: 'directory', path: badPath } } })
    );
    fs.writeFileSync(path.join(home, '.codex/config.toml'), `[marketplaces.${NAME}]\nsource_type = "local"\nsource = "${gone}"\n`);
    assert.doesNotThrow(() => evaluateSources(readRegisteredSources({ home })), `a ${JSON.stringify(badPath)} path does not throw`);
    const result = evaluateSources(readRegisteredSources({ home }));
    assert.equal(result.status, 'fail', `${JSON.stringify(badPath)}: the Codex source is still reported missing`);
    assert(codes(result).includes('SOURCE_MISSING') && codes(result).includes('REGISTRATION_UNREADABLE'));
    const cli = run(home);
    assert.equal(cli.status, 1, cli.stderr);
    JSON.parse(cli.stdout);
  }
  // A Codex config that is a directory (or otherwise unreadable) is reported, not ignored.
  const dirHome = makeHome({});
  fs.mkdirSync(path.join(dirHome, '.codex/config.toml'));
  const dirResult = evaluateSources(readRegisteredSources({ home: dirHome }));
  assert(codes(dirResult).includes('REGISTRATION_UNREADABLE'));
  assert.equal(dirResult.status, 'warn');
  // A github-sourced marketplace has no directory to check: nothing registered, not unreadable.
  const gh = makeHome({});
  fs.writeFileSync(
    path.join(gh, '.claude/plugins/known_marketplaces.json'),
    JSON.stringify({ [NAME]: { source: { source: 'github', repo: 'o/r' } } })
  );
  assert.equal(readRegisteredSources({ home: gh }).filter(s => s.client === 'claude').length, 0);
}

// --- unsupported Codex spellings are reported, never read as "nothing registered" ----------
{
  const spellings = {
    'inline table under [marketplaces]': `[marketplaces]\n${NAME} = { source_type = "local", source = "/gone" }\n`,
    'inline table at the root': `marketplaces = { ${NAME} = { source_type = "local", source = "/gone" } }\n`,
    'array of tables': `[[marketplaces]]\nname = "${NAME}"\nsource = "/gone"\n`,
  };
  for (const [label, toml] of Object.entries(spellings)) {
    const home = makeHome({});
    fs.writeFileSync(path.join(home, '.codex/config.toml'), toml);
    const result = evaluateSources(readRegisteredSources({ home }));
    assert(codes(result).includes('REGISTRATION_UNREADABLE'), `${label} is reported, not silently absent`);
    assert.equal(result.status, 'warn');
  }
  // A comment that merely names the marketplace is not a registration.
  const commented = makeHome({});
  fs.writeFileSync(path.join(commented, '.codex/config.toml'), `# marketplaces.${NAME} is configured elsewhere\nmodel = "x"\n`);
  assert.deepEqual(readRegisteredSources({ home: commented }), []);
}

// --- TOML scoping: dotted keys inside another table are not a root registration ----------
{
  const topic = makeCheckout(path.join(tmp, 'scoping-topic'), { branch: 'feat/scoping' });
  const home = makeHome({});
  fs.writeFileSync(
    path.join(home, '.codex/config.toml'),
    `[other]\nmarketplaces.${NAME}.source_type = "local"\nmarketplaces.${NAME}.source = "${topic}"\n`
  );
  assert.equal(readRegisteredSources({ home }).filter(s => s.path).length, 0, 'not a root marketplace registration');
  const enforce = spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', home, '--enforce', topic], { encoding: 'utf8' });
  assert.equal(enforce.status, 0, 'an unregistered checkout is not blocked by keys that belong to another table');
  // The same keys at the document root, before any table, ARE the registration.
  fs.writeFileSync(
    path.join(home, '.codex/config.toml'),
    `marketplaces.${NAME}.source_type = "local"\nmarketplaces.${NAME}.source = "${topic}"\n[other]\nx = 1\n`
  );
  assert.deepEqual(readRegisteredSources({ home }).map(s => s.path), [topic]);
}

// --- git probe failures are never read as clean or release-line -------------------------
{
  const checkout = makeCheckout(path.join(tmp, 'git-failures'));
  const ok = (stdout = '') => ({ status: 0, stdout, stderr: '' });
  const real = (dir, args) => {
    const run = spawnSync('git', ['--no-optional-locks', ...args], { cwd: dir, encoding: 'utf8' });
    return { status: run.status, stdout: (run.stdout ?? '').trim(), stderr: (run.stderr ?? '').trim() };
  };
  // A detached HEAD is the expected exit 1 and stays a legitimate source.
  git(checkout, 'checkout', '-q', '--detach');
  assert.equal(gitFacts(checkout).branch, null);
  assert.equal(evaluateSources(readRegisteredSources({ home: makeHome({ claudeKnown: checkout }) })).status, 'ok');
  // Real failures throw, and the evaluator reports them instead of passing.
  const failing = name => (dir, args) => (args.includes(name) ? { status: 128, stdout: '', stderr: `fatal: ${name} exploded` } : real(dir, args));
  assert.throws(() => gitFacts(checkout, failing('status')), /git status failed/);
  assert.throws(() => gitFacts(checkout, failing('symbolic-ref')), /symbolic-ref failed/);
  const entries = readRegisteredSources({ home: makeHome({ claudeKnown: checkout }) });
  const result = evaluateSources(entries, { facts: dir => gitFacts(dir, failing('status')) });
  assert(codes(result).includes('INSPECTION_FAILED'));
  assert.equal(result.status, 'warn', 'an inspection failure is advisory, never a pass and never a hard failure');
  void ok;
}

// --- clients registering aliases of one checkout agree ---------------------------------
{
  const real = makeCheckout(path.join(tmp, 'xclient-real'));
  const alias = path.join(tmp, 'xclient-alias');
  fs.symlinkSync(real, alias);
  const result = evaluateSources(readRegisteredSources({ home: makeHome({ claudeKnown: real, codex: alias }) }));
  assert(!codes(result).includes('SOURCES_DISAGREE'), 'a symlink alias of the same checkout is not a disagreement');
  assert.equal(result.status, 'ok', JSON.stringify(result.findings));
}

// --- untracked files make a source dirty ----------------------------------------------
{
  const checkout = makeCheckout(path.join(tmp, 'untracked-only'));
  fs.writeFileSync(path.join(checkout, 'new-skill.md'), 'untracked');
  const result = evaluateSources(readRegisteredSources({ home: makeHome({ claudeKnown: checkout }) }));
  assert(codes(result).includes('SOURCE_DIRTY'), 'an untracked, non-ignored file is a dirty source');
}

// --- the probe never writes the checkout it inspects ----------------------------------
{
  const checkout = makeCheckout(path.join(tmp, 'readonly-probe'));
  // Change a tracked file's stat data without changing its content: a plain `git status`
  // refreshes and rewrites the index; a read-only probe must leave it byte-identical.
  const tracked = path.join(checkout, 'a.txt');
  const later = new Date(Date.now() + 5000);
  fs.utimesSync(tracked, later, later);
  const index = path.join(checkout, '.git/index');
  const before = fs.readFileSync(index);
  const run = spawnSync(process.execPath, [path.join(root, 'scripts/check-plugin-source.js'), '--home', makeHome({ claudeKnown: checkout }), '--json'], { encoding: 'utf8' });
  assert.equal(run.status, 0, run.stderr);
  assert(Buffer.compare(before, fs.readFileSync(index)) === 0, 'the git index is unchanged by the probe');
  // Control: the unprotected command really does rewrite it, so the assertion above has teeth.
  spawnSync('git', ['status', '--porcelain'], { cwd: checkout });
  assert(Buffer.compare(before, fs.readFileSync(index)) !== 0, 'control: a plain git status refreshes the index');
}

console.log('PASS: plugin source topology and native cache skew are classified, and the installer guard is precise');
