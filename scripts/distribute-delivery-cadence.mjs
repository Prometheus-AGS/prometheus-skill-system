import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash, randomUUID } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const OWNER = 'prometheus-delivery-cadence';
const RECEIPT = '.cadence-install.json';
const ADAPTERS = ['shared/scripts/cadence-kbd-adapter.mjs', 'shared/scripts/cadence-karpathy-adapter.mjs'];
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const json = value => Buffer.from(`${JSON.stringify(value, null, 2)}\n`);
const fail = message => { throw new Error(message); };

function options(argv) {
  const args = { home: os.homedir(), sourceRoot: path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'),
    targets: 'all', dryRun: false, adoptIdentical: false, isolated: false };
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (flag === '--dry-run') args.dryRun = true;
    else if (flag === '--adopt-identical') args.adoptIdentical = true;
    else if (flag === '--help') args.help = true;
    else if (['--home', '--source-root', '--targets'].includes(flag)) {
      const value = argv[++index];
      if (!value || value.startsWith('--')) fail(`Missing value for ${flag}`);
      if (flag === '--home') { args.home = path.resolve(value); args.isolated = true; }
      else if (flag === '--source-root') args.sourceRoot = path.resolve(value);
      else args.targets = value;
    } else fail(`Unknown argument: ${flag}`);
  }
  return args;
}

// Existing parent aliases are preserved; this installer never creates a symlink.
function physicalPath(file) {
  if (fs.existsSync(file)) return fs.realpathSync(file);
  const parent = path.dirname(file);
  if (parent === file) return file;
  return path.join(physicalPath(parent), path.basename(file));
}

function targetRoots(args) {
  const kimi = !args.isolated && process.env.KIMI_CODE_HOME || path.join(args.home, '.kimi-code');
  const config = !args.isolated && process.env.XDG_CONFIG_HOME || path.join(args.home, '.config');
  const roots = {
    codex: path.join(args.home, '.codex/skills'),
    claude: path.join(args.home, '.claude/skills'),
    'kimi-code': path.resolve(kimi, 'skills'),
    minimax: path.join(args.home, '.minimax/skills'),
    zed: path.join(args.home, '.agents/skills'),
    opencode: path.resolve(config, 'opencode/skills'),
  };
  const selected = args.targets === 'all' ? Object.keys(roots) : [...new Set(args.targets.split(','))];
  if (!selected.length || selected.some(id => !Object.hasOwn(roots, id))) {
    fail(`Use --targets all or comma-separated IDs: ${Object.keys(roots).join(',')}`);
  }
  return selected.map(id => ({ id, destination: path.join(roots[id], 'delivery-cadence') }));
}

function readTree(root, omitReceipt = false) {
  const files = new Map();
  function visit(directory, relative = '') {
    for (const name of fs.readdirSync(directory).sort()) {
      const key = relative ? `${relative}/${name}` : name;
      if (omitReceipt && key === RECEIPT) continue;
      const file = path.join(directory, name);
      const stat = fs.lstatSync(file);
      if (stat.isSymbolicLink()) fail(`Refusing symlink in payload: ${file}`);
      if (stat.isDirectory()) visit(file, key);
      else if (stat.isFile()) files.set(key, fs.readFileSync(file));
      else fail(`Unsupported payload entry: ${file}`);
    }
  }
  visit(root);
  return files;
}

function hashes(files) {
  return Object.fromEntries([...files].sort(([a], [b]) => a.localeCompare(b)).map(([key, bytes]) => [key, sha256(bytes)]));
}

function equal(left, right) {
  const keys = Object.keys(left).sort();
  return keys.length === Object.keys(right).length && keys.every(key => left[key] === right[key]);
}

