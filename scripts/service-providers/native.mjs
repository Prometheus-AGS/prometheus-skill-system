import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const labels = {
  surrealdb: 'ai.prometheus.surrealdb-native', 'surreal-memory': 'ai.prometheus.surreal-memory-native',
  'pk-cherry': 'ai.prometheus.pk-cherry', 'forge-mcp': 'ai.prometheus.forge-mcp',
  'surface-bridge': 'ai.prometheus.surface-bridge', 'learning-worker': 'ai.prometheus.learning-worker',
  'liter-llm': 'ai.prometheus.liter-llm-api', 'prometheus-exec': 'ai.prometheus.exec',
  'openai-proxy': 'ai.prometheus.openai-proxy', 'prometheus-research': 'com.prometheus.research',
};
const binaries = {'surreal-memory':'surreal-memory-server', 'learning-worker':'prometheus-learning-worker', 'forge-mcp':'forge'};
const mac = process.platform === 'darwin';
const domain = () => 'gui/' + process.getuid();
const digest = contents => crypto.createHash('sha256').update(contents).digest('hex');

function run(program, args, options = {}) {
  const result = spawnSync(program, args, {encoding:'utf8', cwd:root, ...options});
  if (result.error || result.status !== 0) {
    throw new Error(`${program} ${args[0]} failed: ${result.error?.message || result.stderr?.trim() || result.status}`);
  }
  return result.stdout?.trim();
}

function layout(service, options) {
  const label = labels[service.id];
  if (!label) throw new Error('unknown native service ' + service.id);
  const scheduled = service.id === 'learning-worker';
  const units = mac ? [label + '.plist'] : [label + '.service', ...(scheduled ? [label + '.timer', label + '.path'] : [])];
  const home = options.home;
  return {label, units, starts: !mac && scheduled ? units.slice(1) : units,
    directory:path.join(home, mac ? 'Library/LaunchAgents' : '.config/systemd/user'),
    evidence:path.join(home, '.prometheus/service-providers/native', service.id + '.json')};
}

function supervisor(...args) {
  return spawnSync(mac ? 'launchctl' : 'systemctl', mac ? args : ['--user', ...args], {encoding:'utf8'});
}

function active(layout) {
  return mac ? supervisor('print', domain() + '/' + layout.label).status === 0
    : layout.starts.every(unit => supervisor('is-active', '--quiet', unit).status === 0);
}

function readOwnership(layout) {
  const evidence = fs.existsSync(layout.evidence) ? JSON.parse(fs.readFileSync(layout.evidence, 'utf8')) : {};
  for (const name of layout.units) {
    const file = path.join(layout.directory, name);
    const stat = fs.lstatSync(file, {throwIfNoEntry:false});
    if (stat && (!stat.isFile() || digest(fs.readFileSync(file)) !== evidence[name])) {
      throw new Error('unowned or modified service definition: ' + file);
    }
  }
  if (mac) {
    const result = supervisor('print', domain() + '/' + layout.label);
    if (result.status === 0 && (!evidence[layout.units[0]] ||
        !result.stdout.split('\n').some(line => line.trim() === 'path = ' + path.join(layout.directory, layout.units[0])))) {
      throw new Error('launchd label already registered by another owner: ' + layout.label);
    }
  } else {
    for (const unit of layout.units) {
      const loaded = supervisor('show', '--property=FragmentPath', '--value', unit).stdout?.trim();
      if (loaded && (!evidence[unit] || loaded !== path.join(layout.directory, unit))) {
        throw new Error('systemd unit already registered by another owner: ' + unit);
      }
    }
  }
  return evidence;
}

function atomic(file, contents) {
  const temp = file + '.' + crypto.randomUUID();
  try {
    fs.writeFileSync(temp, contents, {mode:0o600,flag:'wx'});
    fs.renameSync(temp, file);
  } finally { fs.rmSync(temp, {force:true}); }
}

function stopOwned(layout) {
  if (mac) {
    run('launchctl',['disable',domain()+'/'+layout.label]);
    if (active(layout)) run('launchctl', ['bootout', domain() + '/' + layout.label]);
  } else {
    run('systemctl', ['--user', 'disable', '--now', ...layout.starts]);
    if (layout.starts.length > 1) run('systemctl', ['--user', 'stop', layout.units[0]]);
  }
}

function supports(id) { return (mac || process.platform === 'linux') && Object.hasOwn(labels,id); }
function defaults() {
  return supports('surrealdb') ? Object.keys(labels).filter(id => id !== 'openai-proxy' &&
    (mac || !['prometheus-research','prometheus-exec','liter-llm'].includes(id))) : [];
}

