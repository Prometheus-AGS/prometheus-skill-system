#!/usr/bin/env node
// Real installed CLI, native signed runtime, filesystem and configured services.
// No fixture server replaces model inference, memory or Companion control.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { spawnSync, spawn } from 'node:child_process';
import { createHash, generateKeyPairSync, randomUUID } from 'node:crypto';
import { request } from 'node:http';

const results = [], commands = [], options = new Map();
let scratch, evidence, env, project, teamCli, binary, installed, scratchOwned = false;
const hash = value => createHash('sha256').update(value).digest('hex');
function blocked(message) { throw Object.assign(new Error(message), { blocked: true }); }
function within(root, file) { const rel = path.relative(root, file); return rel && rel !== '..' && !rel.startsWith(`..${path.sep}`) && !path.isAbsolute(rel); }
function futurePath(value) {
  let ancestor = path.resolve(value); const missing = [];
  while (!fs.existsSync(ancestor)) { missing.unshift(path.basename(ancestor)); ancestor = path.dirname(ancestor); }
  return path.join(fs.realpathSync(ancestor), ...missing);
}
function write(file, value) { fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, typeof value === 'string' ? value : `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 }); }
function read(file) { return JSON.parse(fs.readFileSync(file, 'utf8')); }
function filesystemSnapshot(root) {
  const rows = [];
  const visit = (directory, prefix = '') => {
    for (const name of fs.readdirSync(directory).sort()) {
      const relative = prefix ? `${prefix}/${name}` : name, file = path.join(directory, name);
      const stat = fs.lstatSync(file);
      rows.push({ path: relative, mode: stat.mode & 0o7777, kind: stat.isDirectory() ? 'directory' : stat.isSymbolicLink() ? 'symlink' : 'file',
        sha256: stat.isDirectory() ? null : hash(stat.isSymbolicLink() ? fs.readlinkSync(file) : fs.readFileSync(file)) });
      if (stat.isDirectory()) visit(file, relative);
    }
  };
  visit(root); return rows;
}
function command(program, args, input, extraEnv = {}) {
  const result = spawnSync(program, args, { cwd: project, env: { ...env, ...extraEnv }, input,
    encoding: 'utf8', timeout: 60000, maxBuffer: 16 * 1024 * 1024, shell: false });
  commands.push({ program, args, exitCode: result.status, stdoutSha256: hash(result.stdout ?? ''), stderrSha256: hash(result.stderr ?? '') });
  if (result.error?.code === 'ENOENT') blocked(`required actual executable missing: ${program}`);
  if (result.status !== 0 && /transport_unavailable_or_redirect_refused|credential_environment_unavailable/.test(result.stderr ?? ''))
    blocked('required actual configured provider transport or credential is unavailable');
  if (result.status !== 0) throw Error(`${path.basename(program)} failed (${result.status}): ${result.stderr}`);
  return result.stdout;
}
function team(commandName, input) {
  const file = path.join(scratch, 'requests', `${randomUUID()}.json`); write(file, input);
  return JSON.parse(command(process.execPath, [teamCli, commandName, '--input', file]));
}
function native(...args) { return JSON.parse(command(binary, ['kbd', '--path', project, ...args])); }
async function scenario(name, action) {
  try { const detail = await action(); results.push({ name, result: 'PASS', detail }); }
  catch (error) { results.push({ name, result: error.blocked || ['ENOENT', 'EACCES', 'ENOSPC'].includes(error.code) ? 'BLOCKED' : 'FAIL', diagnosis: error.message }); }
}
function loopbackEndpoint(value, label) {
  if (!value) blocked(`${label} requires an explicitly started real scratch service`);
  const url = new URL(value);
  if (url.protocol !== 'http:' || !['127.0.0.1', '[::1]', 'localhost'].includes(url.hostname) ||
      !url.port || url.port === '23001' || url.username || url.password || url.search || url.hash)
    blocked(`${label} must use an explicit non-23001 loopback port without URL credentials`);
  return url;
}
async function requireService(url, label) {
  try {
    const response = await fetch(url.origin, { redirect: 'error', signal: AbortSignal.timeout(5000) });
    await response.body?.cancel();
  } catch { blocked(`${label} actual scratch service is unavailable`); }
}
function unixHealth(socketPath) {
  return new Promise((resolve, reject) => {
    const exchange = request({ socketPath, path: '/health', method: 'GET', agent: false }, response => {
      response.resume();
      response.on('end', () => response.statusCode === 200 ? resolve() : reject(Error(`socket health returned ${response.statusCode}`)));
      response.on('error', reject);
    });
    exchange.setTimeout(1000, () => exchange.destroy(Error('socket health timed out')));
    exchange.on('error', reject); exchange.end();
  });
}
async function withControlHost(action) {
  if (process.platform === 'win32') blocked('selected Companion Unix-socket integration requires a supported Unix platform');
  const supplied = options.get('--control-host-bin');
  if (!path.isAbsolute(supplied)) blocked('--control-host-bin requires an absolute path to the actual compiled headless host');
  let host;
  try { host = fs.realpathSync(supplied); fs.accessSync(host, fs.constants.X_OK); }
  catch { blocked('actual compiled Companion headless binary is unavailable'); }
  const socket = path.join(scratch, 'control.sock');
  if (Buffer.byteLength(socket) > 100) blocked('scratch path is too long for a portable Unix socket; choose a shorter explicit scratch root');
  // No pack root: attaching the optional platform supervisor could restart real
  // launchctl labels. This scenario exercises local KBD authority and transport.
  const childEnv = { ...env, SOVEREIGN_SYNC_SOCKET: socket };
  delete childEnv.PROMETHEUS_CONTROL_ENDPOINT; delete childEnv.PROMETHEUS_PACK_ROOT;
  const child = spawn(host, [], { cwd: project, env: childEnv, stdio: ['ignore', 'pipe', 'pipe'] });
  let spawnError, stdout = '', stderr = '';
  child.on('error', error => { spawnError = error; });
  child.stdout.on('data', bytes => { stdout = (stdout + bytes).slice(-65536); });
  child.stderr.on('data', bytes => { stderr = (stderr + bytes).slice(-65536); });
  const closed = new Promise(resolve => child.once('close', (code, signal) => resolve({ code, signal })));
  try {
    const deadline = Date.now() + 30000;
    while (true) {
      if (spawnError) blocked(`actual Companion host could not start: ${spawnError.message}`);
      if (child.exitCode !== null || child.signalCode !== null) throw Error(`actual Companion host exited before authority was queried: ${stderr}`);
      try { await unixHealth(socket); break; } catch (error) {
        if (Date.now() >= deadline) throw Error(`actual Companion socket did not become available: ${error.message}; ${stderr}`);
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }
    return await action({ PROMETHEUS_KBD_CONTROL_PLANE: '1', PROMETHEUS_CONTROL_ENDPOINT: undefined,
      SOVEREIGN_SYNC_SOCKET: socket }, { endpoint: `unix://${socket}`, hostProcess: child.pid,
      hostBinary: host, hostBinarySha256: hash(fs.readFileSync(host)), serviceSupervision: 'not attached; real user service management excluded' });
  } finally {
    if (child.exitCode === null && child.signalCode === null && !spawnError) {
      child.kill('SIGTERM');
      let timer;
      const exited = await Promise.race([closed, new Promise(resolve => { timer = setTimeout(() => resolve(null), 5000); })]);
      clearTimeout(timer);
      if (!exited) child.kill('SIGKILL');
    }
    const termination = await closed;
    commands.push({ program: host, args: [], pid: child.pid, exitCode: termination.code, signal: termination.signal,
      stdoutSha256: hash(stdout), stderrSha256: hash(stderr), cleanup: 'only the owned spawned host process' });
  }
}
function simultaneousNativeStatus() {
  const run = () => new Promise((resolve, reject) => {
    const args = ['kbd', '--path', project, 'status', '--json'];
    const child = spawn(binary, args, { cwd: project, env, stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '', stderr = '';
    const timer = setTimeout(() => { child.kill('SIGTERM'); reject(Error('native status process timed out')); }, 60000);
    child.stdout.on('data', bytes => { stdout += bytes; }); child.stderr.on('data', bytes => { stderr += bytes; });
    child.on('error', error => { clearTimeout(timer); reject(error); });
    child.on('close', code => { clearTimeout(timer); commands.push({ program: binary, args, exitCode: code, pid: child.pid });
      if (code !== 0) reject(Error(`native status failed: ${stderr}`)); else { try { resolve(JSON.parse(stdout)); } catch (error) { reject(error); } } });
  });
  return Promise.all([run(), run()]);
}

let definition, stateFile, projectId;
try {
  const allowed = new Set(['--scratch', '--evidence', '--installed-root', '--runtime-bin', '--model-input', '--memory-endpoint', '--control-endpoint', '--control-host-bin']);
  for (let i = 2; i < process.argv.length; i += 2) {
    if (!allowed.has(process.argv[i]) || options.has(process.argv[i]) || !process.argv[i + 1]) blocked('invalid scenario arguments');
    options.set(process.argv[i], process.argv[i + 1]);
  }
  for (const flag of ['--scratch', '--evidence', '--installed-root', '--runtime-bin'])
    if (!options.has(flag) || !path.isAbsolute(options.get(flag))) blocked(`${flag} requires an absolute path`);
  scratch = futurePath(options.get('--scratch')); evidence = futurePath(options.get('--evidence'));
  installed = fs.realpathSync(options.get('--installed-root')); binary = path.resolve(options.get('--runtime-bin'));
  if (!within(scratch, evidence)) blocked('--evidence must be beneath owned scratch');
  if (scratch === installed || within(installed, scratch) || within(scratch, installed)) blocked('scenario scratch and installed payload must be disjoint');
  if (fs.existsSync(scratch) && (fs.lstatSync(scratch).isSymbolicLink() || fs.readdirSync(scratch).length)) blocked('--scratch must be new or empty');
  fs.mkdirSync(scratch, { recursive: true, mode: 0o700 });
  scratchOwned = true;
  const lexicalScratch = scratch;
  scratch = fs.realpathSync(scratch);
  evidence = path.join(scratch, path.relative(lexicalScratch, evidence));
  if (Number(process.versions.node.split('.')[0]) < 22) blocked('installed team runtime requires Node22+');
  if (process.platform === 'win32') blocked('native signed migration and selected control transport require a supported Unix platform');
  if (options.has('--control-host-bin') && options.has('--control-endpoint')) blocked('choose one explicit connected host contract');
  if (!fs.existsSync(binary)) blocked('explicit native prometheus binary is unavailable; coordinator must compile it first');
  try { fs.accessSync(binary, fs.constants.X_OK); } catch { blocked('explicit native prometheus binary is not executable'); }
  teamCli = path.join(installed, 'skills/agent-team-creator/scripts/cli.mjs');
  if (!fs.existsSync(teamCli)) blocked('installed full team CLI is unavailable');
  project = path.join(scratch, 'project'); fs.mkdirSync(project);
  const home = path.join(scratch, 'home'); fs.mkdirSync(home);
  env = { PATH: `${path.dirname(binary)}${path.delimiter}${process.env.PATH ?? ''}`, HOME: home,
    CODEX_HOME: path.join(home, '.codex'), CLAUDE_CONFIG_DIR: path.join(home, '.claude'), XDG_CONFIG_HOME: path.join(home, '.config'),
    XDG_DATA_HOME: path.join(home, '.local/share'), XDG_CACHE_HOME: path.join(home, '.cache'), TMPDIR: path.join(scratch, 'tmp'),
    PROMETHEUS_DATA_DIR: path.join(scratch, 'runtime-data'), PROMETHEUS_DEVICE_KEY_FILE: path.join(scratch, 'device-key.json'),
    PROMETHEUS_KBD_CONTROL_PLANE: '0', PROMETHEUS_CONTROL_ENDPOINT: 'http://127.0.0.1:1',
    PROMETHEUS_PLUGIN_ROOT: installed, CLAUDE_PLUGIN_ROOT: installed, PROMETHEUS_SKILL_PACK_ROOT: installed,
    PROMETHEUS_SKILLS_DIR: path.join(installed, 'skills'),
    PROMETHEUS_HARNESS: 'codex', PROMETHEUS_TEAM_DIGEST_DIR: path.join(scratch, 'digest'),
    PROMETHEUS_LEARNING_LOG_DIR: path.join(scratch, 'learning-log'), PROMETHEUS_LEARNING_INDEX_DIR: path.join(scratch, 'learning-index'),
    PROMETHEUS_LEARNING_CORTEX: '0', PROMETHEUS_LEARNING_PK: '0', CORTEX_DATA_DIR: path.join(scratch, 'cortex'),
    PROMETHEUS_LEARNING_QUEUE: path.join(scratch, 'learning-queue'),
    PYTHONDONTWRITEBYTECODE: '1', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: path.join(scratch, 'gitconfig'),
    OPENSPEC_TELEMETRY: '0', DO_NOT_TRACK: '1' };
  fs.mkdirSync(env.TMPDIR); fs.mkdirSync(env.PROMETHEUS_DATA_DIR); write(env.GIT_CONFIG_GLOBAL, '');
  const pair = generateKeyPairSync('ed25519'), privateJwk = pair.privateKey.export({ format: 'jwk' }), publicJwk = pair.publicKey.export({ format: 'jwk' });
  write(env.PROMETHEUS_DEVICE_KEY_FILE, { schemaVersion: '1', keyId: `ed25519:${hash(Buffer.from(publicJwk.x, 'base64url'))}`,
    privateKey: Buffer.from(privateJwk.d, 'base64url').toString('base64') });
  command('git', ['init']); command('git', ['config', '--local', 'user.name', 'Local integration']);
  command('git', ['config', '--local', 'user.email', 'local-integration@invalid']); command('git', ['config', '--local', 'commit.gpgsign', 'false']);
  write(path.join(project, 'evidence.md'), 'Production entry-point integration in an isolated project.\n');
  command('git', ['add', 'evidence.md']); command('git', ['commit', '-m', 'Initialize scratch project']);
  fs.mkdirSync(path.join(project, '.kbd-orchestrator'));
  definition = { schemaVersion: 1, id: 'integration-team', outcome: 'Preserve isolated task handoff and learning', scope: 'project', harness: 'codex',
    roles: ['implementer', 'reviewer'].map(id => ({ id, description: `${id} responsibility`, prompt: 'Use the project instructions and record actual evidence.',
      skills: [], owns: id === 'implementer' ? ['src/'] : ['evidence/'], inputs: ['Requirements'], outputs: ['Evidence'], dependsOn: [] })) };
  stateFile = path.join(project, '.agent-team/integration-team/state.json');

  await scenario('native projection migration preserves signed journal and frontier', async () => {
    native('status', '--json');
    native('phase', 'create', '--command-id', 'integration-phase-create', '--id', 'integration-phase', '--title', 'Integration phase');
    native('phase', 'activate', '--command-id', 'integration-phase-activate', '--id', 'integration-phase', '--exact-next-work', 'Read isolated evidence');
    const before = native('status', '--json'), auditBefore = command(binary, ['kbd', '--path', project, 'audit', '--json']);
    projectId = before.projectId;
    assert.ok(projectId, 'native runtime must return real project identity');
    const progressFile = path.join(project, '.kbd-orchestrator/phases/integration-phase/progress.json');
    const unmarked = read(progressFile); delete unmarked.generatedBy; write(progressFile, unmarked);
    const original = fs.readFileSync(progressFile);
    const beforeDryRun = filesystemSnapshot(scratch);
    const inventory = native('migrate', '--projections', '--dry-run');
    assert.ok(inventory.projections.some(row => row.disposition === 'adopt'));
    assert.deepEqual(filesystemSnapshot(scratch), beforeDryRun, 'dry run creates no files/locks and changes no bytes or modes');
    assert.deepEqual(fs.readFileSync(progressFile), original, 'dry run changes no input projection');
    assert.equal(command(binary, ['kbd', '--path', project, 'audit', '--json']), auditBefore);
    const adopted = native('migrate', '--projections');
    assert.ok(adopted.projections.some(row => row.disposition === 'adopt' && row.receiptPath));
    assert.equal(read(progressFile).generatedBy, 'kbd-runtime');
    const divergent = read(progressFile); divergent.phase = 'unexpected-legacy-value'; write(progressFile, divergent);
    const divergentBytes = fs.readFileSync(progressFile);
    const archived = native('migrate', '--projections');
    const row = archived.projections.find(item => item.disposition === 'archive');
    assert.ok(row?.archivePath && within(project, row.archivePath));
    assert.deepEqual(fs.readFileSync(row.archivePath), divergentBytes, 'archive preserves exact original');
    assert.equal(command(binary, ['kbd', '--path', project, 'audit', '--json']), auditBefore, 'migration appends no canonical event');
    const after = native('status', '--json');
    assert.equal(after.revision, before.revision); assert.deepEqual(after.frontier, before.frontier);
    const [first, second] = await simultaneousNativeStatus();
    assert.deepEqual(first, second, 'independent native processes read the same persisted canonical state');
    return { projectId, revision: after.revision, journalSha256: hash(auditBefore), receipts: [...adopted.receiptPaths, ...archived.receiptPaths] };
  });

  await scenario('installed team manifest tasks native export and handoff persist across processes', () => {
    if (!projectId) blocked('native project initialization prerequisite failed');
    team('install-project', { project, team: definition, target: 'codex' });
    assert.equal(read(path.join(project, '.agent-team/project-routing.json')).activeTeam, definition.id);
    assert.ok(fs.existsSync(path.join(project, '.codex/agents/implementer.toml')));
    team('init', { state: stateFile, team: definition });
    let state = team('task', { state: stateFile, expectedRevision: 0, task: { action: 'add', id: 'implementation', title: 'Transfer actual persisted context', owner: 'implementer' } });
    state = team('handoff-create', { state: stateFile, expectedRevision: state.revision, cwd: project,
      handoff: { taskId: 'implementation', owner: 'implementer', expectedTaskRevision: state.tasks[0].revision,
        toOwner: 'reviewer', toHarness: 'claude', context: 'Inspect the isolated evidence.', evidence: ['evidence.md'], remaining: ['Read evidence'], memoryRefs: [] } });
    const packet = state.handoffs.at(-1); assert.ok(packet.git.head); assert.equal(packet.git.root, project);
    state = team('handoff-accept', { state: stateFile, expectedRevision: state.revision, id: packet.id, destination: { owner: 'reviewer', harness: 'claude' } });
    assert.equal(state.tasks[0].owner, 'reviewer'); assert.equal(state.tasks[0].harness, 'claude');
    assert.ok(state.handoffs.at(-1).acceptedAt); assert.deepEqual(team('status', { state: stateFile }), state);
    return { teamId: definition.id, revision: state.revision, handoffId: packet.id, nativeDefinitions: '.codex/agents/' };
  });

  await scenario('actual configured model discovery selection and inference', async () => {
    const inputFile = options.get('--model-input');
    if (!inputFile || !path.isAbsolute(inputFile) || !fs.existsSync(inputFile)) blocked('--model-input must name a real configured discovery/inference recipe');
    const recipe = read(inputFile);
    for (const reference of [recipe.discovery?.auth?.env, recipe.inference?.auth?.env].filter(Boolean)) {
      if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(reference) || !process.env[reference]) blocked('required explicitly referenced model credential is unavailable');
      if (Object.hasOwn(env, reference) || /^(?:NODE_|GIT_|LD_|DYLD_|PYTHON|PROMETHEUS_|SOVEREIGN_|CORTEX_|XDG_|CLAUDE_|CODEX_)/.test(reference))
        blocked('model credential reference must not override scratch paths, executable loading or selected runtime configuration');
      env[reference] = process.env[reference];
    }
    const catalog = team('models-discover', recipe.discovery);
    const selection = team('models-select', { team: definition, roleId: 'implementer', skills: [], taskPolicy: recipe.taskPolicy ?? {}, catalog });
    assert.ok(selection.selected, 'real discovered model must satisfy declared policy');
    const url = new URL(recipe.inference.url);
    if ((url.protocol !== 'https:' && !(url.protocol === 'http:' && ['127.0.0.1', 'localhost', '[::1]'].includes(url.hostname))) || url.username || url.password || url.search || url.hash)
      blocked('inference route must be credential-free HTTPS or explicit loopback HTTP');
    const headers = { 'Content-Type': 'application/json' };
    if (recipe.inference.auth) {
      const auth = recipe.inference.auth;
      const header = auth.header ?? 'Authorization', scheme = auth.scheme ?? (header === 'Authorization' ? 'Bearer' : 'raw');
      if (!['Authorization', 'X-API-Key'].includes(header) || !['Bearer', 'raw'].includes(scheme) || /[\r\n]/.test(env[auth.env])) blocked('unsupported inference credential reference');
      headers[header] = (scheme === 'raw' ? '' : 'Bearer ') + env[auth.env];
    }
    let response;
    try { response = await fetch(url, { method: 'POST', headers, body: JSON.stringify({ ...recipe.inference.body, model: selection.selected.id }), redirect: 'error', signal: AbortSignal.timeout(60000) }); }
    catch { blocked('required actual inference transport is unavailable or refused a redirect'); }
    assert.ok(response.ok, `actual inference returned ${response.status}`);
    const payload = await response.json(); assert.ok(payload.choices?.[0]?.message?.content, 'real inference must return assistant content');
    return { selectedModel: selection.selected.id, returnedModel: payload.model ?? null, responseContentSha256: hash(payload.choices[0].message.content), httpStatus: response.status };
  });

  await scenario('real memory publication persists outbox and receives a server receipt', async () => {
    if (!fs.existsSync(stateFile) || !projectId) blocked('persisted team/project prerequisite unavailable');
    const url = loopbackEndpoint(options.get('--memory-endpoint'), 'memory publication');
    await requireService(url, 'memory publication');
    url.pathname = '/api/v1/memory';
    const state = team('status', { state: stateFile });
    const queued = team('memory-queue', { state: stateFile, expectedRevision: state.revision,
      entry: { id: 'integration-lesson', content: 'Preserve the actual persisted handoff receipt.', scope: 'project', projectId, roleId: 'implementer' } });
    const published = team('memory-publish', { state: stateFile, expectedRevision: queued.revision,
      publication: { id: 'integration-lesson', provider: 'surreal-memory', url: url.href, scopeMapping: { scope: 'project' }, projectId } });
    assert.equal(published.publication.status, 'published'); assert.ok(published.publication.receipt.remoteId);
    assert.equal(team('status', { state: stateFile }).outbox[0].status, 'published');
    return { localId: 'integration-lesson', remoteId: published.publication.receipt.remoteId, userId: projectId, endpoint: url.href };
  });

  await scenario('production learning writer and Codex startup expose digest without private lesson text', () => {
    if (!projectId || !fs.existsSync(path.join(project, '.agent-team/project-routing.json'))) blocked('installed team/project prerequisite unavailable');
    // The production writer only enqueues. Codex parent recall must never query
    // either this unavailable endpoint or the optional actual memory service.
    env.SURREAL_MEMORY_URL = options.has('--memory-endpoint') ?
      loopbackEndpoint(options.get('--memory-endpoint'), 'learning writer').origin : 'http://127.0.0.1:1';
    env.PROMETHEUS_MEMORY_URL = env.SURREAL_MEMORY_URL;
    const privateText = `role-private integration lesson ${randomUUID()}`;
    const payloadFile = path.join(scratch, 'writer-payload.json'); write(payloadFile, { cwd: project, agent_type: 'implementer' });
    command('python3', [path.join(installed, 'shared/scripts/lib/learning_write.py'), '--text', privateText,
      '--visibility', 'agent', '--paths', 'src/feature.ts', '--payload', payloadFile, '--cwd', project]);
    const digestFile = path.join(env.PROMETHEUS_TEAM_DIGEST_DIR, projectId, `${definition.id}.jsonl`);
    assert.ok(fs.existsSync(digestFile), 'real writer must persist a digest');
    const digest = fs.readFileSync(digestFile, 'utf8'); assert.ok(!digest.includes(privateText));
    const output = command('bash', [path.join(installed, 'shared/scripts/sessionstart-learning.sh'), 'codex'], JSON.stringify({ cwd: project, turn_id: 'integration-parent' }));
    assert.match(output, /recorded a lesson/); assert.ok(!output.includes(privateText)); assert.ok(!output.includes('knowledgeGaps'));
    return { digestSha256: hash(digest), outputSha256: hash(output), privateTextDelivered: false };
  });

  await scenario('native CLI consumes actual connected control authority without local fallback', async () => {
    if (!projectId) blocked('native project prerequisite unavailable');
    const compare = (transportEnv, host) => {
      // Typed mutations return the remote commit receipt intact. Both local
      // execution and unreachable/ambiguous fallback mark committedLocally.
      // Audit alone is insufficient evidence because it can fall back locally.
      const mutation = JSON.parse(command(binary, ['kbd', '--path', project, 'phase', 'create',
        '--command-id', 'integration-connected-create', '--id', 'connected-phase', '--title', 'Connected integration phase'], undefined, transportEnv));
      assert.notEqual(mutation.committedLocally, true, 'local fallback cannot satisfy connected acceptance');
      assert.equal(mutation.state?.projectId, projectId);
      assert.ok(Number.isInteger(mutation.committedRevision), 'actual remote command receipt required');
      const connected = command(binary, ['kbd', '--path', project, 'audit', '--json'], undefined, transportEnv);
      const local = command(binary, ['kbd', '--path', project, 'audit', '--json']);
      assert.deepEqual(JSON.parse(connected), JSON.parse(local));
      return { ...host, projectId, committedRevision: mutation.committedRevision,
        remoteCommitReceiptSha256: hash(JSON.stringify(mutation)), auditSha256: hash(connected) };
    };
    if (options.has('--control-host-bin')) return await withControlHost(compare);
    const endpoint = loopbackEndpoint(options.get('--control-endpoint'), 'connected control');
    await requireService(endpoint, 'connected control');
    return compare({ PROMETHEUS_KBD_CONTROL_PLANE: '1', PROMETHEUS_CONTROL_ENDPOINT: endpoint.origin },
      { endpoint: endpoint.origin, hostProcess: 'actual external HTTP control host explicitly supplied by coordinator; Companion itself has no TCP proxy' });
  });
} catch (error) {
  results.push({ name: 'scenario prerequisites', result: error.blocked || ['ENOENT', 'EACCES', 'ENOSPC'].includes(error.code) ? 'BLOCKED' : 'FAIL', diagnosis: error.message });
}
const exitCode = results.some(row => row.result === 'FAIL') ? 1 : results.some(row => row.result === 'BLOCKED') ? 2 : 0;
const receipt = { schemaVersion: 1, result: exitCode === 0 ? 'PASS' : exitCode === 1 ? 'FAIL' : 'BLOCKED', scratch,
  installedRoot: installed, runtimeBinary: binary, binarySha256: binary && fs.existsSync(binary) ? hash(fs.readFileSync(binary)) : null,
  teamCliSha256: teamCli && fs.existsSync(teamCli) ? hash(fs.readFileSync(teamCli)) : null, node: process.version, results, commands,
  limits: 'This scenario covers its recorded entry points only; it does not close protected BDD, sites, external cross-project messages, final remote installation or historical closure.' };
if (scratchOwned && scratch && evidence && within(scratch, evidence) && fs.existsSync(scratch) && fs.lstatSync(scratch).isDirectory()) write(evidence, receipt);
process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
process.exitCode = exitCode;
