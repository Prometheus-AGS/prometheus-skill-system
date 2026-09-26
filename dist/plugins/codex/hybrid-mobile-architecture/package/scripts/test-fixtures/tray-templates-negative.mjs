// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/tray-templates-negative.mts
import { spawn, spawnSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// src/native-helpers/common.mts
import { existsSync, readdirSync, readFileSync, realpathSync, statSync } from "node:fs";
var ToolError = class extends Error {
  constructor(message, code = 1) {
    super(message);
    this.code = code;
  }
  code;
};
function text(path) {
  return readFileSync(path, "utf8");
}
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

// src/native-helpers/tray-templates-negative.mts
await main(async () => {
  const args = process.argv.slice(2);
  assert(args.every((arg) => arg === "--fast"), "usage: node scripts/test-fixtures/tray-templates-negative.mjs [--fast]");
  const fast = args.includes("--fast"), root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const gate = (cwd, limited, timeout) => new Promise((resolve2) => {
    const child = spawn(process.execPath, [join(cwd, "scripts/verify-tray-templates.mjs"), ...limited ? ["--fast"] : []], { cwd, shell: false, detached: process.platform !== "win32" });
    let output = "", timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      if (child.pid) {
        if (process.platform === "win32") spawnSync("taskkill.exe", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore", shell: false });
        else try {
          process.kill(-child.pid, "SIGKILL");
        } catch {
        }
      }
    }, timeout);
    child.stdout.on("data", (chunk) => {
      output += String(chunk);
    });
    child.stderr.on("data", (chunk) => {
      output += String(chunk);
    });
    child.on("error", (error) => {
      clearTimeout(timer);
      resolve2({ code: 127, output: output + error.message });
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      resolve2({ code: timedOut ? 124 : code ?? 1, output });
    });
  });
  const control = await gate(root, fast, fast ? 2e5 : 5e5);
  assert(control.code === (fast ? 2 : 0), `POSITIVE CONTROL FAILED; fixture checks would be meaningless
${control.output.split("\n").slice(-8).join("\n")}`);
  const cases = [["aggregator syntax error", "tauri-tray/health-aggregator/src/lib.rs.template", "pub struct", "pub strct", true], ["aggregator failing test", "tauri-tray/health-aggregator/src/lib.rs.template", "assert_eq!(", "assert_ne!(", true]];
  if (!fast) cases.push(["tray.rs E0596", "tauri-tray/tray.rs.template", "pub fn apply_accessory_policy(app: &mut App)", "pub fn apply_accessory_policy(app: &App)", false]);
  for (const [label, template, before, after, limited] of cases) {
    const work = mkdtempSync(join(tmpdir(), "hma-trayneg-"));
    try {
      mkdirSync(join(work, "scripts"), { recursive: true });
      mkdirSync(join(work, "assets"), { recursive: true });
      for (const file of ["scaffold-tauri-tray.mjs", "verify-tray-templates.mjs"]) cpSync(join(root, "scripts", file), join(work, "scripts", file));
      cpSync(join(root, "assets/templates/tauri-tray"), join(work, "assets/templates/tauri-tray"), { recursive: true });
      const path = join(work, "assets/templates", template), original = text(path);
      assert(original.includes(before), `fixture mutation marker missing: ${before}`);
      writeFileSync(path, original.replace(before, after));
      const result = await gate(work, limited, 4e5), needle = label.includes("syntax") ? "health-aggregator clippy failed" : label.includes("failing test") ? "health-aggregator tests failed" : "tray.rs does not compile";
      assert(result.code !== 0 && !(result.code === 2 && result.output.includes("PARTIAL")) && result.output.includes(needle), `${label}: gate failed to detect injected defect: ${result.output.slice(-1500)}`);
      console.log(`${label}: fails naming ${needle}`);
    } finally {
      rmSync(work, { recursive: true, force: true });
    }
  }
  console.log(`tray-templates-negative: all ${cases.length + 1} checks behaved correctly${fast ? " (FAST \u2014 tray.rs fixture NOT run; not full coverage)" : ""}`);
});
