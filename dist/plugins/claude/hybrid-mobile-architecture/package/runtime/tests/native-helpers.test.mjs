import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { copyPathExact } from './support.mjs';
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const invoke = (script, args, options = {}) => spawnSync(process.execPath, [script, ...args], { encoding: 'utf8', ...options });

test('standalone progress payload records mirrored wiki and valid events with private-root override', () => {
  const root = mkdtempSync(join(tmpdir(), 'helper ü space-'));
  try {
    const app = join(root, 'Existing App'); mkdirSync(app);
    assert.equal(spawnSync('git', ['init', app]).status, 0);
    const script = join(root, 'record-progress.mjs');
    copyPathExact(join(repo, 'skills/karpathy-progress-memory/scripts/record-progress.mjs'), script);
    const result = invoke(script, ['--phase', 'native', '--title', 'Portable "helpers"', '--summary', `Updated ${app}`, '--evidence', 'Built and exercised', '--next', 'Windows runner'], { cwd: app, env: { ...process.env, PROMETHEUS_PRIVATE_ROOT: join(root, 'private') } });
    assert.equal(result.status, 0, result.stderr);
    const [local, privateCopy] = result.stdout.trim().split(/\r?\n/);
    assert.equal(readFileSync(local, 'utf8'), readFileSync(privateCopy, 'utf8'));
    assert.ok(readFileSync(local, 'utf8').includes('Updated $REPO_ROOT'));
    const event = JSON.parse(readFileSync(join(app, '.prometheus/events.jsonl'), 'utf8'));
    assert.equal(event.project_root, '$REPO_ROOT');
    assert.equal(event.payload.title, 'Portable "helpers"');
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('progress validation fails before creating directories for secrets or invalid phase', () => {
  const root = mkdtempSync(join(tmpdir(), 'helper rejected-'));
  try {
    const script = join(repo, 'skills/karpathy-progress-memory/scripts/record-progress.mjs');
    const args = ['--phase', 'valid', '--title', 'Change', '--summary', 'api_key=do-not-store', '--evidence', 'evidence', '--next', 'next'];
    const secret = invoke(script, args, { cwd: root });
    assert.equal(secret.status, 3, secret.stderr);
    args[1] = '../escape'; args[5] = 'safe';
    assert.equal(invoke(script, args, { cwd: root }).status, 2);
    assert.ok(!existsSync(join(root, '.prometheus')));
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('Docusaurus refuses an existing destination and platform helpers reject malformed invocations', () => {
  const root = mkdtempSync(join(tmpdir(), 'existing site ü-'));
  try {
    const result = invoke(join(repo, 'skills/build-branded-docusaurus/scripts/scaffold.mjs'), [root, 'Site', 'https://example.com', '/']);
    assert.equal(result.status, 1); assert.match(result.stderr, /refusing to overwrite/);
    assert.equal(invoke(join(repo, 'assets/templates/scripts/android/build.mjs'), ['production']).status, 64);
    assert.equal(invoke(join(repo, 'assets/templates/scripts/android/verify-device-runtime-gates.mjs'), []).status, 64);
    assert.equal(invoke(join(repo, 'assets/templates/scripts/android/verify-native-inference-gates.mjs'), ['missing.apk'], { env: { ...process.env, KNOWME_PLATFORM_NATIVE: join(root, 'absent-native-binary') } }).status, 127);
    assert.equal(invoke(join(repo, 'scripts/verify-tauri-ui-restart.mjs'), []).status, 2);
    assert.equal(invoke(join(repo, 'scripts/install-tauri-webdriver.mjs'), ['--check']).status, 0);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('generated Flutter and Rust bridge sources receive deterministic architecture markers', () => {
  const root = mkdtempSync(join(tmpdir(), 'generated markers-'));
  try {
    const dart = join(root, 'mobile/lib/bridge/generated/api/notes.dart');
    const riverpod = join(root, 'mobile/lib/features/notes/providers/notes.g.dart');
    const rust = join(root, 'rust/gen_ui_ffi/src/frb_generated.rs');
    for (const path of [dart, riverpod, rust]) { mkdirSync(dirname(path), { recursive: true }); writeFileSync(path, '// generated\n'); }
    const script = join(repo, 'scripts/mark-generated-sources.mjs');
    assert.equal(invoke(script, [root]).status, 0);
    assert.equal(invoke(script, [root, '--check']).status, 0);
    for (const path of [dart, riverpod, rust]) assert.match(readFileSync(path, 'utf8'), /^\/\/ TJ-ARCH-MOB-001 compliant\n\/\/ generated\n$/);
    assert.equal(invoke(script, [root]).status, 0);
    for (const path of [dart, riverpod, rust]) assert.equal(readFileSync(path, 'utf8').match(/TJ-ARCH-MOB-001/g)?.length, 1);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('standalone canonical and project-template skills have identical portable scripts', () => {
  for (const [skill, scripts] of [['build-branded-docusaurus', ['scaffold', 'verify', 'build-site']], ['karpathy-progress-memory', ['record-progress']]]) {
    for (const script of scripts) assert.deepEqual(readFileSync(join(repo, 'skills', skill, 'scripts', `${script}.mjs`)), readFileSync(join(repo, 'templates/project-skills', skill, 'scripts', `${script}.mjs`)));
  }
});
