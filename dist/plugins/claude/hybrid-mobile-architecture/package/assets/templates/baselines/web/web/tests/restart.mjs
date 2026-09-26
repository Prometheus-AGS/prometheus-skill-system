// TJ-ARCH-MOB-001 compliant
import { chromium } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { dirname, resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { once } from 'node:events';
import assert from 'node:assert/strict';
const web = resolve(dirname(fileURLToPath(import.meta.url)), '..'), project = resolve(web, '..');
function run(command, args, cwd) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', shell: false });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr || result.stdout || 'command failed');
  return result.stdout;
}
run(process.execPath, ['scripts/build.mjs'], web);
run('cargo', ['build', '--manifest-path', 'server/Cargo.toml'], project);
const metadata = JSON.parse(run('cargo', ['metadata', '--no-deps', '--format-version', '1', '--manifest-path', 'server/Cargo.toml'], project));
const binary = join(metadata.target_directory, 'debug', '__APP_CRATE___server' + (process.platform === 'win32' ? '.exe' : ''));
const data = mkdtempSync(join(tmpdir(), 'notes-restart-'));
let child, browser;
async function start() {
  child = spawn(binary, [], { cwd: project, env: { ...process.env, APP_DATA_DIR: data, PORT: '0', WEB_DIST_DIR: join(web, 'dist') }, stdio: ['ignore','pipe','pipe'], shell: false });
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('Server startup timed out')), 30000);
    let output = '';
    child.once('error', error => { clearTimeout(timer); reject(error); });
    child.once('exit', code => { clearTimeout(timer); reject(new Error('Server exited: ' + code)); });
    child.stderr.on('data', data => process.stderr.write(data));
    child.stdout.on('data', data => { output += data; const match = output.match(/http:\/\/127\.0\.0\.1:\d+/); if (match) { clearTimeout(timer); resolve(match[0]); } });
  });
}
async function stop() { if (child && child.exitCode === null) { const stopped = once(child, 'exit'); child.kill(); await stopped; } }
try {
  let url = await start();
  browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto(url);
  if (process.env.EXPECTED_CAPABILITY) {
    await page.getByText(process.env.EXPECTED_CAPABILITY, { exact: false }).waitFor();
  }
  await page.getByLabel('New note', { exact: true }).fill('A note that survives restart');
  await page.getByRole('button', { name: 'Save note', exact: true }).click();
  await page.getByText('A note that survives restart', { exact: true }).waitFor();
  await stop(); url = await start(); await page.goto(url);
  await page.getByText('A note that survives restart', { exact: true }).waitFor();
  assert.equal((await page.request.get(url + '/api/runtime')).status(), 503);
  await page.getByRole('button', { name: 'Save note', exact: true }).click();
  await page.getByRole('alert').filter({ hasText: 'enter a note' }).waitFor();
  console.log('PASS: real browser -> Axum -> Rust SQLite -> process restart recovery; invalid note rejected; UAR unavailable explicitly');
} finally { await browser?.close(); await stop(); rmSync(data, { recursive: true, force: true }); }