async function validate(service,plan,options) {
  const layoutInfo = layout(service,options);
  readOwnership(layoutInfo);
  if (service.id === 'surface-bridge' && Number(service.port) !== 7890) {
    return 'native surface-bridge binds port 7890; Docker supports an alternate host port';
  }
  if (service.port && !service.endpoint.startsWith('http://')) return 'native server requires an HTTP loopback endpoint';
  if (service.port && new URL(service.endpoint).hostname === '[::1]') return 'native binary binds IPv4 loopback; choose 127.0.0.1';
  if (!mac && supervisor('show-environment').status !== 0) return 'systemd user manager unavailable';
  if (mac && supervisor('print',domain()).status !== 0) return 'launchd GUI domain unavailable';
  if (service.id === 'learning-worker' && plan.services.some(s => s.id === 'surreal-memory' && s.provider === 'docker') &&
      !options.env.PROMETHEUS_WORKER_MEMORY_GATEWAY) return 'Set PROMETHEUS_WORKER_MEMORY_GATEWAY for the authenticated Full memory API';
  if (service.id === 'surreal-memory' && plan.services.find(s => s.id === 'surrealdb')?.provider === 'off' && !options.env.SURREAL_ENDPOINT) {
    return 'Select a database or set SURREAL_ENDPOINT for native memory';
  }
  if (service.id === 'surrealdb') {
    const env={...process.env,...options.env,HOME:options.home};
    const file=resolveSurreal(env);
    try { fs.accessSync(file,fs.constants.X_OK); }
    catch { return 'Install the pinned SurrealDB executable at '+file+' or choose Docker'; }
  }
}
function resolveSurreal(env) {
  return run('bash',['-c','. "$1/config/defaults.env"; . "$1/shared/scripts/surreal-bin.sh"; resolve_surreal_bin "$HOME" "$PATH"','resolve',root],{env});
}

async function fileDigest(file) {
  const hash = crypto.createHash('sha256');
  for await (const chunk of fs.createReadStream(file)) hash.update(chunk);
  return hash.digest('hex');
}

function sourceStamp(id) {
  // Source commits are independent of unrelated pack commits. A dirty worktree
  // cannot prove equivalence and must go through the ordinary binary installer.
  const dirs = {'prometheus-exec':'crates/prometheus-exec','learning-worker':'tools/prometheus-knowledge',
    'pk-cherry':'tools/prometheus-knowledge','forge-mcp':'tools/forge-rs','surreal-memory':'tools/surreal-memory-server',
    'liter-llm':'tools/liter-llm','openai-proxy':'tools/openai-proxy','surface-bridge':'substrate/surface-bridge',
    'prometheus-research':'substrate/prometheus-research'};
  const dir = dirs[id];
  if (!dir || spawnSync('git',['diff','--quiet','HEAD','--',dir],{cwd:root}).status !== 0) return null;
  const pin = spawnSync('git',['rev-parse','HEAD:'+dir],{cwd:root,encoding:'utf8'});
  const version = JSON.parse(fs.readFileSync(path.join(root,'skill-system.json'),'utf8')).releaseVersion;
  return pin.status === 0 ? version + '/' + pin.stdout.trim() : null;
}

