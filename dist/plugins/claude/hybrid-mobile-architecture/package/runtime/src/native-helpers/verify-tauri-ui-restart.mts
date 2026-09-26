import { spawn, spawnSync, type ChildProcess } from 'node:child_process';
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { assert, main } from './common.mjs';

const endpoint = 'http://127.0.0.1:4444';
const elementKey = 'element-6066-11e4-a52e-4f735466cecf';

async function stopTree(child: ChildProcess): Promise<void> {
  if (!child.pid) return;
  if (process.platform === 'win32') {
    spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore', shell: false });
    return;
  }
  try { process.kill(-child.pid, 'SIGTERM'); } catch { return; }
  for (let attempt = 0; attempt < 5; attempt++) {
    await setTimeout(500);
    try { process.kill(-child.pid, 0); } catch { return; }
  }
  try { process.kill(-child.pid, 'SIGKILL'); } catch { /* already stopped */ }
}

async function request(path: string, method = 'GET', body?: unknown): Promise<any> {
  const response = await fetch(`${endpoint}${path}`, {
    method,
    headers: body === undefined ? undefined : { 'content-type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok || payload?.value?.error) {
    const detail = payload?.value?.message ?? JSON.stringify(payload);
    throw new Error(`WebDriver ${method} ${path} failed (${response.status}): ${detail}`);
  }
  return payload?.value;
}

async function waitFor<T>(description: string, deadline: number, operation: () => Promise<T>): Promise<T> {
  let last: unknown;
  while (Date.now() < deadline) {
    try { return await operation(); } catch (error) { last = error; }
    await setTimeout(250);
  }
  throw new Error(`${description} did not become ready: ${last instanceof Error ? last.message : String(last)}`);
}

async function createSession(binary: string, deadline: number): Promise<string> {
  const value = await waitFor('Tauri WebDriver session', deadline, () => request('/session', 'POST', {
    capabilities: { alwaysMatch: { browserName: 'wry', 'tauri:options': { application: binary } } },
  }));
  const id = value?.sessionId;
  assert(typeof id === 'string' && id.length > 0, 'WebDriver response omitted sessionId');
  return id;
}

async function element(session: string, selector: string, deadline: number): Promise<string> {
  const value = await waitFor(`element ${selector}`, deadline, () => request(`/session/${session}/element`, 'POST', {
    using: 'css selector', value: selector,
  }));
  const id = value?.[elementKey];
  assert(typeof id === 'string' && id.length > 0, `WebDriver response omitted element id for ${selector}`);
  return id;
}

async function texts(session: string, selector: string): Promise<string[]> {
  const values = await request(`/session/${session}/elements`, 'POST', { using: 'css selector', value: selector });
  return Promise.all((values as Array<Record<string, string>>).map(value =>
    request(`/session/${session}/element/${value[elementKey]}/text`)));
}

async function waitForNote(session: string, title: string, deadline: number): Promise<void> {
  await waitFor(`persisted note ${title}`, deadline, async () => {
    const values = await texts(session, '[data-testid="saved-notes"] li');
    if (!values.includes(title)) throw new Error(`note not visible; found ${JSON.stringify(values)}`);
  });
}

await main(async () => {
  assert(process.argv.length >= 4 && process.argv.length <= 5,
    'node scripts/verify-tauri-ui-restart.mjs <binary> <empty-app-data-dir> [timeout-seconds]', 2);
  const binary = resolve(process.argv[2]!);
  const data = resolve(process.argv[3]!);
  const seconds = Number(process.argv[4] ?? 240);
  assert(existsSync(binary) && statSync(binary).isFile(), `Tauri binary is not a file: ${binary}`, 2);
  assert(!existsSync(data), `App-data proof directory must not already exist: ${data}`, 2);
  assert(Number.isFinite(seconds) && seconds > 0, 'timeout must be positive', 2);
  mkdirSync(data, { recursive: true });
  const log = join(data, 'tauri-webdriver.log');
  const handle = openSync(log, 'w');
  const driver = spawn(process.env.TAURI_DRIVER ?? 'tauri-driver', [], {
    env: { ...process.env, APP_DATA_DIR: data, GEN_UI_APP_DATA_DIR: data, RUST_LOG: 'info' },
    stdio: ['ignore', handle, handle],
    detached: process.platform !== 'win32',
    shell: false,
  });
  closeSync(handle);
  let startupError: Error | undefined;
  driver.on('error', error => { startupError = error; });
  const interrupt = () => { void stopTree(driver).finally(() => process.exit(130)); };
  process.once('SIGINT', interrupt);
  process.once('SIGTERM', interrupt);
  let session: string | undefined;
  const deadline = Date.now() + seconds * 1000;
  try {
    await waitFor('tauri-driver', deadline, async () => {
      assert(!startupError, `tauri-driver failed to launch: ${startupError?.message}`);
      assert(driver.exitCode === null && driver.signalCode === null, 'tauri-driver exited before accepting a session');
      await request('/status');
    });
    const title = `WebDriver persistence ${Date.now()}`;
    session = await createSession(binary, deadline);
    const input = await element(session, '[data-testid="note-input"]', deadline);
    await request(`/session/${session}/element/${input}/value`, 'POST', { text: title, value: [...title] });
    const save = await element(session, '[data-testid="save-note"]', deadline);
    await request(`/session/${session}/element/${save}/click`, 'POST', {});
    await waitForNote(session, title, deadline);
    await request(`/session/${session}`, 'DELETE');
    session = undefined;
    session = await createSession(binary, deadline);
    await waitForNote(session, title, deadline);
    assert(existsSync(join(data, 'notes.sqlite3')), 'UI workflow did not create the persisted SQLite database');
    process.stdout.write(`PASS: packaged Tauri UI -> invoke -> Rust -> SQLite survived application relaunch\napp_data=${data}\n`);
  } catch (error) {
    if (existsSync(log)) console.error(readFileSync(log, 'utf8').split(/\r?\n/).slice(-200).join('\n'));
    throw error;
  } finally {
    if (session) await request(`/session/${session}`, 'DELETE').catch(() => undefined);
    process.off('SIGINT', interrupt);
    process.off('SIGTERM', interrupt);
    await stopTree(driver);
  }
});
