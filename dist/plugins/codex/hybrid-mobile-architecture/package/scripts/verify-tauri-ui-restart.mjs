// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/verify-tauri-ui-restart.mts
import { spawn, spawnSync } from "node:child_process";
import { closeSync, existsSync, mkdirSync, openSync, readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";
import { setTimeout } from "node:timers/promises";

// src/native-helpers/common.mts
var ToolError = class extends Error {
  constructor(message, code = 1) {
    super(message);
    this.code = code;
  }
  code;
};
function assert(condition, message, code = 1) {
  if (!condition) throw new ToolError(message, code);
}
async function main(fn) {
  try {
    await fn();
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = error instanceof ToolError ? error.code : 1;
  }
}

// src/native-helpers/verify-tauri-ui-restart.mts
var endpoint = "http://127.0.0.1:4444";
var elementKey = "element-6066-11e4-a52e-4f735466cecf";
async function stopTree(child) {
  if (!child.pid) return;
  if (process.platform === "win32") {
    spawnSync("taskkill.exe", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore", shell: false });
    return;
  }
  try {
    process.kill(-child.pid, "SIGTERM");
  } catch {
    return;
  }
  for (let attempt = 0; attempt < 5; attempt++) {
    await setTimeout(500);
    try {
      process.kill(-child.pid, 0);
    } catch {
      return;
    }
  }
  try {
    process.kill(-child.pid, "SIGKILL");
  } catch {
  }
}
async function request(path, method = "GET", body) {
  const response = await fetch(`${endpoint}${path}`, {
    method,
    headers: body === void 0 ? void 0 : { "content-type": "application/json" },
    body: body === void 0 ? void 0 : JSON.stringify(body)
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok || payload?.value?.error) {
    const detail = payload?.value?.message ?? JSON.stringify(payload);
    throw new Error(`WebDriver ${method} ${path} failed (${response.status}): ${detail}`);
  }
  return payload?.value;
}
async function waitFor(description, deadline, operation) {
  let last;
  while (Date.now() < deadline) {
    try {
      return await operation();
    } catch (error) {
      last = error;
    }
    await setTimeout(250);
  }
  throw new Error(`${description} did not become ready: ${last instanceof Error ? last.message : String(last)}`);
}
async function createSession(binary, deadline) {
  const value = await waitFor("Tauri WebDriver session", deadline, () => request("/session", "POST", {
    capabilities: { alwaysMatch: { browserName: "wry", "tauri:options": { application: binary } } }
  }));
  const id = value?.sessionId;
  assert(typeof id === "string" && id.length > 0, "WebDriver response omitted sessionId");
  return id;
}
async function element(session, selector, deadline) {
  const value = await waitFor(`element ${selector}`, deadline, () => request(`/session/${session}/element`, "POST", {
    using: "css selector",
    value: selector
  }));
  const id = value?.[elementKey];
  assert(typeof id === "string" && id.length > 0, `WebDriver response omitted element id for ${selector}`);
  return id;
}
async function texts(session, selector) {
  const values = await request(`/session/${session}/elements`, "POST", { using: "css selector", value: selector });
  return Promise.all(values.map((value) => request(`/session/${session}/element/${value[elementKey]}/text`)));
}
async function waitForNote(session, title, deadline) {
  await waitFor(`persisted note ${title}`, deadline, async () => {
    const values = await texts(session, '[data-testid="saved-notes"] li');
    if (!values.includes(title)) throw new Error(`note not visible; found ${JSON.stringify(values)}`);
  });
}
await main(async () => {
  assert(
    process.argv.length >= 4 && process.argv.length <= 5,
    "node scripts/verify-tauri-ui-restart.mjs <binary> <empty-app-data-dir> [timeout-seconds]",
    2
  );
  const binary = resolve(process.argv[2]);
  const data = resolve(process.argv[3]);
  const seconds = Number(process.argv[4] ?? 240);
  assert(existsSync(binary) && statSync(binary).isFile(), `Tauri binary is not a file: ${binary}`, 2);
  assert(!existsSync(data), `App-data proof directory must not already exist: ${data}`, 2);
  assert(Number.isFinite(seconds) && seconds > 0, "timeout must be positive", 2);
  mkdirSync(data, { recursive: true });
  const log = join(data, "tauri-webdriver.log");
  const handle = openSync(log, "w");
  const driver = spawn(process.env.TAURI_DRIVER ?? "tauri-driver", [], {
    env: { ...process.env, APP_DATA_DIR: data, GEN_UI_APP_DATA_DIR: data, RUST_LOG: "info" },
    stdio: ["ignore", handle, handle],
    detached: process.platform !== "win32",
    shell: false
  });
  closeSync(handle);
  let startupError;
  driver.on("error", (error) => {
    startupError = error;
  });
  const interrupt = () => {
    void stopTree(driver).finally(() => process.exit(130));
  };
  process.once("SIGINT", interrupt);
  process.once("SIGTERM", interrupt);
  let session;
  const deadline = Date.now() + seconds * 1e3;
  try {
    await waitFor("tauri-driver", deadline, async () => {
      assert(!startupError, `tauri-driver failed to launch: ${startupError?.message}`);
      assert(driver.exitCode === null && driver.signalCode === null, "tauri-driver exited before accepting a session");
      await request("/status");
    });
    const title = `WebDriver persistence ${Date.now()}`;
    session = await createSession(binary, deadline);
    const input = await element(session, '[data-testid="note-input"]', deadline);
    await request(`/session/${session}/element/${input}/value`, "POST", { text: title, value: [...title] });
    const save = await element(session, '[data-testid="save-note"]', deadline);
    await request(`/session/${session}/element/${save}/click`, "POST", {});
    await waitForNote(session, title, deadline);
    await request(`/session/${session}`, "DELETE");
    session = void 0;
    session = await createSession(binary, deadline);
    await waitForNote(session, title, deadline);
    assert(existsSync(join(data, "notes.sqlite3")), "UI workflow did not create the persisted SQLite database");
    process.stdout.write(`PASS: packaged Tauri UI -> invoke -> Rust -> SQLite survived application relaunch
app_data=${data}
`);
  } catch (error) {
    if (existsSync(log)) console.error(readFileSync(log, "utf8").split(/\r?\n/).slice(-200).join("\n"));
    throw error;
  } finally {
    if (session) await request(`/session/${session}`, "DELETE").catch(() => void 0);
    process.off("SIGINT", interrupt);
    process.off("SIGTERM", interrupt);
    await stopTree(driver);
  }
});