function supportFiles(sourceRoot) {
  const files = new Map();
  const pending = [...ADAPTERS];
  const recorder = 'shared/scripts/record-progress.mjs';
  if (fs.existsSync(path.join(sourceRoot, recorder))) pending.push(recorder);
  while (pending.length) {
    const relative = path.posix.normalize(pending.pop());
    if (files.has(relative)) continue;
    if (relative.startsWith('../') || path.posix.isAbsolute(relative)) fail(`Adapter escapes source root: ${relative}`);
    const file = path.join(sourceRoot, ...relative.split('/'));
    if (!fs.existsSync(file)) fail(`Missing declared adapter dependency: ${relative}`);
    const stat = fs.lstatSync(file);
    if (!stat.isFile() || stat.isSymbolicLink()) fail(`Adapter dependency must be a regular file: ${file}`);
    const bytes = fs.readFileSync(file);
    files.set(relative, bytes);
    for (const match of bytes.toString('utf8').matchAll(/^\s*(?:import|export)\s+(?:[^'";]*?\sfrom\s*)?['"](\.[^'"]+)['"]/gm)) {
      pending.push(path.posix.join(path.posix.dirname(relative), match[1]));
    }
  }
  return files;
}

function inspect(plan, args) {
  const stat = fs.lstatSync(plan.destination, { throwIfNoEntry: false });
  if (!stat) return { ...plan, action: 'install' };
  if (!stat.isDirectory() || stat.isSymbolicLink()) fail(`Refusing non-directory destination: ${plan.destination}`);
  const actual = hashes(readTree(plan.destination, true));
  const expected = hashes(plan.files);
  const receiptPath = path.join(plan.destination, RECEIPT);
  if (!fs.existsSync(receiptPath)) {
    if (!args.adoptIdentical || !equal(actual, expected)) {
      fail(`Unmanaged destination: ${plan.destination}. Only exact payloads can be adopted with --adopt-identical.`);
    }
    return { ...plan, action: 'adopt' };
  }
  if (!fs.lstatSync(receiptPath).isFile() || fs.lstatSync(receiptPath).isSymbolicLink()) fail(`Invalid ownership receipt: ${receiptPath}`);
  const receipt = JSON.parse(fs.readFileSync(receiptPath, 'utf8'));
  if (receipt.schemaVersion !== 1 || receipt.owner !== OWNER || !receipt.files || !equal(actual, receipt.files)) {
    fail(`Ownership mismatch or locally edited payload: ${plan.destination}. Preserve and resolve the existing files before updating.`);
  }
  return { ...plan, action: equal(actual, expected) ? 'unchanged' : 'update' };
}

function writeTree(root, files) {
  for (const [relative, bytes] of files) {
    const file = path.join(root, ...relative.split('/'));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes, { flag: 'wx' });
  }
}

function apply(plan, args, backupRoot) {
  if (plan.action === 'unchanged') return null;
  // Recheck ownership immediately before replacement, after the all-target preflight.
  const current = inspect(plan, args);
  if (current.action !== plan.action) fail(`Destination changed since preflight: ${plan.destination}`);
  fs.mkdirSync(path.dirname(plan.destination), { recursive: true });
  const staging = fs.mkdtempSync(path.join(path.dirname(plan.destination), '.cadence-stage-'));
  let backup = null;
  try {
    writeTree(staging, plan.files);
    fs.writeFileSync(path.join(staging, RECEIPT), json({ schemaVersion: 1, owner: OWNER,
      sourceRoot: args.sourceRoot, installedAt: new Date().toISOString(), files: hashes(plan.files) }), { flag: 'wx' });
    if (plan.action !== 'install') {
      backup = path.join(backupRoot, plan.id);
      fs.mkdirSync(path.dirname(backup), { recursive: true });
      fs.cpSync(plan.destination, backup, { recursive: true, errorOnExist: true, force: false });
      if (!equal(hashes(readTree(backup)), hashes(readTree(plan.destination)))) fail(`Backup does not match: ${backup}`);
      fs.rmSync(plan.destination, { recursive: true });
    }
    try { fs.renameSync(staging, plan.destination); }
    catch (error) {
      if (backup && !fs.existsSync(plan.destination)) fs.cpSync(backup, plan.destination, { recursive: true });
      throw error;
    }
    return backup;
  } finally { fs.rmSync(staging, { recursive: true, force: true }); }
}

function main() {
  const args = options(process.argv.slice(2));
  if (args.help) {
    console.log('node scripts/distribute-delivery-cadence.mjs [--targets all|codex,claude,kimi-code,minimax,zed,opencode] [--home DIR] [--source-root DIR] [--dry-run] [--adopt-identical]');
    return;
  }
  const contractPath = path.join(args.sourceRoot, 'skill-system.json');
  if (!fs.existsSync(contractPath) || JSON.parse(fs.readFileSync(contractPath, 'utf8')).name !== 'prometheus-skill-pack') {
    fail('Global Cadence distribution requires the full prometheus-skill-pack source or its packaged distribution; mini is not a global installer.');
  }
  const source = ['skills/process/delivery-cadence', 'skills/delivery-cadence']
    .map(relative => path.join(args.sourceRoot, relative)).find(directory => fs.existsSync(path.join(directory, 'SKILL.md')));
  if (!source) fail(`Cadence skill not found under ${args.sourceRoot}`);
  if (fs.lstatSync(source).isSymbolicLink()) fail(`Source skill must be a real directory: ${source}`);
  const files = readTree(source);
  if (files.has(RECEIPT)) fail('Source contains a destination ownership receipt');
  const version = files.get('SKILL.md').toString('utf8').match(/^\s+version:\s*["']?([^"'\s]+)/m)?.[1];
  if (!version) fail('Cadence metadata.version is missing');
  const targets = targetRoots(args);
  const plans = targets.map(target => {
    const payload = new Map(files);
    if (target.id === 'minimax') payload.set('_meta.json', json({
      id: Number.parseInt(createHash('md5').update('delivery-cadence').digest('hex').slice(0, 8), 16),
      version, name: 'delivery-cadence', updated_at: Math.floor(fs.statSync(path.join(source, 'SKILL.md')).mtimeMs), platform: 'minimax',
    }));
    return { ...target, files: payload };
  });
  const supportRoot = path.join(args.home, '.prometheus/delivery-cadence/support');
  plans.push({ id: 'support', destination: supportRoot, files: supportFiles(args.sourceRoot) });
  const destinations = new Set();
  for (const plan of plans) {
    const physical = physicalPath(plan.destination);
    if (destinations.has(physical)) fail(`Selected targets share a physical destination: ${physical}. Select one of those targets.`);
    destinations.add(physical);
  }
  const inspected = plans.map(plan => inspect(plan, args));
  const backupRoot = path.join(args.home, '.prometheus/delivery-cadence/backups', `${Date.now()}-${randomUUID()}`);
  const result = { schemaVersion: 1, dryRun: args.dryRun, source, version,
    targets: inspected.map(plan => ({ id: plan.id, destination: plan.destination, action: plan.action, fileCount: plan.files.size,
      backup: args.dryRun ? (['update', 'adopt'].includes(plan.action) ? path.join(backupRoot, plan.id) : null) : apply(plan, args, backupRoot) })),
    adapters: Object.fromEntries(ADAPTERS.map(relative => [path.basename(relative, '.mjs'), path.join(supportRoot, relative)])),
    note: 'Reload native sessions to discover installed skills. File installation does not prove native harness execution; adapters require explicit configuration.' };
  console.log(JSON.stringify(result, null, 2));
}

try { main(); }
catch (error) { console.error(`distribute-delivery-cadence: ${error.message}`); process.exitCode = 1; }
