// Owned scratch world and recorded-command helpers for the kbd-194-196 integration driver.
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
export const sha256 = (v) => crypto.createHash('sha256').update(v).digest('hex');
export const PHASE = 'dmg-phase';

const writeFile = (file, value) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, typeof value === 'string' ? value : `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 });
};
export { writeFile };

function deviceKey(file) {
  const pair = crypto.generateKeyPairSync('ed25519');
  const pub = pair.publicKey.export({ format: 'jwk' });
  const priv = pair.privateKey.export({ format: 'jwk' });
  writeFile(file, { schemaVersion: '1', keyId: `ed25519:${sha256(Buffer.from(pub.x, 'base64url'))}`,
    privateKey: Buffer.from(priv.d, 'base64url').toString('base64') });
}

/** An owned scratch world: child HOME/CODEX_HOME/XDG/data/key roots and a recorded command log. */
export class World {
  constructor(label, { prometheus, ROOT }) {
    this.prometheus = prometheus; this.ROOT = ROOT;
    this.base = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `kbd194-${label}-`)));
    const home = path.join(this.base, 'home');
    fs.mkdirSync(path.join(this.base, 'tmp'), { recursive: true });
    this.project = path.join(this.base, 'project');
    fs.mkdirSync(path.join(this.project, '.kbd-orchestrator'), { recursive: true });
    this.env = {
      PATH: `${path.dirname(prometheus)}${path.delimiter}${process.env.PATH}`,
      HOME: home, USERPROFILE: home, CODEX_HOME: path.join(home, '.codex'), CLAUDE_CONFIG_DIR: path.join(home, '.claude'),
      XDG_CONFIG_HOME: path.join(home, '.config'), XDG_DATA_HOME: path.join(home, '.local/share'), XDG_CACHE_HOME: path.join(home, '.cache'),
      TMPDIR: path.join(this.base, 'tmp'), PROMETHEUS_DATA_DIR: path.join(this.base, 'data'),
      PROMETHEUS_DEVICE_KEY_FILE: path.join(this.base, 'device-key.json'), PROMETHEUS_KBD_CONTROL_PLANE: '0',
      PROMETHEUS_CONTROL_ENDPOINT: 'http://127.0.0.1:1', PROMETHEUS_LEARNING_CORTEX: '0', PROMETHEUS_LEARNING_PK: '0',
      KBD_MEMORY_MCP_URL: 'http://127.0.0.1:1', UAR_MEMORY_MCP_URL: 'http://127.0.0.1:1',
      PYTHONDONTWRITEBYTECODE: '1', GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: os.devNull,
    };
    fs.mkdirSync(home, { recursive: true });
    deviceKey(this.env.PROMETHEUS_DEVICE_KEY_FILE);
    this.log = [];
    this.assertions = [];
  }

  run(exe, args, { cwd = this.project, env = this.env, input, timeout = 180_000 } = {}) {
    const startedAt = new Date().toISOString();
    const r = spawnSync(exe, args, { cwd, env, input, encoding: 'utf8', timeout, maxBuffer: 32 * 1024 * 1024 });
    const out = { exit: r.status ?? (r.error ? 127 : 1), stdout: r.stdout ?? '', stderr: `${r.stderr ?? ''}${r.error ?? ''}` };
    this.log.push({ argv: [path.basename(exe), ...args].map((a) => String(a).replace(this.base, '<scratch>')),
      envAllowlist: Object.keys(env).sort(), startedAt, exit: out.exit,
      stdout: out.stdout.slice(-6000), stderr: out.stderr.slice(-6000) });
    return out;
  }
  kbd(args, o) { return this.run(this.prometheus, ['kbd', '--path', this.project, ...args], o); }
  state() { return JSON.parse(this.kbd(['status', '--json']).stdout); }
  ok(name, cond, observed) { this.assertions.push({ name, ok: Boolean(cond), observed: typeof observed === 'string' ? observed.slice(0, 600) : observed }); return Boolean(cond); }

  /** A healthy canonical project: phase C1/T1 created through the CLI. */
  healthy(phase = PHASE) {
    const steps = [
      ['status', '--json'],
      ['phase', 'create', '--command-id', 'fx-create', '--id', phase, '--title', 'Healthy phase'],
      ['phase', 'activate', '--command-id', 'fx-activate', '--id', phase],
      ['guard', 'evaluate', '--boundary', 'phase', '--edge', 'before', '--subject', phase, '--json', '--repair-projections'],
      ['phase', 'transition', '--command-id', 'fx-start', '--id', phase, '--status', 'in-progress'],
      ['change', 'register', '--command-id', 'fx-change', '--phase', phase, '--id', 'C1', '--title', 'First change'],
      ['task', 'register', '--command-id', 'fx-task', '--phase', phase, '--change', 'C1', '--id', 'T1', '--title', 'Shared title'],
    ];
    for (const s of steps) { const o = this.kbd(s); if (o.exit !== 0) throw new Error(`healthy fixture step failed: ${s.join(' ')}\n${o.stdout}${o.stderr}`); }
    return this;
  }

  /** A damaged canonical project written through the real runtime API and signer. */
  damaged(seeder, variant) {
    this.kbd(['status', '--json']);
    const o = this.run(seeder, [this.project, this.env.PROMETHEUS_DATA_DIR, PHASE, variant]);
    if (o.exit !== 0) throw new Error(`seeder failed: ${o.stdout}${o.stderr}`);
    // The production phase guard writes the waypoint projection that reconcile reads.
    this.kbd(['guard', 'evaluate', '--boundary', 'phase', '--edge', 'before', '--subject', PHASE, '--json', '--repair-projections']);
    return this;
  }

  /** Remove checkpoints so the next process replays the signed journal from scratch. */
  coldReplay() {
    const projects = path.join(this.env.PROMETHEUS_DATA_DIR, 'prometheus/kbd/projects');
    for (const p of fs.existsSync(projects) ? fs.readdirSync(projects) : []) fs.rmSync(path.join(projects, p, 'checkpoints'), { recursive: true, force: true });
    return this.state();
  }

  journalKinds() {
    const counts = {};
    const projects = path.join(this.env.PROMETHEUS_DATA_DIR, 'prometheus/kbd/projects');
    for (const p of fs.existsSync(projects) ? fs.readdirSync(projects) : []) {
      const replicas = path.join(projects, p, 'replicas');
      for (const r of fs.existsSync(replicas) ? fs.readdirSync(replicas) : []) {
        const journal = path.join(replicas, r, 'events.jsonl');
        if (!fs.existsSync(journal)) continue;
        for (const line of fs.readFileSync(journal, 'utf8').split('\n').filter(Boolean)) {
          const kind = JSON.parse(line).kind?.type; counts[kind] = (counts[kind] ?? 0) + 1;
        }
      }
    }
    return counts;
  }

  result(attempted, extra = {}) {
    return { attempted, assertions: this.assertions, commands: this.log, ...extra };
  }
  cleanup() {
    // Only an owned mkdtemp base is ever removed. A plugin hook may still be writing into a session's
    // project (background learning worker), so retry and record a leak instead of throwing.
    if (!(path.basename(this.base).startsWith('kbd194-') && this.base.startsWith(fs.realpathSync(os.tmpdir())))) return;
    for (let attempt = 0; attempt < 6; attempt += 1) {
      try { fs.rmSync(this.base, { recursive: true, force: true, maxRetries: 4, retryDelay: 300 }); return; } catch { spawnSync('sleep', ['3']); }
    }
    this.leaked = this.base;
  }
}