async function apply(service, plan, options) {
  const {id} = service;
  const l = layout(service, options);
  const previous = readOwnership(l);
  const home = options.home;
  const env = {...process.env, ...options.env, HOME:home};
  const binDir = env.PROMETHEUS_COMMAND_DIRECTORY || path.join(home,'.local/bin');
  env.PATH = [binDir, path.dirname(process.execPath), env.PATH].filter(Boolean).join(path.delimiter);
  if (id === 'surface-bridge' && Number(service.port) !== 7890) {
    throw new Error('The native surface-bridge listens on port 7890. Select Docker for a different host port.');
  }
  let surreal;
  const source = sourceStamp(id);
  if (id === 'surrealdb') {
    surreal = resolveSurreal(env);
    fs.accessSync(surreal, fs.constants.X_OK);
    fs.mkdirSync(path.join(home,'.prometheus/data/surrealdb'), {recursive:true,mode:0o700});
  } else {
    const installed = path.join(binDir,binaries[id] || id);
    const confirmed = source && source === previous.source && previous.binarySha256 &&
      fs.existsSync(installed) && await fileDigest(installed) === previous.binarySha256;
    if (!options.nativeBinariesReady && !confirmed) {
      run('bash', ['scripts/install-binaries.sh'], {env:{...env, PROMETHEUS_NATIVE_SERVICES:id,
        PROMETHEUS_SERVICE_BINARIES_ONLY:'1'}, stdio:['ignore',2,2]});
    }
    fs.accessSync(path.join(binDir,binaries[id] || id), fs.constants.X_OK);
  }
  if (id === 'prometheus-exec') {
    const state = path.join(home,'.prometheus/exec');
    const identity = path.join(state,'identity.json');
    for (const dir of [state,path.dirname(env.PROMETHEUS_EXEC_SOCKET || path.join(home,'.prometheus/run/prometheus-exec.sock'))]) {
      fs.mkdirSync(dir,{recursive:true,mode:0o700});
    }
    if (!fs.existsSync(identity)) run(path.join(binDir,id),['init','--identity',identity],{env});
  }
  if (id === 'learning-worker') {
    for (const queue of ['pending','retry','memory/pending','memory/retry']) {
      fs.mkdirSync(path.join(service.endpoint,queue),{recursive:true,mode:0o700});
    }
  }
  if (id === 'pk-cherry') fs.mkdirSync(path.join(mac ? root : home,'.prometheus/knowledge'),{recursive:true});
  for (const dir of [path.dirname(l.evidence), l.directory, path.join(home,'.prometheus/logs')]) {
    fs.mkdirSync(dir,{recursive:true,mode:0o700});
  }
  const input = JSON.stringify({root, home, label:l.label, binDir, service, env, surreal, user:os.userInfo().username});
  const files = JSON.parse(run('python3', [path.join(root,'scripts/service-providers/render-native.py')], {input}));
  const fingerprints = Object.fromEntries(l.units.map(name => [name,digest(files[name])]));
  const binary = id === 'surrealdb' ? surreal : path.join(binDir,binaries[id] || id);
  const binarySha256 = await fileDigest(binary);
  const changed = previous.binarySha256 !== binarySha256 || l.units.some(name => previous[name] !== fingerprints[name] || !fs.existsSync(path.join(l.directory,name)));
  if (changed && Object.keys(previous).length && active(l)) stopOwned(l);
  for (const name of l.units) {
    if (previous[name] !== fingerprints[name] || !fs.existsSync(path.join(l.directory,name))) {
      atomic(path.join(l.directory,name),files[name]);
      previous[name] = fingerprints[name];
      atomic(l.evidence,JSON.stringify(previous));
    }
  }
  previous.binarySha256 = binarySha256;
  previous.source = source;
  atomic(l.evidence,JSON.stringify(previous));
  if (mac) {
    if (!active(l)) {
      run('launchctl',['enable',domain()+'/'+l.label]);
      const job = path.join(l.directory,l.units[0]);
      try { run('launchctl',['bootstrap',domain(),job]); }
      catch {
        // bootout can release a previous registration asynchronously.
        await new Promise(resolve => setTimeout(resolve,750));
        run('launchctl',['bootstrap',domain(),job]);
      }
    }
  } else {
    if (changed) run('systemctl',['--user','daemon-reload']);
    if (!active(l)) run('systemctl',['--user','enable','--now',...l.starts]);
  }
}

async function stop(service, plan, options) {
  const l = layout(service,options);
  const evidence = readOwnership(l);
  if (l.units.some(name => !evidence[name])) throw new Error('native ownership record missing for '+service.id);
  stopOwned(l);
}
async function uninstall(service, plan, options) {
  const l = layout(service,options);
  await stop(service,plan,options);
  for (const name of l.units) fs.rmSync(path.join(l.directory,name),{force:true});
  if (!mac) run('systemctl',['--user','daemon-reload']);
  fs.rmSync(l.evidence,{force:true});
}
async function status(service,plan,options) {
  const l = layout(service,options);
  try {
    const owner = readOwnership(l);
    const owns = l.units.every(name => owner[name] && fs.existsSync(path.join(l.directory,name)));
    const alive = mac && service.id !== 'learning-worker'
      ? /^\s*pid = \d+$/m.test(supervisor('print',domain()+'/'+l.label).stdout || '') : active(l);
    const healthy = Boolean(owns && alive);
    return {healthy,status:healthy ? 'active' : 'inactive or unowned'};
  } catch (error) {
    return {healthy:false,status:error.message};
  }
}

export default {defaults,supports,validate,apply,stop,uninstall,status};
